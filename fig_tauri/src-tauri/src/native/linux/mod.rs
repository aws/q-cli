mod sway;
mod x11;

use anyhow::Result;
use tracing::{
    error,
    info,
};
use wry::application::event_loop::EventLoopProxy;

use crate::FigEvent;

pub use x11::CURSOR_POSITION_KIND;

#[derive(Debug)]
pub struct NativeState;

impl NativeState {
    pub fn new(proxy: EventLoopProxy<FigEvent>) -> Self {
        match DisplayServer::detect() {
            Ok(DisplayServer::X11) => {
                info!("Detected X11 server");
                tokio::task::spawn_blocking(move || x11::handle_x11(proxy));
            },
            Ok(DisplayServer::Wayland) => {
                info!("Detected Wayland server");
                if let Ok(sway_socket) = std::env::var("SWAYSOCK") {
                    info!("Using sway socket: {sway_socket}");
                    tokio::spawn(async { sway::handle_sway(proxy, sway_socket).await });
                } else {
                    error!("Unknown wayland compositor");
                }
            },
            Err(err) => {
                error!("{err}");
            },
        }

        Self
    }
}

enum DisplayServer {
    X11,
    Wayland,
}

impl DisplayServer {
    fn detect() -> Result<Self> {
        match std::env::var("XDG_SESSION_TYPE") {
            Ok(ref session_type) if session_type == "wayland" => Ok(Self::Wayland),
            _ => Ok(Self::X11),
        }
    }
}
