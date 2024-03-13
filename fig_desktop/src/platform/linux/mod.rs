mod ibus;
pub mod icons;
pub mod integrations;
mod sway;
mod x11;

use std::borrow::Cow;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
use std::sync::Arc;

use fig_util::system_info::linux::{
    get_desktop_environment,
    get_display_server,
    DesktopEnvironment,
    DisplayServer,
};
use fig_util::Terminal;
use parking_lot::Mutex;
use tao::dpi::{
    LogicalPosition,
    PhysicalPosition,
    Position,
};
use tracing::{
    error,
    info,
    trace,
    warn,
};

use self::x11::X11State;
use super::PlatformBoundEvent;
use crate::platform::linux::sway::SwayState;
use crate::protocol::icons::{
    AssetSpecifier,
    ProcessedAsset,
};
use crate::webview::notification::WebviewNotificationsState;
use crate::webview::{
    FigIdMap,
    WindowId,
};
use crate::{
    EventLoopProxy,
    EventLoopWindowTarget,
};

static WM_REVICED_DATA: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
#[allow(dead_code)] // we will definitely need inner_x and inner_y at some point
pub(super) struct ActiveWindowData {
    inner_x: i32,
    inner_y: i32,
    outer_x: i32,
    outer_y: i32,
    scale: f32,
}

#[derive(Debug)]
pub(super) enum DisplayServerState {
    X11(Arc<x11::X11State>),
    Sway(Arc<sway::SwayState>),
}

#[derive(Debug)]
pub struct PlatformWindowImpl;

pub(super) struct PlatformStateImpl {
    pub(super) proxy: EventLoopProxy,
    pub(super) active_window_data: Mutex<Option<ActiveWindowData>>,
    pub(super) display_server_state: Mutex<Option<DisplayServerState>>,
    pub(super) active_terminal: Mutex<Option<Terminal>>,
}

impl PlatformStateImpl {
    pub(super) fn new(proxy: EventLoopProxy) -> Self {
        Self {
            proxy,
            active_window_data: Mutex::new(None),
            display_server_state: Mutex::new(None),
            active_terminal: Mutex::new(None),
        }
    }

    pub(super) fn handle(
        self: &Arc<Self>,
        event: PlatformBoundEvent,
        _: &EventLoopWindowTarget,
        _: &FigIdMap,
        _: &Arc<WebviewNotificationsState>,
    ) -> anyhow::Result<()> {
        match event {
            PlatformBoundEvent::Initialize => {
                let platform_state = self.clone();
                tokio::runtime::Handle::current().spawn(async move {
                    let proxy_ = platform_state.proxy.clone();
                    match get_display_server() {
                        Ok(DisplayServer::X11) => {
                            info!("Detected X11 server");

                            let x11_state = Arc::new(X11State::default());
                            *platform_state.display_server_state.lock() =
                                Some(DisplayServerState::X11(x11_state.clone()));

                            let platform_state_ = platform_state.clone();
                            tokio::spawn(async { x11::handle_x11(proxy_, x11_state, platform_state_).await });
                        },
                        Ok(DisplayServer::Wayland) => {
                            info!("Detected Wayland server");

                            match get_desktop_environment() {
                                Ok(env @ DesktopEnvironment::Gnome | env @ DesktopEnvironment::Plasma) => {
                                    info!("Detected {env:?}")
                                },
                                Ok(DesktopEnvironment::Sway) => {
                                    if let Ok(sway_socket) = std::env::var("SWAYSOCK") {
                                        info!(%sway_socket, "Detected sway");
                                        let (sway_tx, sway_rx) = flume::unbounded();
                                        let sway_state = Arc::new(SwayState {
                                            active_window_rect: Mutex::new(None),
                                            active_terminal: Mutex::new(None),
                                            sway_tx,
                                        });
                                        *platform_state.display_server_state.lock() =
                                            Some(DisplayServerState::Sway(sway_state.clone()));
                                        tokio::spawn(async {
                                            sway::handle_sway(proxy_, sway_state, sway_socket, sway_rx).await
                                        });
                                    }
                                },
                                Ok(env) => warn!("Detected non wayland compositor {env:?}"),
                                Err(err) => error!(%err, "Unknown wayland compositor"),
                            }
                        },
                        Err(err) => error!(%err, "Unable to detect display server"),
                    }

                    if let Err(err) = icons::init() {
                        error!(%err, "Unable to initialize icons");
                    }

                    if let Err(err) = ibus::init(platform_state.proxy.clone(), platform_state.clone()).await {
                        error!(%err, "Unable to initialize ibus");
                    }
                });
            },
            PlatformBoundEvent::InitializePostRun => {
                trace!("Ignoring initialize post run event");
            },
            PlatformBoundEvent::EditBufferChanged => {
                trace!("Ignoring edit buffer changed event");
            },
            PlatformBoundEvent::FullscreenStateUpdated { .. } => {
                trace!("Ignoring full screen state updated event");
            },
            PlatformBoundEvent::AccessibilityUpdated { .. } => {
                trace!("Ignoring accessibility updated event");
            },
            PlatformBoundEvent::AppWindowFocusChanged { .. } => {
                trace!("Ignoring app window focus changed event");
            },
            PlatformBoundEvent::CaretPositionUpdateRequested => {
                trace!("Ignoring caret position update requested event");
            },
            PlatformBoundEvent::WindowDestroyed { .. } => {
                trace!("Ignoring window destroyed event");
            },
            PlatformBoundEvent::ExternalWindowFocusChanged { .. } => {
                trace!("Ignoring external window focus changed event");
            },
            PlatformBoundEvent::AccessibilityUpdateRequested => {
                trace!("Ignoring accessibility update requested event");
            },
        }
        Ok(())
    }

    pub(super) fn position_window(
        &self,
        webview_window: &tao::window::Window,
        _window_id: &WindowId,
        position: Position,
    ) -> wry::Result<()> {
        match &*self.display_server_state.lock() {
            Some(DisplayServerState::Sway(sway)) => {
                let (x, y) = match position {
                    Position::Physical(PhysicalPosition { x, y }) => (x, y),
                    // TODO(grant): prob do something with logical position here
                    Position::Logical(LogicalPosition { x, y }) => (x as i32, y as i32),
                };

                if let Err(err) = sway.sway_tx.send(sway::SwayCommand::PositionWindow {
                    x: x as i64,
                    y: y as i64,
                }) {
                    tracing::warn!(%err, "Failed to send sway command");
                }
            },
            _ => {
                webview_window.set_outer_position(position);
            },
        };
        Ok(())
    }

    pub(super) fn get_cursor_position(&self) -> Option<crate::utils::Rect> {
        None
    }

    pub(super) fn get_active_window(&self) -> Option<super::PlatformWindow> {
        match &*self.display_server_state.lock() {
            Some(DisplayServerState::X11(x11_state)) => x11_state.active_window.lock().as_ref().and_then(|window| {
                window.window_geometry.map(|rect| super::PlatformWindow {
                    rect,
                    inner: PlatformWindowImpl,
                })
            }),
            _ => None,
        }
    }

    pub(super) async fn icon_lookup(asset: &AssetSpecifier) -> Option<ProcessedAsset> {
        match asset {
            AssetSpecifier::Named(name) => icons::lookup(name).await,
            AssetSpecifier::PathBased(path) => path
                .metadata()
                .ok()
                .and_then(|metadata| {
                    let name = if metadata.is_dir() {
                        Some("folder")
                    } else if metadata.is_file() {
                        Some("text-x-generic-template")
                    } else {
                        None
                    };
                    name.and_then(icons::lookup)
                })
                .or_else(|| {
                    icons::lookup(if path.to_str().map(|x| x.ends_with('/')).unwrap_or_default() {
                        "folder"
                    } else {
                        "text-x-generic-template"
                    })
                }),
        }
    }

    pub(super) fn shell() -> Cow<'static, str> {
        for shell in &["bash", "zsh", "sh"] {
            if let Ok(shell_path) = which::which(shell) {
                return shell_path.to_string_lossy().to_string().into();
            }
        }
        "/bin/bash".into()
    }

    pub fn accessibility_is_enabled() -> Option<bool> {
        None
    }
}

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
            let name = ["codewhisperer"];
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
