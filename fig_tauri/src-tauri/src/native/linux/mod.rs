mod sway;
mod x11;

use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{
    error,
    info,
};

use crate::window::WindowEvent;

#[derive(Debug)]
pub struct NativeState;

impl NativeState {
    pub fn new(window_event_sender: UnboundedSender<WindowEvent>) -> Self {
        match DisplayServer::detect() {
            Ok(DisplayServer::X11) => {
                info!("Detected X11 server");
                tauri::async_runtime::spawn_blocking(move || x11::handle_x11(window_event_sender));
            },
            Ok(DisplayServer::Wayland) => {
                info!("Detected Wayland server");
                if let Ok(sway_socket) = std::env::var("SWAYSOCK") {
                    info!("Using sway socket: {sway_socket}");
                    tauri::async_runtime::spawn(async { sway::handle_sway(window_event_sender, sway_socket).await });
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
