mod ibus;
pub mod icons;
pub mod integrations;
mod sway;
mod x11;

use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
use std::sync::Arc;

use anyhow::Result;
use parking_lot::Mutex;
use tracing::{
    error,
    info,
};

use super::WindowGeometry;
use crate::event::NativeEvent;
use crate::EventLoopProxy;

pub const SHELL: &str = "/bin/bash";

#[derive(Debug)]
pub struct ActiveWindowData {
    x: i32,
    y: i32,
    off_x: i32,
    off_y: i32,
}

#[derive(Debug)]
pub struct X11WindowData {
    pub id: x11rb::protocol::xproto::Window,
    pub class: Option<Vec<u8>>,
    pub instance: Option<Vec<u8>>,
    pub window_geometry: Option<WindowGeometry>,
}

#[derive(Debug)]
pub struct NativeState {
    pub active_window_data: Mutex<Option<ActiveWindowData>>,
    x11_active_window: Mutex<Option<X11WindowData>>,
}

impl NativeState {
    pub fn new(_proxy: EventLoopProxy) -> Self {
        Self {
            active_window_data: Mutex::new(None),
            x11_active_window: Mutex::new(None),
        }
    }

    pub fn handle(&self, _event: NativeEvent) -> Result<()> {
        Ok(())
    }

    pub fn get_window_geometry(&self) -> Option<WindowGeometry> {
        let active_window = self.x11_active_window.lock();
        active_window.as_ref().and_then(|window| window.window_geometry.clone())
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

pub async fn init(proxy: EventLoopProxy, native_state: Arc<NativeState>) -> Result<()> {
    let proxy_ = proxy.clone();
    match DisplayServer::detect() {
        Ok(DisplayServer::X11) => {
            info!("Detected X11 server");
            let native_state_ = native_state.clone();
            tokio::spawn(async { x11::handle_x11(proxy_, native_state_).await });
        },
        Ok(DisplayServer::Wayland) => {
            info!("Detected Wayland server");
            if let Ok(sway_socket) = std::env::var("SWAYSOCK") {
                info!("Using sway socket: {sway_socket}");
                tokio::spawn(async { sway::handle_sway(proxy_, sway_socket).await });
            } else {
                error!("Unknown wayland compositor");
            }
        },
        Err(err) => error!("Unable to detect display server: {err}"),
    }

    icons::init()?;
    ibus::init(proxy.clone(), native_state.clone()).await?;

    Ok(())
}

static WM_REVICED_DATA: AtomicBool = AtomicBool::new(false);

pub fn autocomplete_active() -> bool {
    WM_REVICED_DATA.load(Ordering::Relaxed)
}
