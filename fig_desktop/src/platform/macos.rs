use std::borrow::Cow;
use std::ffi::CString;
use std::slice;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
use std::sync::Arc;

use accessibility_sys::{
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
use core_graphics::window::CGWindowID;
use fig_util::Terminal;
use macos_accessibility_position::accessibility::accessibility_is_enabled;
use macos_accessibility_position::caret_position::{
    get_caret_position,
    CaretPosition,
};
use macos_accessibility_position::window_server::UIElement;
use macos_accessibility_position::{
    get_active_window,
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

#[derive(Debug)]
#[allow(dead_code)]
pub struct PlatformWindowImpl {
    window_id: CGWindowID,
}

impl PlatformWindowImpl {
    pub fn from_window_id(window_id: CGWindowID) -> Self {
        Self { window_id }
    }

    pub fn from_ui_element(ui_element: UIElement) -> Result<Self, AXError> {
        let window_id = unsafe { ui_element.get_window_id()? };
        Ok(Self { window_id })
    }

    pub fn get_window_id(&self) -> CGWindowID {
        self.window_id
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
                distributed.subscribe(ax_notification_name, Some(queue), move |_, _| {
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
                            WindowServerEvent::FocusChanged { window } => {
                                let fullscreen = unsafe { window.is_fullscreen().unwrap_or(false) };
                                events.push(Event::WindowEvent {
                                    window_id: AUTOCOMPLETE_ID,
                                    window_event: WindowEvent::Hide,
                                });

                                events.push(Event::PlatformBoundEvent(PlatformBoundEvent::FullscreenStateUpdated {
                                    fullscreen,
                                }));

                                if let Ok(window) = PlatformWindowImpl::from_ui_element(window) {
                                    events.push(Event::PlatformBoundEvent(
                                        PlatformBoundEvent::ExternalWindowFocusChanged { window },
                                    ));
                                }
                            },
                            WindowServerEvent::WindowDestroyed { window } => {
                                // TODO(sean) seems like this is failing -- can't get window id for
                                // the destroyed elements :(
                                if let Ok(window) = PlatformWindowImpl::from_ui_element(window) {
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

                let application_observer = self.proxy.clone();
                NotificationCenter::shared().subscribe("io.fig.show-dashboard", None, move |_, _| {
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
                focused.replace(window);
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

                if enabled != was_enabled {
                    self.proxy.send_event(Event::ReloadAccessibility).ok();
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
        let current_terminal =
            get_active_window().and_then(|window| Terminal::from_bundle_id(window.bundle_id.as_str()));

        let supports_ime = current_terminal
            .map(|t| t.supports_macos_input_method())
            .unwrap_or(false);

        // let supports_accessibility = current_terminal
        // .map(|t| t.supports_macos_accessibility())
        // .unwrap_or(false);

        if supports_ime {
            tracing::warn!("Sending notif io.fig.edit_buffer_updated");
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
        let window = get_active_window()?;
        Some(PlatformWindow {
            rect: Rect {
                position: LogicalPosition::new(window.position.x, window.position.y),
                size: LogicalSize::new(window.position.width, window.position.height),
            },
            inner: PlatformWindowImpl::from_window_id(window.window_id),
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
