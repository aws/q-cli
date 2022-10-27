use std::borrow::Cow;
use std::fmt::{
    self,
    Display,
};
use std::sync::atomic::AtomicBool;

use bytes::BytesMut;
use fig_proto::fig::notification::Type as NotificationEnum;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    EventNotification,
    Notification,
    NotificationType,
    ServerOriginatedMessage,
};
use fig_proto::prost::Message;
use parking_lot::RwLock;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{
    error,
    warn,
};
use wry::application::dpi::{
    LogicalPosition,
    LogicalSize,
    PhysicalSize,
    Position,
};
use wry::application::window::Theme;
use wry::webview::{
    WebContext,
    WebView,
};

use super::notification::WebviewNotificationsState;
use crate::event::{
    ClippingBehavior,
    EmitEventName,
    Placement,
    RelativeDirection,
    WindowEvent,
};
use crate::figterm::{
    FigtermCommand,
    FigtermState,
};
use crate::platform::{
    self,
    PlatformState,
};
use crate::{
    EventLoopWindowTarget,
    AUTOCOMPLETE_ID,
    DASHBOARD_ID,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowId(pub Cow<'static, str>);

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// TODO: Add state for the active terminal window
// TODO: Switch to using LogicalPosition and LogicalSize
pub struct WindowState {
    pub webview: WebView,
    pub context: WebContext,
    pub window_id: WindowId,
    pub anchor: RwLock<LogicalPosition<f64>>,
    pub outer_position: RwLock<LogicalPosition<f64>>,
    pub inner_size: RwLock<LogicalSize<f64>>,
    pub placement: RwLock<Placement>,
    pub enabled: AtomicBool,
}

impl WindowState {
    pub fn new(window_id: WindowId, webview: WebView, context: WebContext, enabled: bool) -> Self {
        let window = webview.window();
        let scale_factor = window.scale_factor();

        let position = webview
            .window()
            .outer_position()
            .expect("Failed to acquire window position")
            .to_logical(scale_factor);

        let size = window.inner_size().to_logical(scale_factor);

        Self {
            webview,
            context,
            window_id,
            anchor: RwLock::new(LogicalPosition::default()),
            outer_position: RwLock::new(position),
            inner_size: RwLock::new(size),
            placement: RwLock::new(Placement::Absolute),
            enabled: AtomicBool::new(enabled),
        }
    }

    fn update_position(&self, platform_state: &PlatformState) {
        let anchor = *self.anchor.read();
        let outer_position = *self.outer_position.read();
        let inner_size = *self.inner_size.read();
        let placement = *self.placement.read();

        // TODO: this should be handled directly by client apps (eg. autocomplete engine) rather than being
        // hardcoded
        let vertical_padding = anchor.y + 5.0;

        let monitor_frame = platform_state.get_current_monitor_frame(self.webview.window());

        let x = match placement {
            Placement::Absolute => outer_position.x,
            Placement::RelativeTo(caret, RelativeDirection::Above | RelativeDirection::Below, clipping_behavior) => {
                match (clipping_behavior, monitor_frame) {
                    (ClippingBehavior::Allow, _) | (ClippingBehavior::KeepInFrame, None) => caret.left() + anchor.x,
                    (ClippingBehavior::KeepInFrame, Some(frame)) => {
                        let origin_x = caret.left() + anchor.x;
                        let offset_x = frame.right() - (caret.left() + inner_size.width + anchor.x);
                        if offset_x < 0.0 { origin_x + offset_x } else { origin_x }
                    },
                }
            },
        };

        let y = match placement {
            Placement::Absolute => outer_position.y,
            Placement::RelativeTo(caret, RelativeDirection::Above, _) => {
                caret.top() - vertical_padding - inner_size.height
            },
            Placement::RelativeTo(caret, RelativeDirection::Below, _) => caret.bottom() + vertical_padding,
        };

        if let Err(err) = platform_state.position_window(
            self.webview.window(),
            &self.window_id,
            Position::Logical(LogicalPosition { x, y }),
        ) {
            tracing::error!(%err, window_id =% self.window_id, "Failed to position window");
        }
    }

    pub fn handle(
        &self,
        event: WindowEvent,
        figterm_state: &FigtermState,
        platform_state: &PlatformState,
        notifications_state: &WebviewNotificationsState,
        window_target: &EventLoopWindowTarget,
        api_tx: &UnboundedSender<(WindowId, String)>,
    ) {
        match event {
            WindowEvent::Reanchor { position } => {
                *self.anchor.write() = position;
                self.update_position(platform_state);
            },
            WindowEvent::PositionAbsolute { position } => {
                *self.placement.write() = Placement::Absolute;
                *self.outer_position.write() = position;
                self.update_position(platform_state);
            },
            WindowEvent::PositionRelativeToCaret { caret } => {
                let max_height = fig_settings::settings::get_int_or("autocomplete.height", 140) as f64;

                // TODO: these calculations do not take into account anchor offset (or default vertical padding)
                let overflows_monitor_above = platform_state
                    .get_current_monitor_frame(self.webview.window())
                    .map(|monitor| monitor.top() >= caret.top() - max_height)
                    .unwrap_or(true);

                let overflows_window_below = platform_state
                    .get_active_window()
                    .map(|window| window.rect.bottom() < caret.bottom() + max_height)
                    .unwrap_or(true);

                *self.placement.write() = Placement::RelativeTo(
                    caret,
                    if overflows_window_below && !overflows_monitor_above {
                        RelativeDirection::Above
                    } else {
                        RelativeDirection::Below
                    },
                    ClippingBehavior::KeepInFrame,
                );
                self.update_position(platform_state);
            },
            WindowEvent::PositionRelativeToRect {
                rect,
                direction,
                clipping_behavior,
            } => {
                *self.placement.write() = Placement::RelativeTo(rect, direction, clipping_behavior);
                self.update_position(platform_state);
            },
            WindowEvent::Resize { size } => {
                *self.inner_size.write() = size;
                self.update_position(platform_state);
                cfg_if::cfg_if! {
                    if #[cfg(target_os = "linux")] {
                        if self.window_id == AUTOCOMPLETE_ID {
                            self.webview
                                .window()
                                .set_min_inner_size(Some(size));
                        } else {
                            self.webview.window().set_inner_size(size);
                        }
                    } else {
                        self.webview.window().set_inner_size(size);
                    }
                }
            },
            WindowEvent::Hide => {
                if !self.webview.window().is_visible() {
                    return;
                }
                self.webview.window().set_visible(false);

                if self.window_id == AUTOCOMPLETE_ID {
                    for session in figterm_state.linked_sessions.lock().iter() {
                        let _ = session.sender.send(FigtermCommand::InterceptClear);
                    }

                    #[cfg(not(target_os = "linux"))]
                    self.webview.window().set_resizable(true);
                    #[cfg(target_os = "linux")]
                    self.webview
                        .window()
                        .set_min_inner_size(Some(PhysicalSize { width: 1, height: 1 }));

                    self.webview
                        .window()
                        .set_inner_size(PhysicalSize { width: 1, height: 1 });

                    #[cfg(not(target_os = "linux"))]
                    self.webview.window().set_resizable(false);
                }

                #[cfg(target_os = "macos")]
                if self.window_id == DASHBOARD_ID {
                    use wry::application::platform::macos::{
                        ActivationPolicy,
                        EventLoopWindowTargetExtMacOS,
                    };
                    window_target.set_activation_policy_at_runtime(ActivationPolicy::Accessory);
                }
            },
            WindowEvent::Show => {
                if self.window_id == AUTOCOMPLETE_ID {
                    if platform::autocomplete_active() {
                        self.webview.window().set_visible(true);
                        cfg_if::cfg_if!(
                            if #[cfg(target_os = "macos")] {
                                // We handle setting window level on focus changed on MacOS
                                // TODO(sean) pull this out into platform code.
                            } else if #[cfg(target_os = "windows")] {
                                self.webview.window().set_always_on_top(false);
                            } else {
                                self.webview.window().set_always_on_top(true);
                            }
                        );
                    }
                } else {
                    #[cfg(target_os = "macos")]
                    if self.window_id == DASHBOARD_ID {
                        use wry::application::platform::macos::{
                            ActivationPolicy,
                            EventLoopWindowTargetExtMacOS,
                        };
                        window_target.set_activation_policy_at_runtime(ActivationPolicy::Regular);
                    }

                    self.webview.window().set_visible(true);
                    self.webview.window().set_focus();
                }
            },
            WindowEvent::NavigateAbsolute { url } => {
                self.webview
                    .evaluate_script(&format!("window.location.href = '{url}';"))
                    .unwrap();
            },
            WindowEvent::NavigateRelative { path } => {
                let event_name = "mission-control.navigate";
                let payload = serde_json::json!({ "path": path });

                self.notification(notifications_state, &NotificationType::NotifyOnEvent, Notification {
                    r#type: Some(NotificationEnum::EventNotification(EventNotification {
                        event_name: Some(event_name.to_string()),
                        payload: Some(payload.to_string()),
                    })),
                })
            },
            WindowEvent::Emit { event_name, payload } => {
                self.emit(event_name, payload);
            },
            WindowEvent::Api { payload } => {
                api_tx.send((self.window_id.clone(), payload)).unwrap();
            },
            WindowEvent::Devtools => {
                if self.webview.is_devtools_open() {
                    self.webview.close_devtools();
                } else {
                    self.webview.open_devtools();
                }
            },
            WindowEvent::DebugMode(debug_mode) => {
                // Macos does not support setting the webview background color so we have
                // to set the css background root color to see the window
                cfg_if::cfg_if! {
                    if #[cfg(target_os = "macos")] {
                        self.webview
                            .evaluate_script(if debug_mode {
                                "document.querySelector(':root').style.setProperty('background-color', 'red');"
                            } else {
                                "document.querySelector(':root').style.removeProperty('background-color');"
                            })
                            .unwrap();
                    } else {
                        self.webview
                            .set_background_color(if debug_mode {
                                (0xff, 0, 0, 0xff)
                            } else {
                                (0, 0, 0, 0) }
                            ).unwrap();
                    }

                }
            },
            WindowEvent::SetEnabled(enabled) => self.set_enabled(enabled),
            WindowEvent::SetTheme(theme) => self.set_theme(theme),
            WindowEvent::Center => {
                let window = self.webview.window();
                if let Some(display) = platform_state.get_current_monitor_frame(window) {
                    let outer_size = window.outer_size().to_logical::<f64>(window.scale_factor());
                    *self.placement.write() = Placement::Absolute;
                    *self.outer_position.write() = LogicalPosition::new(
                        display.center() - outer_size.width * 0.5,
                        display.middle() - outer_size.height * 0.5,
                    );
                    self.update_position(platform_state);
                }
            },
        }
    }

    pub fn emit(&self, event_name: impl Display, payload: impl Into<serde_json::Value>) {
        let payload = payload.into();
        self.webview
            .evaluate_script(&format!(
                "document.dispatchEvent(new CustomEvent('{event_name}', {{'detail': {payload}}}));"
            ))
            .unwrap();
    }

    pub fn notification(
        &self,
        notifications_state: &WebviewNotificationsState,
        notification_type: &NotificationType,
        notification: Notification,
    ) {
        let window_id = &self.window_id;
        if let Some(notifications) = notifications_state.subscriptions.get(window_id) {
            if let Some(message_id) = notifications.get(notification_type) {
                let message = ServerOriginatedMessage {
                    id: Some(*message_id),
                    submessage: Some(ServerOriginatedSubMessage::Notification(notification)),
                };

                let mut encoded = BytesMut::new();

                match message.encode(&mut encoded) {
                    Ok(_) => self.emit(EmitEventName::ProtoMessageReceived, base64::encode(encoded)),
                    Err(err) => error!(%err, "Failed to encode notification"),
                }
            } else {
                warn!(?notification_type, %window_id, "No subscription for notification type");
            }
        } else {
            warn!(?notification_type, %window_id, "No subscriptions for window");
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.webview
            .evaluate_script(format!("document.fig.enabled = {enabled};").as_str())
            .unwrap();
        self.enabled.store(enabled, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set_theme(&self, _theme: Option<Theme>) {
        // TODO: blocked on https://github.com/tauri-apps/tao/issues/582
    }
}
