#![cfg(target_os = "macos")]

#[macro_use]
extern crate objc;

pub mod accessibility;
pub mod bundle;
pub mod caret_position;
mod core_graphics_patch;
pub mod image;
pub mod platform_api;
mod util;
mod window_position;
pub mod window_server;

use core_graphics::window::CGWindowID;
use platform_api::PlatformApi;
pub use util::{
    NSArray,
    NSString,
    NSStringRef,
    NotificationCenter,
    Subscription,
    NSURL,
};
use window_position::WindowPosition;
pub use window_server::{
    WindowServer,
    WindowServerEvent,
};

#[derive(Debug)]
pub struct ActiveWindow {
    pub window_id: CGWindowID,
    // Or pass complete application object???
    pub process_id: u64,
    pub position: WindowPosition,
    pub bundle_id: String,
}

pub fn get_position() -> Option<WindowPosition> {
    PlatformApi::get_position().ok()
}

pub fn get_active_window() -> Option<ActiveWindow> {
    PlatformApi::get_active_window().ok()
}
