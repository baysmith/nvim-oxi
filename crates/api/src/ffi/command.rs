use types::*;

use crate::opts::*;

#[cfg(not(feature = "neovim-nightly"))]
pub(crate) type ParseCmdOutput = Dictionary;

#[cfg(feature = "neovim-nightly")]
pub(crate) type ParseCmdOutput = crate::types::KeyDict_cmd;

#[cfg_attr(
    all(target_os = "windows", target_env = "msvc"),
    link(name = "nvim.exe", kind = "raw-dylib", modifiers = "+verbatim")
)]
extern "C" {
    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/command.c#L938
    pub(crate) fn nvim_buf_create_user_command(
        #[cfg(any(feature = "neovim-0-9", feature = "neovim-nightly"))]
        channel_id: u64,
        buf: BufHandle,
        name: NonOwning<String>,
        command: NonOwning<Object>,
        opts: *const CreateCommandOpts,
        err: *mut Error,
    );

    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/buffer.c#L145
    pub(crate) fn nvim_buf_del_user_command(
        buf: BufHandle,
        name: NonOwning<String>,
        err: *mut Error,
    );

    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/command.c#L1243
    pub(crate) fn nvim_buf_get_commands(
        buf: BufHandle,
        opts: *const GetCommandsOpts,
        #[cfg(feature = "neovim-nightly")] arena: *mut Arena,
        err: *mut Error,
    ) -> Dictionary;

    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/command.c#L320
    pub(crate) fn nvim_cmd(
        channel_id: u64,
        cmd: *const crate::types::KeyDict_cmd,
        opts: *const CmdOpts,
        #[cfg(feature = "neovim-nightly")] arena: *mut Arena,
        err: *mut Error,
    ) -> String;

    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/command.c#L938
    pub(crate) fn nvim_create_user_command(
        #[cfg(any(feature = "neovim-0-9", feature = "neovim-nightly"))]
        channel_id: u64,
        name: NonOwning<String>,
        command: NonOwning<Object>,
        opts: *const CreateCommandOpts,
        err: *mut Error,
    );

    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/command.c#L949
    pub(crate) fn nvim_del_user_command(
        name: NonOwning<String>,
        err: *mut Error,
    );

    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/command.c#L1230
    pub(crate) fn nvim_get_commands(
        opts: *const GetCommandsOpts,
        #[cfg(feature = "neovim-nightly")] arena: *mut Arena,
        error: *mut Error,
    ) -> Dictionary;

    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/command.c#L98
    pub(crate) fn nvim_parse_cmd(
        src: NonOwning<String>,
        #[cfg(not(feature = "neovim-nightly"))] opts: NonOwning<Dictionary>,
        #[cfg(feature = "neovim-nightly")] opts: *const ParseCmdOpts,
        #[cfg(feature = "neovim-nightly")] arena: *mut Arena,
        error: *mut Error,
    ) -> ParseCmdOutput;
}
