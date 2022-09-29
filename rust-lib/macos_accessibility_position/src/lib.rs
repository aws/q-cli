#![cfg(target_os = "macos")]

mod general;

pub mod mac;
pub use general::active_window::ActiveWindow;
use general::platform_api::PlatformApi;
pub use general::window_position::WindowPosition;
use general::window_server::WindowServer;
use mac::{
    init_platform_api,
    init_window_server,
};

pub fn get_position() -> Option<WindowPosition> {
    let api = init_platform_api();
    api.get_position().ok()
}

pub fn get_active_window() -> Option<ActiveWindow> {
    let api = init_platform_api();
    api.get_active_window().ok()
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn register_observer() {
    let api = init_window_server();
    api.register_observer();
}
