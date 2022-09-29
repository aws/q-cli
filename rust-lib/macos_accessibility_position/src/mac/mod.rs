pub mod caret;
mod core_graphics_patch;
mod platform_api;
mod window_position;
mod window_server;
use platform_api::MacPlatformApi;
use window_server::WindowServerApi;

use crate::general::platform_api::PlatformApi;
use crate::general::window_server::WindowServer;

pub fn init_platform_api() -> impl PlatformApi {
    MacPlatformApi {}
}

pub fn init_window_server() -> impl WindowServer {
    WindowServerApi {}
}
