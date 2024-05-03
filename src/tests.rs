use std::any::Any;
use std::env;
use std::fmt::{Debug, Display};
use std::panic::{self, Location, UnwindSafe};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};
use std::thread;

use miniserde::json;

/// Returns the `target` directory in which cargo will place the compiled
/// artifacts for the crate whose manifest is located at `manifest_dir`.
pub fn target_dir(manifest_dir: &Path) -> PathBuf {
    let output = Command::new(
        env::var("CARGO").ok().unwrap_or_else(|| "cargo".to_owned()),
    )
    .arg("metadata")
    .arg("--format-version=1")
    .arg("--no-deps")
    .current_dir(manifest_dir)
    .output()
    .unwrap();

    let object: json::Object =
        json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();

    let target_dir = match object.get("target_directory").unwrap() {
        json::Value::String(s) => s,
        _ => panic!("must be string value"),
    };

    target_dir.into()
}

/// This function is used as the body of the `#[nvim_oxi::plugin]` generated by
/// the `#[nvim_oxi::test]` macro.
pub fn plugin_body<F, R>(test_body: F)
where
    F: FnOnce() -> R + UnwindSafe,
    R: IntoResult,
{
    let panic_info: Arc<OnceLock<PanicInfo>> = Arc::default();

    {
        let panic_info = panic_info.clone();

        panic::set_hook(Box::new(move |info| {
            let _ = panic_info.set(info.into());
        }));
    }

    let result = match panic::catch_unwind(|| test_body().into_result()) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(Failure::Error(err.to_string())),
        Err(_) => Err(Failure::Panic(panic_info.get().unwrap().clone())),
    };

    exit(result);
}

/// This function is used as the body of the `#[nvim_oxi::plugin]` generated by
/// the `#[nvim_oxi::test]` macro when the `test-terminator` feature is enabled
/// and the test function takes a `TestTerminator` argument.
#[cfg(feature = "test-terminator")]
pub fn plugin_body_with_terminator<F>(test_body: F)
where
    F: FnOnce(TestTerminator),
{
    let lock = Arc::new(OnceLock::<Result<(), Failure>>::new());

    let handle = {
        let lock = lock.clone();

        crate::libuv::AsyncHandle::new(move || {
            let result = lock.get().unwrap().clone();
            crate::schedule(move |()| {
                exit(result);
                Ok(())
            });
            Ok::<_, std::convert::Infallible>(())
        })
    }
    .unwrap();

    test_body(TestTerminator { lock, handle });
}

/// A handle used to terminate a test annotated by [`test`](crate::test).
///
/// The `test` macro works by turning the annotated function into its own
/// plugin, which is then loaded by Neovim and evalutated by `require`ing it
/// when the test is run, before immediately quitting.
///
/// When testing asynchronous code this can be problematic, as the test may
/// need to continue running after the generated plugin has been `require`d.
///
/// To allow for this, the test function can take a `TestTerminator` as its
/// only argument. This allows the test to be terminated asynchronously by
/// calling [`terminate`](Self::terminate).
///
/// Note that if the `TestTerminator` is dropped without first calling
/// `terminate`, the test will run forever.
#[cfg(feature = "test-terminator")]
pub struct TestTerminator {
    lock: Arc<OnceLock<Result<(), Failure>>>,
    handle: crate::libuv::AsyncHandle,
}

#[cfg(feature = "test-terminator")]
impl TestTerminator {
    /// Terminates the test and consumes the `TestTerminator`.
    pub fn terminate<E: Display>(self, res: Result<(), TestFailure<'_, E>>) {
        let res = res.map_err(Into::into);
        let Ok(()) = self.lock.set(res) else { unreachable!() };
        self.handle.send().unwrap();
    }
}

/// The error type given to [`TestTerminator::terminate`].
///
/// The two variants of this enum represent the two ways a test can fail:
/// either by returning an error or by panicking.
pub enum TestFailure<'a, E> {
    /// This is used to indicate that the test failed due to an error being
    /// returned from the test function.
    Error(E),

    /// This is used to indicate that the test failed due to a panic. The
    /// [`PanicInfo`](std::panic::PanicInfo) contains information about the
    /// panic and can be obtained by calling
    /// [`set_hook`](std::panic::set_hook).
    Panic(&'a std::panic::PanicInfo<'a>),
}

fn exit(result: Result<(), Failure>) {
    if let Err(failure) = result {
        eprintln!("{failure}");
        crate::api::exec("cquit 1", false).unwrap();
    } else {
        crate::api::exec("qall!", false).unwrap();
    }
}

/// TODO: docs
pub fn test_body(
    crate_name: &str,
    manifest_dir: &str,
    plugin_name: &str,
    library_path: Option<impl AsRef<Path>>,
    extra_cmd: Option<&str>,
) -> Result<(), String> {
    panic::set_hook(Box::new(move |info| {
        let mut info = info
            .payload()
            .downcast_ref::<PanicInfo>()
            .cloned()
            .unwrap_or_else(|| info.into());

        if let Some(thread) = thread::current().name() {
            if !thread.is_empty() {
                info.thread = thread.to_owned();
            }
        }

        eprintln!("{}", info);
    }));

    let output = run_nvim_command(
        crate_name,
        manifest_dir,
        plugin_name,
        library_path,
        extra_cmd,
    )?
    .output()
    .map_err(|err| err.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout.trim();

    if !stdout.is_empty() {
        println!("{}", stdout)
    }

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.is_empty() {
        let msg = output
            .status
            .code()
            .map(|i| format!("Neovim exited with non-zero exit code: {i}"))
            .unwrap_or_else(|| String::from("Neovim segfaulted"));

        return Err(msg);
    }

    let Ok(failure) = Failure::from_str(&stderr) else {
        return Err(stderr.into_owned());
    };

    match failure {
        Failure::Error(err) => return Err(err),
        Failure::Panic(info) => panic::panic_any(info),
    }
}

/// TODO: docs
fn run_nvim_command(
    crate_name: &str,
    manifest_dir: &str,
    plugin_name: &str,
    library_path: Option<impl AsRef<Path>>,
    extra_cmd: Option<&str>,
) -> Result<Command, String> {
    let library_path = library_path
        .map(|path| path.as_ref().to_owned())
        .unwrap_or_else(|| {
            let library_name = format!(
                "{prefix}{crate_name}{suffix}",
                prefix = env::consts::DLL_PREFIX,
                suffix = env::consts::DLL_SUFFIX,
            );
            target_dir(Path::new(manifest_dir))
                .join("debug")
                .join(library_name)
        });

    if !library_path.exists() {
        return Err(format!(
            "Compiled library not found in '{}'. Please run `cargo build` \
             before running the tests.",
            library_path.display()
        ));
    }

    let load_library = format!(
        "lua local f = package.loadlib([[{}]], 'luaopen_{}'); f()",
        library_path.display(),
        plugin_name,
    );

    let mut command = Command::new("nvim");

    command
        .args(["-u", "NONE", "--headless"])
        .args(["-i", "NONE"])
        .args(["-c", "set noswapfile"])
        .args(extra_cmd.map(|cmd| ["-c", cmd]).unwrap_or_default())
        .args(["-c", &load_library]);

    Ok(command)
}

#[derive(Clone)]
struct PanicInfo {
    msg: String,
    thread: String,
    file: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
}

impl Debug for PanicInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "panic:{}", self.msg)?;

        write!(f, "\nthread:{}", self.thread)?;

        if let Some(file) = &self.file {
            write!(f, "\nfile:{file}")?;
        }

        if let Some(line) = self.line {
            write!(f, "\nline:{line}")?;
        }

        if let Some(column) = self.column {
            write!(f, "\ncolumn:{column}")?;
        }

        Ok(())
    }
}

impl Display for PanicInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "thread '{}' panicked", self.thread)?;

        if let Some(file) = &self.file {
            write!(f, " at {file}")?;

            if let (Some(line), Some(col)) = (self.line, self.column) {
                write!(f, ":{line}:{col}")?;
            }
        }

        write!(f, ":\n{}", self.msg)?;

        Ok(())
    }
}

impl FromStr for PanicInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut info = PanicInfo {
            msg: String::new(),
            thread: String::new(),
            file: None,
            line: None,
            column: None,
        };

        let (_, s) = s.split_once("panic:").ok_or(())?;

        let (msg, s) = s.split_once("thread:").ok_or(())?;
        info.msg = msg.trim().to_owned();

        let (thread, s) = s.split_once("file:").ok_or(())?;
        info.thread = thread.trim().to_owned();

        let (file, s) = s.split_once("line:").ok_or(())?;
        info.file = Some(file.trim().to_owned());

        let (line, s) = s.split_once("column:").ok_or(())?;
        info.line = Some(line.trim().parse().map_err(|_| ())?);

        let column = s.trim().parse().map_err(|_| ())?;
        info.column = Some(column);

        Ok(info)
    }
}

impl From<&panic::PanicInfo<'_>> for PanicInfo {
    fn from(info: &panic::PanicInfo) -> Self {
        let payload = info.payload();

        let msg = downcast_display::<&str>(payload)
            .or_else(|| downcast_display::<String>(payload))
            .or_else(|| downcast_display::<&String>(payload))
            .map(ToString::to_string)
            .unwrap_or_default();

        let current_thread = thread::current();

        let thread = match current_thread.name() {
            Some(name) if !name.is_empty() => name,
            _ => "<unnamed>",
        };

        Self {
            msg,
            thread: thread.to_owned(),
            file: info.location().map(|l| l.file().to_owned()),
            line: info.location().map(Location::line),
            column: info.location().map(Location::column),
        }
    }
}

#[derive(Clone)]
enum Failure {
    Error(String),
    Panic(PanicInfo),
}

impl Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Error(err) => write!(f, "error:{err}"),
            Failure::Panic(info) => write!(f, "{info:?}"),
        }
    }
}

impl FromStr for Failure {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("error:") {
            Some((_, msg)) => Ok(Failure::Error(msg.trim().to_owned())),
            None => PanicInfo::from_str(s).map(Self::Panic),
        }
    }
}

#[cfg(feature = "test-terminator")]
impl<E: Display> From<TestFailure<'_, E>> for Failure {
    fn from(err: TestFailure<'_, E>) -> Self {
        match err {
            TestFailure::Error(err) => Self::Error(err.to_string()),
            TestFailure::Panic(info) => Self::Panic(info.into()),
        }
    }
}

pub trait IntoResult {
    type Error: Display;

    fn into_result(self) -> Result<(), Self::Error>;
}

impl IntoResult for () {
    type Error = std::convert::Infallible;

    fn into_result(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<E: Display> IntoResult for Result<(), E> {
    type Error = E;

    fn into_result(self) -> Result<(), E> {
        self
    }
}

fn downcast_display<T: Any + Display>(
    value: &dyn Any,
) -> Option<&dyn Display> {
    value.downcast_ref::<T>().map(|msg| msg as &dyn Display)
}