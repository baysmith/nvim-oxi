use types::*;

use crate::opts::*;

#[cfg_attr(
    all(target_os = "windows", target_env = "msvc"),
    link(name = "nvim.exe", kind = "raw-dylib", modifiers = "+verbatim")
)]
extern "C" {
    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/options.c#L289
    pub(crate) fn nvim_get_all_options_info(
        #[cfg(feature = "neovim-nightly")] arena: *mut Arena,
        err: *mut Error,
    ) -> Dictionary;

    // https://github.com/neovim/neovim/blob/master/src/nvim/api/options.c#L305
    #[cfg(feature = "neovim-nightly")]
    pub(crate) fn nvim_get_option_info2(
        name: NonOwning<String>,
        opts: *const OptionOpts,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Dictionary;

    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/options.c#L146
    pub(crate) fn nvim_get_option_value(
        name: NonOwning<String>,
        opts: *const OptionOpts,
        err: *mut Error,
    ) -> Object;

    // https://github.com/neovim/neovim/blob/v0.9.0/src/nvim/api/options.c#L232
    pub(crate) fn nvim_set_option_value(
        #[cfg(any(feature = "neovim-0-9", feature = "neovim-nightly"))]
        channel_id: u64,
        name: NonOwning<String>,
        value: NonOwning<Object>,
        opts: *const OptionOpts,
        err: *mut Error,
    );
}
