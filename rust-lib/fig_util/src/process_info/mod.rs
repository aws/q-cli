use std::path::PathBuf;
use std::{
    fmt,
    str,
};

use cfg_if::cfg_if;

macro_rules! pid_decl {
    ($typ:ty) => {
        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct Pid(pub(crate) $typ);

        impl From<$typ> for Pid {
            fn from(v: $typ) -> Self {
                Self(v)
            }
        }
        impl From<Pid> for $typ {
            fn from(v: Pid) -> Self {
                v.0
            }
        }
        impl str::FromStr for Pid {
            type Err = <$typ as str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(<$typ>::from_str(s)?))
            }
        }
        impl fmt::Display for Pid {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "macos"))] {
        use nix::libc::pid_t;

        pid_decl!(pid_t);

        impl From<nix::unistd::Pid> for Pid {
            fn from(pid: nix::unistd::Pid) -> Self {
                Pid(pid.as_raw())
            }
        }

        impl From<Pid> for nix::unistd::Pid {
            fn from(pid: Pid) -> Self {
                nix::unistd::Pid::from_raw(pid.0)
            }
        }
    } else if #[cfg(target_os = "windows")] {
        pid_decl!(usize);
    } else {
        compile_error!("Unsupported platform");
    }
}

pub trait PidExt {
    fn current() -> Self;
    fn parent(&self) -> Option<Pid>;
    fn exe(&self) -> Option<PathBuf>;
}

cfg_if!(
    if #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::*;
    } else if #[cfg(target_os = "macos")] {
        mod macos;
        pub use macos::*;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::*;
    } else {
        compile_error!("Unsupported platform");
    }
);

pub fn get_parent_process_exe() -> Option<PathBuf> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_name() {
        let process_pid = Pid::current();
        let parent_pid = process_pid.parent().unwrap();
        let parent_exe = parent_pid.exe().unwrap();
        let parent_name = parent_exe.file_name().unwrap().to_str().unwrap();
        assert_eq!(parent_name, "cargo");
    }
}
