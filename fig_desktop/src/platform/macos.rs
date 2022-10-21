use std::borrow::Cow;
use std::ffi::CString;
use std::slice;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
use std::sync::Arc;

use accessibility_sys::{
    pid_t,
    AXError,
    AXUIElementCreateSystemWide,
    AXUIElementSetMessagingTimeout,
};
use anyhow::Context;
use cocoa::base::{
    id,
    NO,
    YES,
};
use core_graphics::display::CGRect;
use core_graphics::window::CGWindowID;
use fig_util::Terminal;
use macos_accessibility_position::accessibility::accessibility_is_enabled;
use macos_accessibility_position::caret_position::{
    get_caret_position,
    CaretPosition,
};
use macos_accessibility_position::window_server::{
    CGWindowLevelForKey,
    UIElement,
};
use macos_accessibility_position::{
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
    class,
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
    error,
    trace,
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
    WindowExtMacOS,
};

use super::{
    PlatformBoundEvent,
    PlatformWindow,
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

pub const DEFAULT_CARET_WIDTH: f64 = 10.0;

// See for other window level keys
// https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.8.sdk/System/Library/Frameworks/CoreGraphics.framework/Versions/A/Headers/CGWindowLevel.h
#[allow(non_upper_case_globals)]
const kCGFloatingWindowLevelKey: i32 = 5;

static UNMANAGED: Lazy<Unmanaged> = Lazy::new(|| Unmanaged {
    event_sender: RwLock::new(Option::<EventLoopProxy>::None),
    window_server: RwLock::new(Option::<Arc<Mutex<WindowServer>>>::None),
});

static ACCESSIBILITY_ENABLED: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(accessibility_is_enabled()));

struct Unmanaged {
    event_sender: RwLock<Option<EventLoopProxy>>,
    window_server: RwLock<Option<Arc<Mutex<WindowServer>>>>,
}

#[derive(Debug)]
pub(super) struct PlatformStateImpl {
    proxy: EventLoopProxy,
    focused_window: Mutex<Option<PlatformWindowImpl>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlatformWindowImpl {
    window_id: CGWindowID,
    ui_element: UIElement,
    x_term_tree_cache: Option<Vec<UIElement>>,
    pub bundle_id: String,
    pub pid: pid_t,
}

impl From<CGRect> for Rect {
    fn from(cgr: CGRect) -> Rect {
        Rect {
            position: LogicalPosition::new(cgr.origin.x, cgr.origin.y),
            size: LogicalSize::new(cgr.size.width, cgr.size.height),
        }
    }
}

impl PlatformWindowImpl {
    pub fn new(bundle_id: String, pid: pid_t, ui_element: UIElement) -> Result<Self, AXError> {
        let window_id = unsafe { ui_element.get_window_id()? };
        Ok(Self {
            window_id,
            ui_element,
            pid,
            x_term_tree_cache: None,
            bundle_id,
        })
    }

    pub fn get_window_id(&self) -> CGWindowID {
        self.window_id
    }

    pub fn bundle_id(&self) -> &str {
        self.bundle_id.as_str()
    }

    pub fn get_bounds(&self) -> Option<CGRect> {
        let info = self.ui_element.window_info()?;
        Some(info.bounds)
    }

    pub fn get_level(&self) -> Option<i64> {
        let info = self.ui_element.window_info()?;
        Some(info.level)
    }

    pub fn get_x_term_cursor_elem(&mut self) -> Option<UIElement> {
        let tree = self
            .x_term_tree_cache
            .as_ref()
            .and_then(|cached| {
                warn!("About to walk through {:?}", cached.len());
                let result: Option<Vec<UIElement>> =
                    cached.iter().fold(None::<Vec<UIElement>>, |accum, item| match accum {
                        Some(mut x) => {
                            x.push(item.clone());
                            Some(x)
                        },
                        None => item.find_x_term_caret_tree().ok(),
                    });
                result
            })
            .or_else(|| self.ui_element.find_x_term_caret_tree().ok());

        self.x_term_tree_cache = tree;
        self.x_term_tree_cache.as_ref()?.first().cloned()
    }
}

impl PlatformStateImpl {
    pub(super) fn new(proxy: EventLoopProxy) -> Self {
        let focused_window: Option<PlatformWindowImpl> = None;
        Self {
            proxy,
            focused_window: Mutex::new(focused_window),
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
        // https://github.com/tauri-apps/wry/blob/17d324b70e4d580c43c9d4ab37bd265005356bf4/src/webview/wkwebview/mod.rs#L258
        Self::override_objc_class_method("WryWebView", sel, func)
    }

    fn override_app_delegate_method<F>(sel: Sel, func: F)
    where
        F: MethodImplementation<Callee = Object>,
    {
        // https://github.com/tauri-apps/tao/blob/7c7ce8ab2d838a79ecdf83df00124c418a6a51f6/src/platform_impl/macos/app_delegate.rs#L35
        Self::override_objc_class_method("TaoAppDelegate", sel, func)
    }

    fn override_objc_class_method<F>(class: &str, sel: Sel, func: F)
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

        let name = CString::new(class).unwrap();

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
        warn!("Handling platform event: {:?}", event);
        match event {
            PlatformBoundEvent::Initialize => {
                UNMANAGED.event_sender.write().replace(self.proxy.clone());
                let (tx, rx) = flume::unbounded::<WindowServerEvent>();

                UNMANAGED
                    .window_server
                    .write()
                    .replace(Arc::new(Mutex::new(WindowServer::new(tx))));

                let accessibility_proxy = self.proxy.clone();
                let mut distributed = NotificationCenter::distributed();
                let ax_notification_name: NSString = "com.apple.accessibility.api".into();
                let queue: id = unsafe {
                    let queue: id = msg_send![class!(NSOperationQueue), alloc];
                    msg_send![queue, init]
                };
                distributed.subscribe(ax_notification_name, Some(queue), move |_| {
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
                    while let std::result::Result::Ok(result) = rx.recv_async().await {
                        let mut events: Vec<Event> = vec![];

                        match result {
                            WindowServerEvent::FocusChanged { window, app } => {
                                let fullscreen = window.is_fullscreen().unwrap_or(false);
                                events.push(Event::WindowEvent {
                                    window_id: AUTOCOMPLETE_ID,
                                    window_event: WindowEvent::Hide,
                                });

                                events.push(Event::PlatformBoundEvent(PlatformBoundEvent::FullscreenStateUpdated {
                                    fullscreen,
                                }));

                                if let Ok(window) = PlatformWindowImpl::new(app.bundle_id, app.pid, window) {
                                    events.push(Event::PlatformBoundEvent(
                                        PlatformBoundEvent::ExternalWindowFocusChanged { window },
                                    ));
                                }
                            },
                            WindowServerEvent::WindowDestroyed { window, app } => {
                                // TODO(sean) seems like this is failing -- can't get window id for
                                // the destroyed elements :(
                                if let Ok(window) = PlatformWindowImpl::new(app.bundle_id, app.pid, window) {
                                    events.push(Event::PlatformBoundEvent(PlatformBoundEvent::WindowDestroyed {
                                        window,
                                    }));
                                }
                            },
                            WindowServerEvent::ActiveSpaceChanged { is_fullscreen } => {
                                events.push(Event::PlatformBoundEvent(PlatformBoundEvent::FullscreenStateUpdated {
                                    fullscreen: is_fullscreen,
                                }));
                            },
                            WindowServerEvent::RequestCaretPositionUpdate => {
                                events.push(Event::PlatformBoundEvent(
                                    PlatformBoundEvent::CaretPositionUpdateRequested,
                                ));
                            },
                        };

                        for event in events {
                            if let Err(e) = observer_proxy.send_event(event) {
                                warn!("Error sending event: {e:?}");
                            }
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

                extern "C" fn perform_key_equivalent(this: &Object, _cmd: Sel, event: id) -> BOOL {
                    warn!("perform_key_equivalent");

                    unsafe {
                        // Allow super class to handle.
                        let supercls = msg_send![this, superclass];
                        let super_res: BOOL = msg_send![super(this, supercls), performKeyEquivalent: event];
                        if super_res == YES {
                            return YES;
                        }

                        // Handle common text manipulation like copy, paste, select all, etc
                        let app: id = msg_send![class!(NSApplication), sharedApplication];
                        let menu: id = msg_send![app, mainMenu];
                        let app_res: BOOL = msg_send![menu, performKeyEquivalent: event];
                        if app_res == YES {
                            return YES;
                        }

                        // Mark any unhandled events as handled to suppress beeps
                        YES
                    }
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
                Self::override_webview_method(
                    sel!(performKeyEquivalent:),
                    perform_key_equivalent as extern "C" fn(&Object, Sel, id) -> BOOL,
                );
                Self::override_webview_method(sel!(mouseDown:), mouse_down as extern "C" fn(&Object, Sel, id));
                Self::override_webview_method(
                    sel!(acceptsFirstMouse:),
                    accepts_first_mouse as extern "C" fn(&Object, Sel, id) -> BOOL,
                );

                extern "C" fn application_should_handle_reopen(
                    _this: &Object,
                    _cmd: Sel,
                    _sender: id,
                    _visible_windows: BOOL,
                ) -> BOOL {
                    trace!("application_should_handle_reopen");
                    NotificationCenter::shared()
                        .post_notification("io.fig.show-dashboard", std::iter::empty::<(&str, &str)>());
                    YES
                }

                let queue: id = unsafe {
                    let queue: id = msg_send![class!(NSOperationQueue), alloc];
                    msg_send![queue, init]
                };
                let application_observer = self.proxy.clone();
                NotificationCenter::shared().subscribe("io.fig.show-dashboard", Some(queue), move |_| {
                    if let Err(e) = application_observer.send_event(Event::WindowEvent {
                        window_id: DASHBOARD_ID,
                        window_event: WindowEvent::Show,
                    }) {
                        warn!("Error sending event: {e:?}");
                    }
                });

                Self::override_app_delegate_method(
                    sel!(applicationShouldHandleReopen:hasVisibleWindows:),
                    application_should_handle_reopen as extern "C" fn(&Object, Sel, id, BOOL) -> BOOL,
                );

                Ok(())
            },
            PlatformBoundEvent::EditBufferChanged => {
                if let Err(e) = self.refresh_window_position() {
                    error!("Failed to refresh window position: {e:?}");
                }
                Ok(())
            },
            PlatformBoundEvent::ExternalWindowFocusChanged { window } => {
                let mut focused = self.focused_window.lock();
                let level = window.get_level();
                focused.replace(window);

                if let Some(window) = window_map.get(&AUTOCOMPLETE_ID) {
                    let ns_window = window.webview.window().ns_window() as *mut Object;
                    // Handle iTerm Quake mode by explicitly setting window level. See
                    // https://github.com/gnachman/iTerm2/blob/1a5a09f02c62afcc70a647603245e98862e51911/sources/iTermProfileHotKey.m#L276-L310
                    // for more on window levels.
                    let above = match level {
                        None | Some(0) => unsafe { CGWindowLevelForKey(kCGFloatingWindowLevelKey) as i64 },
                        Some(level) => level,
                    };
                    debug!("Setting window level to {level:?}");
                    let _: () = unsafe { msg_send![ns_window, setLevel: above] };
                }

                Ok(())
            },
            PlatformBoundEvent::CaretPositionUpdateRequested => {
                if let Err(e) = self.refresh_window_position() {
                    error!("Failed to refresh window position: {e:?}");
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
                let was_enabled = ACCESSIBILITY_ENABLED.swap(enabled, Ordering::SeqCst);
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

                self.proxy.send_event(Event::ReloadAccessibility).ok();

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
            PlatformBoundEvent::WindowDestroyed { window } => {
                let mut focused = self.focused_window.lock();
                if let Some(focused_window) = focused.as_ref() {
                    if focused_window.get_window_id() == window.get_window_id() {
                        focused.take();
                        self.proxy
                            .send_event(Event::WindowEvent {
                                window_id: AUTOCOMPLETE_ID,
                                window_event: WindowEvent::Hide,
                            })
                            .ok();
                    }
                }
                Ok(())
            },
        }
    }

    fn refresh_window_position(&self) -> anyhow::Result<()> {
        let mut guard = self.focused_window.lock();
        let active_window = guard.as_mut().context("No active window")?;
        let current_terminal = Terminal::from_bundle_id(active_window.bundle_id());

        let supports_ime = current_terminal
            .clone()
            .map(|t| t.supports_macos_input_method())
            .unwrap_or(false);

        let is_xterm = current_terminal.map(|t| t.is_xterm()).unwrap_or(false);

        // let supports_accessibility = current_terminal
        // .map(|t| t.supports_macos_accessibility())
        // .unwrap_or(false);

        if !is_xterm && supports_ime {
            tracing::warn!("Sending notif io.fig.edit_buffer_updated");
            NotificationCenter::distributed()
                .post_notification("io.fig.edit_buffer_updated", std::iter::empty::<(&str, &str)>());
        } else {
            let caret = if is_xterm {
                let cursor = active_window.get_x_term_cursor_elem();
                /*
                for i in 0..20 {
                    // std::thread::sleep(std::time::Duration::from_millis(20));
                    cursor = active_window.get_x_term_cursor();
                    // println!("Iter {i}, {cursor:?}");
                }
                */

                cursor.and_then(|c| c.frame().ok()).map(Rect::from)
            } else {
                None
            };

            let caret = caret
                .or_else(|| self.get_cursor_position())
                .context("Failed to get cursor position")?;
            warn!("Sending caret update {:?}", caret);

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

    pub(super) fn get_cursor_position(&self) -> Option<Rect> {
        let caret: CaretPosition = unsafe { get_caret_position(true) };

        if caret.valid {
            Some(Rect {
                position: LogicalPosition::new(caret.x, caret.y),
                size: LogicalSize::new(DEFAULT_CARET_WIDTH, caret.height),
            })
        } else {
            None
        }
    }

    /// Gets the currently active window on the platform
    pub(super) fn get_active_window(&self) -> Option<PlatformWindow> {
        let active_window = self.focused_window.lock().as_ref()?.clone();
        Some(PlatformWindow {
            rect: active_window.get_bounds()?.into(),
            inner: active_window,
        })
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

    pub(super) fn accessibility_is_enabled() -> Option<bool> {
        Some(ACCESSIBILITY_ENABLED.load(Ordering::SeqCst))
    }
}

pub const fn autocomplete_active() -> bool {
    true
}
