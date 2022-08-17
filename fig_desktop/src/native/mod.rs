cfg_if::cfg_if! {
    if #[cfg(target_os="linux")] {
        mod linux;
        pub use self::linux::*;
    } else if #[cfg(target_os="macos")] {
        mod macos;
        pub use self::macos::*;
    } else if #[cfg(windows)] {
        mod windows;
        pub use self::windows::*;
    } else {
        compile_error!("Unsupported platform");
    }
}

#[derive(Debug, Clone)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
