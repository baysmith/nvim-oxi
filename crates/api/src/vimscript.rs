use types::{self as nvim, conversion::FromObject, Array, Object};

use crate::choose;
use crate::ffi::vimscript::*;
use crate::types::*;
use crate::Result;

/// Binding to [`nvim_call_dict_function()`][1].
///
/// Calls a VimL dictionary function with the given arguments, returning the
/// result of the funtion call.
///
/// [1]: https://neovim.io/doc/user/api.html#nvim_call_dict_function()
pub fn call_dict_function<Args, Ret>(
    dict: &str,
    func: &str,
    args: Args,
) -> Result<Ret>
where
    Args: Into<Array>,
    Ret: FromObject,
{
    let dict = Object::from(nvim::String::from(dict));
    let func = nvim::String::from(func);
    let args = args.into();
    let mut err = nvim::Error::new();
    let res = unsafe {
        nvim_call_dict_function(
            dict.non_owning(),
            func.non_owning(),
            args.non_owning(),
            #[cfg(feature = "neovim-nightly")]
            types::arena(),
            &mut err,
        )
    };
    choose!(err, Ok(Ret::from_object(res)?))
}

/// Binding to [`nvim_call_function()`][1].
///
/// Calls a VimL function with the given arguments, returning the result of the
/// funtion call.
///
/// [1]: https://neovim.io/doc/user/api.html#nvim_call_function()
pub fn call_function<Args, Ret>(func: &str, args: Args) -> Result<Ret>
where
    Args: Into<Array>,
    Ret: FromObject,
{
    let func = nvim::String::from(func);
    let args = args.into();
    let mut err = nvim::Error::new();
    let res = unsafe {
        nvim_call_function(
            func.non_owning(),
            args.non_owning(),
            #[cfg(feature = "neovim-nightly")]
            types::arena(),
            &mut err,
        )
    };
    choose!(err, Ok(Ret::from_object(res)?))
}

/// Binding to [`nvim_command()`][1].
///
/// Executes an Ex command.
///
/// [1]: https://neovim.io/doc/user/api.html#nvim_command()
pub fn command(command: &str) -> Result<()> {
    let command = nvim::String::from(command);
    let mut err = nvim::Error::new();
    unsafe { nvim_command(command.non_owning(), &mut err) };
    choose!(err, ())
}

/// Binding to [`nvim_eval()`][1].
///
/// Evaluates a VimL expression.
///
/// [1]: https://neovim.io/doc/user/api.html#nvim_eval()
pub fn eval<V>(expr: &str) -> Result<V>
where
    V: FromObject,
{
    let expr = nvim::String::from(expr);
    let mut err = nvim::Error::new();
    let output = unsafe {
        nvim_eval(
            expr.non_owning(),
            #[cfg(feature = "neovim-nightly")]
            types::arena(),
            &mut err,
        )
    };
    choose!(err, Ok(V::from_object(output)?))
}

/// Binding to [`nvim_parse_expression()`][1].
///
/// Parses a VimL expression.
///
/// [1]: https://neovim.io/doc/user/api.html#nvim_parse_expression()
pub fn parse_expression(
    expr: &str,
    flags: &str,
    include_highlight: bool,
) -> Result<ParsedVimLExpression> {
    let expr = nvim::String::from(expr);
    let flags = nvim::String::from(flags);
    let mut err = nvim::Error::new();
    let dict = unsafe {
        nvim_parse_expression(
            expr.non_owning(),
            flags.non_owning(),
            include_highlight,
            #[cfg(feature = "neovim-nightly")]
            types::arena(),
            &mut err,
        )
    };
    choose!(err, Ok(ParsedVimLExpression::from_object(dict.into())?))
}
