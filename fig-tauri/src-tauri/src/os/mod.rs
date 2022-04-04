// all os-specific code under a unified interface

mod utils;

cfg_if::cfg_if!(
    if #[cfg(target_os="windows")] {
        mod windows;
        pub mod native {
            pub use super::windows::*;
            pub use super::utils::*;
        }
    } else if #[cfg(target_os="macos")] {
        mod macos;
        mod unix;
        pub mod native {
            pub use super::macos::*;
            pub use super::unix::*;
            pub use super::utils::*;
        }
    } else if #[cfg(target_os="linux")] {
        mod linux;
        mod unix;
        pub mod native {
            pub use super::linux::*;
            pub use super::unix::*;
            pub use super::utils::*;
        }
    } else {
        compile_error!("Unsupported platform");
    }
);
