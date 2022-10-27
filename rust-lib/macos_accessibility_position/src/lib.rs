#![cfg(target_os = "macos")]

#[macro_use]
extern crate objc;

pub mod accessibility;
pub mod applications;
pub mod bundle;
pub mod caret_position;
pub mod image;
mod util;
pub mod window_server;

pub use util::{
    NSArray,
    NSArrayRef,
    NSString,
    NSStringRef,
    NotificationCenter,
    NSURL,
};
pub use window_server::{
    WindowServer,
    WindowServerEvent,
};
