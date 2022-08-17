#[cfg(target_os = "linux")]
mod ibus;
#[cfg(target_os = "linux")]
pub use ibus::*;

#[cfg(not(target_os = "linux"))]
pub fn _dummy() {}
