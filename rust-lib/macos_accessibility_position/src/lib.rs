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

use std::sync::Arc;

use flume::Sender;
use parking_lot::Mutex;
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
use window_server::subscribe_to_all;
pub use window_server::{
    WindowServer,
    WindowServerEvent,
};

#[derive(Debug)]
pub struct ActiveWindow {
    pub window_id: String,
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

#[allow(clippy::missing_safety_doc)]
pub unsafe fn register_observer(sender: Sender<WindowServerEvent>) -> Arc<Mutex<WindowServer>> {
    let server = WindowServer::new(sender);
    let window_server = Arc::new(Mutex::new(server));
    subscribe_to_all(&window_server);

    let server = window_server.clone();
    let mut locked = server.lock();
    locked.init();

    window_server
}
