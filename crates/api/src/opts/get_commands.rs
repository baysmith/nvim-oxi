/// Options passed to [`Buffer::get_commands()`](crate::Buffer::get_commands)
/// and [`get_commands()`](crate::get_commands).
#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct GetCommandsOpts {
    #[cfg(not(feature = "neovim-nightly"))]
    builtin: types::Object,

    #[cfg(feature = "neovim-nightly")]
    builtin: bool,
}

impl GetCommandsOpts {
    #[inline(always)]
    pub fn builder() -> GetCommandsOptsBuilder {
        GetCommandsOptsBuilder::default()
    }
}

#[derive(Clone, Default)]
pub struct GetCommandsOptsBuilder(GetCommandsOpts);

impl GetCommandsOptsBuilder {
    #[inline]
    pub fn builtin(&mut self, builtin: bool) -> &mut Self {
        #[cfg(not(feature = "neovim-nightly"))]
        {
            self.0.builtin = builtin.into();
        }
        #[cfg(feature = "neovim-nightly")]
        {
            self.0.builtin = builtin;
        }
        self
    }

    #[inline]
    pub fn build(&mut self) -> GetCommandsOpts {
        std::mem::take(&mut self.0)
    }
}
