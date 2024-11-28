#![cfg(target_os = "macos")]
// This is needed for objc
#![allow(unexpected_cfgs)]

#[macro_use]
extern crate objc;

pub mod accessibility;
pub mod applications;
pub mod bundle;
pub mod caret_position;
pub mod image;
pub mod os;
pub mod url;
mod util;
pub mod window_server;

pub use util::{
    NSArray,
    NSArrayRef,
    NSString,
    NSStringRef,
    NSURL,
    NotificationCenter,
    get_user_info_from_notification,
};
pub use window_server::{
    WindowServer,
    WindowServerEvent,
};
