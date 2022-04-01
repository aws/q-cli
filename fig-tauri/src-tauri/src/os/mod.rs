// contains all platform-specific code under a unified interface

mod linux;
mod macos;
mod utils;
mod windows;

pub mod native {
    cfg_if::cfg_if!(
        if #[cfg(target_os="linux")] {
            pub use super::linux::*;
        } else if #[cfg(target_os="windows")] {
            pub use super::windows::*;
        } else if #[cfg(target_os="macos")] {
            pub use super::macos::*;
        } else {
            compile_error!("Unsupported platform");
        }
    );

    pub use super::utils::*;
}
