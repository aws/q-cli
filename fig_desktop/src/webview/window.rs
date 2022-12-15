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
use parking_lot::Mutex;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{
    error,
    info,
    warn,
};
use url::Url;
#[cfg(target_os = "linux")]
use wry::application::dpi::PhysicalSize;
use wry::application::dpi::{
    LogicalPosition,
    LogicalSize,
    Position,
};
use wry::application::window::Theme;
use wry::webview::{
    WebContext,
    WebView,
};

use super::notification::WebviewNotificationsState;
use crate::event::{
    EmitEventName,
    WindowEvent,
    WindowPosition,
};
use crate::figterm::{
    FigtermCommand,
    FigtermState,
};
use crate::platform::{
    self,
    PlatformState,
};
use crate::utils::Rect;
#[cfg(target_os = "macos")]
use crate::DASHBOARD_ID;
use crate::{
    EventLoopWindowTarget,
    AUTOCOMPLETE_ID,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowId(pub Cow<'static, str>);

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl serde::Serialize for WindowId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

pub struct WindowGeometryState {
    /// The outer position of the window by positioning scheme
    pub position: WindowPosition,
    /// The inner size of the window
    pub size: LogicalSize<f64>,
    /// The window anchor, which is added onto the final position
    pub anchor: LogicalSize<f64>,
}

// TODO: Add state for the active terminal window
pub struct WindowState {
    pub webview: WebView,
    pub context: WebContext,
    pub window_id: WindowId,
    pub window_geometry_state: Mutex<WindowGeometryState>,
    pub enabled: AtomicBool,
    pub url: Mutex<Url>,
}

impl WindowState {
    pub fn new(window_id: WindowId, webview: WebView, context: WebContext, enabled: bool, url: Url) -> Self {
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
            window_geometry_state: Mutex::new(WindowGeometryState {
                position: WindowPosition::Absolute(Position::Logical(position)),
                size,
                anchor: LogicalSize::<f64>::default(),
            }),
            enabled: enabled.into(),
            url: Mutex::new(url),
        }
    }

    fn update_position(
        &self,
        position: Option<WindowPosition>,
        size: Option<LogicalSize<f64>>,
        anchor: Option<LogicalSize<f64>>,
        platform_state: &PlatformState,
        dry_run: bool,
    ) -> (bool, bool) {
        // Lock our atomic state
        let mut state = self.window_geometry_state.lock();

        // Acquire our position, size, and anchor, and update them if dirty
        let position = match position {
            Some(position) if !dry_run => {
                state.position = position;
                position
            },
            Some(position) => position,
            None => state.position,
        };

        let size = match size {
            Some(size) if !dry_run => {
                state.size = size;
                size
            },
            _ => state.size,
        };

        let anchor = match anchor {
            Some(anchor) if !dry_run => {
                state.anchor = anchor;
                anchor
            },
            _ => state.anchor,
        };

        let window = self.webview.window();
        let monitor_state = match position {
            WindowPosition::Absolute(_) | WindowPosition::Centered => {
                let scale_factor = window.scale_factor();
                window.current_monitor().map(|monitor| {
                    let monitor_position: LogicalPosition<f64> = monitor.position().to_logical(scale_factor);
                    let monitor_size: LogicalSize<f64> = monitor.size().to_logical(scale_factor);
                    (monitor, monitor_position, monitor_size, scale_factor)
                })
            },
            WindowPosition::RelativeToCaret { caret_position, .. } => window
                .available_monitors()
                .find(|monitor| {
                    let scale_factor = monitor.scale_factor();
                    let monitor_frame = Rect {
                        position: monitor.position().into(),
                        size: monitor.size().into(),
                    };
                    monitor_frame.contains(caret_position, scale_factor)
                })
                .map(|monitor| {
                    let scale_factor = monitor.scale_factor();
                    let monitor_position: LogicalPosition<f64> = monitor.position().to_logical(scale_factor);
                    let monitor_size: LogicalSize<f64> = monitor.size().to_logical(scale_factor);
                    (monitor, monitor_position, monitor_size, scale_factor)
                }),
        };

        let (position, is_above, is_clipped) = match position {
            WindowPosition::Absolute(position) => (position, false, false),
            WindowPosition::Centered => match monitor_state {
                Some((_, monitor_position, monitor_size, _scale_factor)) => (
                    Position::Logical(LogicalPosition::new(
                        monitor_position.x + monitor_size.width * 0.5 - size.width * 0.5,
                        monitor_position.y + monitor_size.height * 0.5 - size.height * 0.5,
                    )),
                    false,
                    false,
                ),
                None => return (false, false),
            },
            WindowPosition::RelativeToCaret {
                caret_position,
                caret_size,
            } => {
                let max_height = fig_settings::settings::get_int_or("autocomplete.height", 140) as f64;

                let (caret_position, overflows_monitor_above, overflows_monitor_below, scale_factor) =
                    match &monitor_state {
                        Some((_, monitor_position, monitor_size, scale_factor)) => {
                            let logical_caret_position = caret_position.to_logical::<f64>(*scale_factor);
                            (
                                logical_caret_position,
                                monitor_position.y >= logical_caret_position.y - max_height,
                                monitor_position.y + monitor_size.height
                                    < logical_caret_position.y
                                        + caret_size.to_logical::<f64>(*scale_factor).height
                                        + max_height,
                                *scale_factor,
                            )
                        },
                        None => (caret_position.to_logical(1.0), false, false, 1.0),
                    };

                let caret_size = caret_size.to_logical::<f64>(scale_factor);

                let overflows_window_below = platform_state
                    .get_active_window()
                    .map(|window| window.rect.bottom(scale_factor) < max_height + caret_position.y + caret_size.height)
                    .unwrap_or(false);

                let above = !overflows_monitor_above & (overflows_monitor_below | overflows_window_below);

                let mut x: f64 = caret_position.x + anchor.width;
                let mut y: f64 = match above {
                    true => caret_position.y - size.height - anchor.height,
                    false => caret_position.y + caret_size.height + anchor.height,
                };

                #[allow(clippy::all)]
                let clipped = if let Some((_, monitor_position, monitor_size, _)) = &monitor_state {
                    let clipped = caret_position.x + size.width > monitor_position.x + monitor_size.width;

                    x = x
                        .min(monitor_position.x + monitor_size.width - size.width)
                        .max(monitor_position.x);
                    y = y
                        .min(monitor_position.y + monitor_size.height - size.height)
                        .max(monitor_position.y);

                    clipped
                } else {
                    false
                };

                (Position::Logical(LogicalPosition::new(x, y)), above, clipped)
            },
        };

        if !dry_run {
            match platform_state.position_window(self.webview.window(), &self.window_id, position) {
                Ok(_) => {
                    tracing::trace!(window_id =% self.window_id, ?position, ?size, ?anchor, "updated window geometry")
                },
                Err(err) => tracing::error!(%err, window_id =% self.window_id, "failed to position window"),
            }

            // Apply the diff to atomic state
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

            match platform_state.position_window(self.webview.window(), &self.window_id, position) {
                Ok(_) => {
                    tracing::trace!(window_id =% self.window_id, ?position, ?size, ?anchor, "updated window geometry")
                },
                Err(err) => tracing::error!(%err, window_id =% self.window_id, "failed to position window"),
            }
        }

        (is_above, is_clipped)
    }

    #[allow(clippy::only_used_in_recursion)]
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
            WindowEvent::UpdateWindowGeometry {
                position,
                size,
                anchor,
                dry_run,
                tx,
            } => {
                let (is_above, is_clipped) = self.update_position(position, size, anchor, platform_state, dry_run);
                if let Some(tx) = tx {
                    if let Err(err) = tx.send((is_above, is_clipped)) {
                        tracing::error!(%err, "failed to send window geometry update result");
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
                        let _ = session
                            .sender
                            .send(FigtermCommand::InterceptFigJSVisible { visible: false });
                    }

                    #[cfg(not(target_os = "linux"))]
                    self.webview.window().set_resizable(true);

                    // TODO: move to only happen when size is set to 1x1 by ae
                    #[cfg(target_os = "linux")]
                    self.webview
                        .window()
                        .set_min_inner_size(Some(PhysicalSize { width: 1, height: 1 }));

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
                        for session in figterm_state.linked_sessions.lock().iter_mut() {
                            let _ = session
                                .sender
                                .send(FigtermCommand::InterceptFigJSVisible { visible: true });
                        }

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
                *self.url.lock() = url;
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
            WindowEvent::NavigateForward => {
                self.webview.evaluate_script("window.history.forward();").unwrap();
            },
            WindowEvent::NavigateBack => {
                self.webview.evaluate_script("window.history.back();").unwrap();
            },
            WindowEvent::ReloadIfNotLoaded => {
                info!(%self.window_id, "Reloading window if not loaded");

                let url = serde_json::json!(self.url.lock().clone());

                self.webview
                    .evaluate_script(&format!(
                        "if (window.location.href === 'about:blank') {{\
                            console.log('Reloading window to', {url});\
                            window.location.href = {url};\
                        }}"
                    ))
                    .unwrap();
            },
            WindowEvent::Reload => {
                info!(%self.window_id, "Reloading window");

                let url = serde_json::json!(self.url.lock().clone());

                self.webview
                    .evaluate_script(&format!(
                        "if (window.location.href === 'about:blank') {{\
                            console.log('Reloading window to', {url});\
                            window.location.href = {url};\
                        }} else {{\
                            console.log('Reloading window');\
                            window.location.reload();\
                        }}"
                    ))
                    .unwrap();
            },
            WindowEvent::Emit { event_name, payload } => {
                self.emit(event_name, payload);
            },
            WindowEvent::Api { payload } => {
                api_tx.send((self.window_id.clone(), payload.into())).unwrap();
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
            WindowEvent::SetHtml { html } => {
                self.webview
                    .evaluate_script(&format!("document.documentElement.innerHTML = `{html}`;"))
                    .unwrap();
            },
            WindowEvent::Batch(events) => {
                for event in events {
                    self.handle(
                        event,
                        figterm_state,
                        platform_state,
                        notifications_state,
                        window_target,
                        api_tx,
                    );
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
            .evaluate_script(format!("window.fig.enabled = {enabled};").as_str())
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
