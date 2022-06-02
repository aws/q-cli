pub mod icons;
mod sway;
mod x11;

use std::sync::Arc;

use anyhow::Result;
use parking_lot::Mutex;
use tracing::{
    error,
    info,
};
pub use x11::CURSOR_POSITION_KIND;

use crate::event::Event;
use crate::{
    EventLoopProxy,
    GlobalState,
};

pub const SHELL: &str = "/bin/bash";
pub const SHELL_ARGS: [&str; 3] = ["--noprofile", "--norc", "-c"];

#[derive(Debug, Default)]
pub struct NativeState {
    active_window: Mutex<Option<String>>,
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

pub async fn init(global_state: Arc<GlobalState>, proxy: EventLoopProxy) -> Result<()> {
    match DisplayServer::detect() {
        Ok(DisplayServer::X11) => {
            info!("Detected X11 server");
            tokio::spawn(async { x11::handle_x11(global_state, proxy).await });
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
