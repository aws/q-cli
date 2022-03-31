// contains all platform-specific code under a unified interface

pub mod linux;
pub mod macos;
pub mod windows;

cfg_if::cfg_if!(
    if #[cfg(target_os="linux")] {
        pub use linux as native;
    } else if #[cfg(target_os="windows")] {
        pub use windows as native;
    } else if #[cfg(target_os="macos")] {
        pub use macos as native;
    } else {
        compile_error!("Unsupported platform");
    }
);
