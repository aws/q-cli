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
use fig_util::system_info::linux::{
    get_desktop_environment,
    get_display_server,
    DesktopEnvironment,
    DisplayServer,
};
use parking_lot::Mutex;
use tracing::{
    error,
    info,
    warn,
};

use self::x11::X11State;
use super::WindowGeometry;
use crate::event::NativeEvent;
use crate::native::linux::sway::SwayState;
use crate::webview::window::WindowId;
use crate::EventLoopProxy;

pub const SHELL: &str = "/bin/bash";

#[derive(Debug)]
#[allow(dead_code)] // we will definitely need inner_x and inner_y at some point
pub struct ActiveWindowData {
    inner_x: i32,
    inner_y: i32,
    outer_x: i32,
    outer_y: i32,
    scale: f32,
}

#[derive(Debug)]
pub enum DisplayServerState {
    X11(Arc<x11::X11State>),
    Sway(Arc<sway::SwayState>),
}

#[derive(Debug)]
pub struct NativeState {
    pub active_window_data: Mutex<Option<ActiveWindowData>>,
    pub display_server_state: Mutex<Option<DisplayServerState>>,
}

impl NativeState {
    pub fn new(_proxy: EventLoopProxy) -> Self {
        Self {
            active_window_data: Mutex::new(None),
            display_server_state: Mutex::new(None),
        }
    }

    pub fn handle(&self, _event: NativeEvent) -> Result<()> {
        Ok(())
    }

    pub fn get_window_geometry(&self) -> Option<WindowGeometry> {
        match &*self.display_server_state.lock() {
            Some(DisplayServerState::X11(x11_state)) => x11_state
                .active_window
                .lock()
                .as_ref()
                .and_then(|window| window.window_geometry.clone()),
            _ => None,
        }
    }

    pub fn position_window(&self, _window_id: &WindowId, x: i32, y: i32, fallback: impl FnOnce()) {
        match &*self.display_server_state.lock() {
            Some(DisplayServerState::Sway(sway)) => {
                if let Err(err) = sway.sway_tx.send(sway::SwayCommand::PositionWindow {
                    x: x as i64,
                    y: y as i64,
                }) {
                    tracing::warn!(%err, "Failed to send sway command");
                }
            },
            _ => fallback(),
        }
    }
}

pub async fn init(proxy: EventLoopProxy, native_state: Arc<NativeState>) -> Result<()> {
    let proxy_ = proxy.clone();
    match get_display_server() {
        Ok(DisplayServer::X11) => {
            info!("Detected X11 server");

            let x11_state = Arc::new(X11State {
                active_window: Mutex::new(None),
            });
            *native_state.display_server_state.lock() = Some(DisplayServerState::X11(x11_state.clone()));

            tokio::spawn(async { x11::handle_x11(proxy_, x11_state).await });
        },
        Ok(DisplayServer::Wayland) => {
            info!("Detected Wayland server");

            match get_desktop_environment() {
                Ok(env @ DesktopEnvironment::Gnome | env @ DesktopEnvironment::Plasma) => info!("Detected {env:?}"),
                Ok(DesktopEnvironment::Sway) => {
                    if let Ok(sway_socket) = std::env::var("SWAYSOCK") {
                        info!(%sway_socket, "Detected sway");
                        let (sway_tx, sway_rx) = flume::unbounded();
                        let sway_state = Arc::new(SwayState {
                            active_window_rect: Mutex::new(None),
                            active_terminal: Mutex::new(None),
                            sway_tx,
                        });
                        *native_state.display_server_state.lock() = Some(DisplayServerState::Sway(sway_state.clone()));
                        tokio::spawn(async { sway::handle_sway(proxy_, sway_state, sway_socket, sway_rx).await });
                    }
                },
                Ok(env) => warn!("Detected non wayland compositor {env:?}"),
                Err(err) => error!(%err, "Unknown wayland compositor"),
            }
        },
        Err(err) => error!(%err, "Unable to detect display server"),
    }

    icons::init()?;
    ibus::init(proxy.clone(), native_state.clone()).await?;

    Ok(())
}

static WM_REVICED_DATA: AtomicBool = AtomicBool::new(false);

pub fn autocomplete_active() -> bool {
    WM_REVICED_DATA.load(Ordering::Relaxed)
}

pub mod gtk {
    pub fn init() -> Result<(), gtk::glib::BoolError> {
        use gtk::glib::translate::{
            from_glib,
            ToGlibPtr,
        };
        use gtk::{
            ffi,
            glib,
            is_initialized,
            set_initialized,
        };

        if gtk::is_initialized_main_thread() {
            return Ok(());
        } else if is_initialized() {
            panic!("Attempted to initialize GTK from two different threads.");
        }
        unsafe {
            let name = vec!["fig"];
            if from_glib(ffi::gtk_init_check(&mut 1, &mut name.to_glib_none().0)) {
                let result: bool = from_glib(glib::ffi::g_main_context_acquire(
                    gtk::glib::ffi::g_main_context_default(),
                ));
                if !result {
                    return Err(glib::bool_error!("Failed to acquire default main context"));
                }
                set_initialized();
                Ok(())
            } else {
                Err(glib::bool_error!("Failed to initialize GTK"))
            }
        }
    }
}
