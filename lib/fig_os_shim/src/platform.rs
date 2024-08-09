use cfg_if::cfg_if;

use crate::Shim;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Os {
    Mac,
    Linux,
}

#[derive(Default, Debug, Clone)]
pub struct Platform(inner::Inner);

mod inner {
    use super::*;

    #[derive(Default, Debug, Clone)]
    pub(super) enum Inner {
        #[default]
        Real,
        Fake(Os),
    }
}

impl Platform {
    /// Returns a new fake [Platform].
    pub fn new_fake(os: Os) -> Self {
        Self(inner::Inner::Fake(os))
    }

    /// Returns the current [Os].
    pub fn os(&self) -> Os {
        use inner::Inner;
        match &self.0 {
            Inner::Real => {
                cfg_if! {
                    if #[cfg(target_os = "macos")] {
                        Os::Mac
                    } else if #[cfg(target_os = "linux")] {
                        Os::Linux
                    } else {
                        compile_error!("unsupported platform");
                    }
                }
            },
            Inner::Fake(os) => *os,
        }
    }
}

impl Shim for Platform {
    fn is_real(&self) -> bool {
        matches!(self.0, inner::Inner::Real)
    }
}
