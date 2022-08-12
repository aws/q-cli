pub mod icons;
mod sway;
mod x11;

use anyhow::Result;
use parking_lot::Mutex;
use tracing::{
    error,
    info,
};

use crate::event::NativeEvent;
use crate::EventLoopProxy;

pub const SHELL: &str = "/bin/bash";

#[derive(Debug)]
pub struct NativeState {
    _active_window: Mutex<Option<String>>,
}

impl NativeState {
    pub fn new(_proxy: EventLoopProxy) -> Self {
        Self {
            _active_window: Mutex::new(None),
        }
    }

    pub fn handle(&self, _event: NativeEvent) -> Result<()> {
        Ok(())
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

pub async fn init(proxy: EventLoopProxy) -> Result<()> {
    match DisplayServer::detect() {
        Ok(DisplayServer::X11) => {
            info!("Detected X11 server");
            tokio::spawn(async { x11::handle_x11(proxy).await });
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

    icons::init()?;

    Ok(())
}
