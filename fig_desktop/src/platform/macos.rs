use std::borrow::Cow;
use std::ffi::CString;
use std::slice;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
use std::sync::Arc;

use accessibility_sys::{
    AXUIElementCreateSystemWide,
    AXUIElementSetMessagingTimeout,
};
use anyhow::Context;
use cocoa::base::{
    id,
    NO,
    YES,
};
use fig_util::Terminal;
use macos_accessibility_position::accessibility::accessibility_is_enabled;
use macos_accessibility_position::caret_position::{
    get_caret_position,
    CaretPosition,
};
use macos_accessibility_position::{
    get_active_window,
    register_observer,
    NSString,
    NotificationCenter,
    WindowServer,
    WindowServerEvent,
};
use objc::declare::MethodImplementation;
use objc::runtime::{
    class_addMethod,
    objc_getClass,
    Class,
    Object,
    Sel,
    BOOL,
};
use objc::{
    msg_send,
    sel,
    sel_impl,
    Encode,
    EncodeArguments,
    Encoding,
};
use once_cell::sync::Lazy;
use parking_lot::{
    Mutex,
    RwLock,
};
use tracing::{
    debug,
    warn,
};
use wry::application::dpi::{
    LogicalPosition,
    LogicalSize,
    Position,
};
use wry::application::platform::macos::{
    ActivationPolicy,
    EventLoopWindowTargetExtMacOS,
};

use super::{
    PlatformBoundEvent,
    PlatformWindow,
    WindowGeometry,
};
use crate::event::{
    Event,
    WindowEvent,
};
use crate::icons::{
    AssetKind,
    AssetSpecifier,
    ProcessedAsset,
};
use crate::utils::Rect;
use crate::webview::window::WindowId;
use crate::webview::FigIdMap;
use crate::{
    EventLoopProxy,
    EventLoopWindowTarget,
    AUTOCOMPLETE_ID,
    AUTOCOMPLETE_WINDOW_TITLE,
    DASHBOARD_ID,
};

pub const DEFAULT_CARET_WIDTH: i32 = 10;

static UNMANAGED: Lazy<Unmanaged> = Lazy::new(|| Unmanaged {
    event_sender: RwLock::new(Option::<EventLoopProxy>::None),
    window_server: RwLock::new(Option::<Arc<Mutex<WindowServer>>>::None),
});

struct Unmanaged {
    event_sender: RwLock<Option<EventLoopProxy>>,
    window_server: RwLock<Option<Arc<Mutex<WindowServer>>>>,
}

#[derive(Debug)]
pub(super) struct PlatformStateImpl {
    proxy: EventLoopProxy,
    accessibility_enabled: AtomicBool,
}

impl PlatformStateImpl {
    pub(super) fn new(proxy: EventLoopProxy) -> Self {
        Self {
            proxy,
            accessibility_enabled: AtomicBool::new(accessibility_is_enabled()),
        }
    }

    //
    fn count_args(sel: Sel) -> usize {
        sel.name().chars().filter(|&c| c == ':').count()
    }

    fn method_type_encoding(ret: &Encoding, args: &[Encoding]) -> CString {
        let mut types = ret.as_str().to_owned();
        // First two arguments are always self and the selector
        types.push_str(<*mut Object>::encode().as_str());
        types.push_str(Sel::encode().as_str());
        types.extend(args.iter().map(|e| e.as_str()));
        CString::new(types).unwrap()
    }

    // Add an implementation for an ObjC selector, which will override default WKWebView & WryWebView
    // behavior
    fn override_webview_method<F>(sel: Sel, func: F)
    where
        F: MethodImplementation<Callee = Object>,
    {
        let encs = F::Args::encodings();
        let encs = encs.as_ref();
        let sel_args = Self::count_args(sel);
        assert!(
            sel_args == encs.len(),
            "Selector accepts {} arguments, but function accepts {}",
            sel_args,
            encs.len(),
        );

        let types = Self::method_type_encoding(&F::Ret::encode(), encs);

        // https://github.com/tauri-apps/wry/blob/17d324b70e4d580c43c9d4ab37bd265005356bf4/src/webview/wkwebview/mod.rs#L258
        let name = CString::new("WryWebView").unwrap();

        unsafe {
            let cls = objc_getClass(name.as_ptr()) as *mut Class;
            class_addMethod(cls, sel, func.imp(), types.as_ptr());
        }
    }

    pub(super) fn handle(
        self: &Arc<Self>,
        event: PlatformBoundEvent,
        window_target: &EventLoopWindowTarget,
        window_map: &FigIdMap,
    ) -> anyhow::Result<()> {
        match event {
            PlatformBoundEvent::Initialize => {
                UNMANAGED.event_sender.write().replace(self.proxy.clone());
                let (tx, rx) = flume::unbounded::<WindowServerEvent>();
                UNMANAGED
                    .window_server
                    .write()
                    .replace(unsafe { register_observer(tx) });

                let accessibility_proxy = self.proxy.clone();
                let mut distributed = NotificationCenter::distributed();
                let ax_notification_name: NSString = "com.apple.accessibility.api".into();
                distributed.subscribe(ax_notification_name, move |_, _| {
                    let enabled = accessibility_is_enabled();
                    accessibility_proxy
                        .clone()
                        .send_event(Event::PlatformBoundEvent(PlatformBoundEvent::AccessibilityUpdated {
                            enabled,
                        }))
                        .ok();
                    if enabled {
                        unsafe {
                            // This prevents Fig from becoming unresponsive if one of the applications
                            // we are tracking becomes unresponsive.
                            AXUIElementSetMessagingTimeout(AXUIElementCreateSystemWide(), 0.25);
                        }
                    }
                });

                let observer_proxy = self.proxy.clone();
                tokio::runtime::Handle::current().spawn(async move {
                    let update_fullscreen = |fullscreen: bool| {
                        Event::PlatformBoundEvent(PlatformBoundEvent::FullscreenStateUpdated { fullscreen })
                    };
                    while let std::result::Result::Ok(result) = rx.recv_async().await {
                        match result {
                            WindowServerEvent::FocusChanged { window, .. } => {
                                if let Err(e) = observer_proxy.send_event(Event::WindowEvent {
                                    window_id: AUTOCOMPLETE_ID,
                                    window_event: WindowEvent::Hide,
                                }) {
                                    warn!("Error sending event: {e:?}");
                                }
                                if let Some(window) = window {
                                    let is_fullscreen = unsafe { window.is_fullscreen().unwrap_or(false) };
                                    if let Err(e) = observer_proxy.send_event(update_fullscreen(is_fullscreen)) {
                                        warn!("Error sending event: {e:?}");
                                    }
                                }
                            },
                            WindowServerEvent::ActiveSpaceChanged { is_fullscreen } => {
                                if let Err(e) = observer_proxy.send_event(update_fullscreen(is_fullscreen)) {
                                    warn!("Error sending event: {e:?}");
                                }
                            },
                        }
                    }
                });

                fn to_s<'a>(nsstring_obj: *mut Object) -> Option<&'a str> {
                    const UTF8_ENCODING: libc::c_uint = 4;

                    let bytes = unsafe {
                        let length = msg_send![nsstring_obj, lengthOfBytesUsingEncoding: UTF8_ENCODING];
                        let utf8_str: *const u8 = msg_send![nsstring_obj, UTF8String];
                        slice::from_raw_parts(utf8_str, length)
                    };
                    std::str::from_utf8(bytes).ok()
                }

                extern "C" fn should_delay_window_ordering(this: &Object, _cmd: Sel, _event: id) -> BOOL {
                    debug!("should_delay_window_ordering");

                    unsafe {
                        let window: id = msg_send![this, window];
                        let title: id = msg_send![window, title];

                        // TODO: implement better method for determining if WebView belongs to autocomplete
                        if let Some(title) = to_s(title) {
                            if title == AUTOCOMPLETE_WINDOW_TITLE {
                                return YES;
                            }
                        }
                    }

                    NO
                }

                extern "C" fn mouse_down(this: &Object, _cmd: Sel, event: id) {
                    let application = Class::get("NSApplication").unwrap();

                    unsafe {
                        let window: id = msg_send![this, window];
                        let title: id = msg_send![window, title];

                        // TODO: implement better method for determining if WebView belongs to autocomplete
                        if let Some(title) = to_s(title) {
                            if title == AUTOCOMPLETE_WINDOW_TITLE {
                                // Prevent clicked window from taking focus
                                let app: id = msg_send![application, sharedApplication];
                                let _: () = msg_send![app, preventWindowOrdering];
                            }
                        }

                        // Invoke superclass implementation
                        let supercls = msg_send![this, superclass];
                        let _: () = msg_send![super(this, supercls), mouseDown: event];
                    }
                }

                extern "C" fn accepts_first_mouse(_this: &Object, _cmd: Sel, _event: id) -> BOOL {
                    debug!("accepts_first_mouse");
                    YES
                }

                // Use objc runtime to override WryWebview methods
                Self::override_webview_method(
                    sel!(shouldDelayWindowOrderingForEvent:),
                    should_delay_window_ordering as extern "C" fn(&Object, Sel, id) -> BOOL,
                );
                Self::override_webview_method(sel!(mouseDown:), mouse_down as extern "C" fn(&Object, Sel, id));
                Self::override_webview_method(
                    sel!(acceptsFirstMouse:),
                    accepts_first_mouse as extern "C" fn(&Object, Sel, id) -> BOOL,
                );

                Ok(())
            },
            PlatformBoundEvent::EditBufferChanged => {
                tracing::warn!("Sending notif io.fig.edit_buffer_updated");
                let current_terminal =
                    get_active_window().and_then(|window| Terminal::from_bundle_id(window.bundle_id.as_str()));

                let supports_ime = current_terminal
                    .map(|t| t.supports_macos_input_method())
                    .unwrap_or(false);

                // let supports_accessibility = current_terminal
                // .map(|t| t.supports_macos_accessibility())
                // .unwrap_or(false);

                if supports_ime {
                    NotificationCenter::distributed()
                        .post_notification("io.fig.edit_buffer_updated", std::iter::empty::<(&str, &str)>());
                } else {
                    let caret = self.get_cursor_position().context("Failed to get cursor position")?;
                    UNMANAGED
                        .event_sender
                        .read()
                        .clone()
                        .unwrap()
                        .send_event(Event::WindowEvent {
                            window_id: AUTOCOMPLETE_ID,
                            window_event: WindowEvent::PositionRelativeToCaret { caret },
                        })
                        .ok();
                }

                Ok(())
            },
            PlatformBoundEvent::FullscreenStateUpdated { fullscreen } => {
                let policy = if fullscreen {
                    ActivationPolicy::Accessory
                } else {
                    let mission_control_visible = window_map
                        .get(&DASHBOARD_ID)
                        .map(|window| window.webview.window().is_visible())
                        .unwrap_or(false);

                    if mission_control_visible {
                        ActivationPolicy::Regular
                    } else {
                        ActivationPolicy::Accessory
                    }
                };
                window_target.set_activation_policy_at_runtime(policy);
                Ok(())
            },
            PlatformBoundEvent::AccessibilityUpdated { enabled } => {
                let was_enabled = self.accessibility_enabled.swap(enabled, Ordering::Relaxed);
                if enabled && !was_enabled {
                    tokio::runtime::Handle::current().spawn(async move {
                        fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
                            fig_telemetry::TrackEventType::GrantedAXPermission,
                            fig_telemetry::TrackSource::Desktop,
                            env!("CARGO_PKG_VERSION").into(),
                            std::iter::empty::<(&str, &str)>(),
                        ))
                        .await
                        .ok();
                    });
                }

                if enabled != was_enabled {
                    self.proxy.send_event(Event::ReloadTray).ok();
                }

                Ok(())
            },
            PlatformBoundEvent::AppWindowFocusChanged {
                window_id,
                focused: _,
                fullscreen,
            } => {
                // Update activation policy
                if window_id == DASHBOARD_ID {
                    self.proxy
                        .send_event(Event::PlatformBoundEvent(PlatformBoundEvent::FullscreenStateUpdated {
                            fullscreen,
                        }))
                        .ok();
                }

                Ok(())
            },
        }
    }

    pub(super) fn position_window(
        &self,
        webview_window: &wry::application::window::Window,
        _window_id: &WindowId,
        position: Position,
    ) -> wry::Result<()> {
        webview_window.set_outer_position(position);
        std::result::Result::Ok(())
    }

    pub(super) fn get_cursor_position(&self) -> Option<Rect<i32, i32>> {
        let caret: CaretPosition = unsafe { get_caret_position(true) };

        if caret.valid {
            Some(Rect {
                x: caret.x as i32,
                y: caret.y as i32,
                width: DEFAULT_CARET_WIDTH,
                height: caret.height as i32,
            })
        } else {
            None
        }
    }

    pub(super) fn get_current_monitor_frame(
        &self,
        window: &wry::application::window::Window,
    ) -> Option<Rect<i32, i32>> {
        match window.current_monitor() {
            Some(monitor) => {
                let origin = monitor.position().to_logical(monitor.scale_factor()) as LogicalPosition<i32>;
                let size = monitor.size().to_logical(monitor.scale_factor()) as LogicalSize<i32>;

                Some(Rect {
                    x: origin.x,
                    y: origin.y,
                    width: size.width as i32,
                    height: size.height as i32,
                })
            },
            None => None,
        }
    }

    /// Gets the currently active window on the platform
    pub(super) fn get_active_window(&self) -> Option<PlatformWindow> {
        let window = get_active_window()?;
        let geometry = WindowGeometry {
            x: window.position.x as i32,
            y: window.position.y as i32,
            width: window.position.width as i32,
            height: window.position.height as i32,
        };

        Some(PlatformWindow { geometry })
    }

    pub(super) fn icon_lookup(asset: &AssetSpecifier) -> Option<ProcessedAsset> {
        match asset {
            AssetSpecifier::Named(_) => None,
            AssetSpecifier::PathBased(path) => {
                let data = unsafe { macos_accessibility_position::image::png_for_path(path)? };
                Some((data.into(), AssetKind::Png))
            },
        }
    }

    pub(super) fn shell() -> Cow<'static, str> {
        "/bin/bash".into()
    }

    pub fn accessibility_is_enabled(&self) -> Option<bool> {
        Some(self.accessibility_enabled.load(Ordering::Relaxed))
    }
}

pub const fn autocomplete_active() -> bool {
    true
}
