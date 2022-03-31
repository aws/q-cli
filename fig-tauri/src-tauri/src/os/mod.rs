// contains all platform-specific code under a unified interface

pub mod linux;
pub mod windows;

#[cfg(target_os="linux")]
pub use linux as native;

#[cfg(target_os="windows")]
pub use windows as native;
