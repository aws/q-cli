pub mod menu;
pub mod notification;
pub mod window;

use std::borrow::Cow;
use std::iter::empty;
use std::sync::Arc;

use cfg_if::cfg_if;
use dashmap::DashMap;
use fig_desktop_api::init_script::javascript_init;
use fig_proto::fig::client_originated_message::Submessage;
use fig_proto::fig::ClientOriginatedMessage;
use fig_request::auth::is_logged_in;
use fig_util::directories;
use fnv::FnvBuildHasher;
use once_cell::sync::Lazy;
use regex::RegexSet;
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};
use url::Url;
use window::{
    WindowId,
    WindowState,
};
use wry::application::dpi::LogicalSize;
use wry::application::event::{
    Event as WryEvent,
    StartCause,
    WindowEvent as WryWindowEvent,
};
use wry::application::event_loop::{
    ControlFlow,
    EventLoop as WryEventLoop,
};
use wry::application::menu::MenuType;
use wry::application::window::{
    Theme,
    WindowBuilder,
    WindowId as WryWindowId,
};
use wry::webview::{
    WebContext,
    WebView,
    WebViewBuilder,
};

use self::notification::WebviewNotificationsState;
use crate::event::{
    Event,
    WindowEvent,
};
use crate::figterm::FigtermState;
use crate::notification_bus::{
    JsonNotification,
    NOTIFICATION_BUS,
};
use crate::platform::{
    PlatformBoundEvent,
    PlatformState,
};
use crate::request::api_request;
use crate::tray::{
    self,
    build_tray,
    get_context_menu,
};
use crate::{
    file_watcher,
    icons,
    local_ipc,
    secure_ipc,
    utils,
    DebugState,
    EventLoop,
    EventLoopProxy,
    InterceptState,
};

pub const FIG_PROTO_MESSAGE_RECEIVED: &str = "FigProtoMessageRecieved";

pub const DASHBOARD_ID: WindowId = WindowId(Cow::Borrowed("mission-control"));
pub const AUTOCOMPLETE_ID: WindowId = WindowId(Cow::Borrowed("autocomplete"));

pub const DASHBOARD_ONBOARDING_SIZE: LogicalSize<f64> = LogicalSize::new(590.0, 480.0);
pub const DASHBOARD_INITIAL_SIZE: LogicalSize<f64> = LogicalSize::new(1030.0, 720.0);
pub const DASHBOARD_MINIMUM_SIZE: LogicalSize<f64> = LogicalSize::new(700.0, 480.0);

pub const AUTOCOMPLETE_WINDOW_TITLE: &str = "Fig Autocomplete";

pub const ONBOARDING_PATH: &str = "/onboarding/welcome";

fn map_theme(theme: &str) -> Option<Theme> {
    match theme {
        "dark" => Some(Theme::Dark),
        "light" => Some(Theme::Light),
        _ => None,
    }
}

pub static THEME: Lazy<Option<Theme>> = Lazy::new(|| {
    fig_settings::settings::get_string("app.theme")
        .ok()
        .flatten()
        .as_deref()
        .and_then(map_theme)
});

pub type FigIdMap = DashMap<WindowId, Arc<WindowState>, FnvBuildHasher>;
pub type WryIdMap = DashMap<WryWindowId, Arc<WindowState>, FnvBuildHasher>;

pub struct WebviewManager {
    fig_id_map: Arc<FigIdMap>,
    window_id_map: Arc<WryIdMap>,
    event_loop: EventLoop,
    debug_state: Arc<DebugState>,
    figterm_state: Arc<FigtermState>,
    intercept_state: Arc<InterceptState>,
    platform_state: Arc<PlatformState>,
    notifications_state: Arc<WebviewNotificationsState>,
}

impl Default for WebviewManager {
    fn default() -> Self {
        let event_loop = WryEventLoop::with_user_event();
        let proxy = event_loop.create_proxy();

        Self {
            fig_id_map: Default::default(),
            window_id_map: Default::default(),
            event_loop,
            debug_state: Arc::new(DebugState::default()),
            figterm_state: Arc::new(FigtermState::default()),
            intercept_state: Arc::new(InterceptState::default()),
            platform_state: Arc::new(PlatformState::new(proxy)),
            notifications_state: Arc::new(WebviewNotificationsState::default()),
        }
    }
}

impl WebviewManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert_webview(&mut self, window_id: WindowId, webview: WebView, context: WebContext, enabled: bool) {
        let webview_arc = Arc::new(WindowState::new(window_id.clone(), webview, context, enabled));
        self.fig_id_map.insert(window_id, webview_arc.clone());
        self.window_id_map
            .insert(webview_arc.webview.window().id(), webview_arc);
    }

    pub fn build_webview<T>(
        &mut self,
        window_id: WindowId,
        builder: impl Fn(&mut WebContext, &EventLoop, T) -> wry::Result<WebView>,
        options: T,
        enabled: bool,
    ) -> anyhow::Result<()> {
        let context_path = directories::fig_data_dir()?
            .join("webcontexts")
            .join(window_id.0.as_ref());
        let mut context = WebContext::new(Some(context_path));
        let webview = builder(&mut context, &self.event_loop, options)?;
        self.insert_webview(window_id, webview, context, enabled);
        Ok(())
    }

    pub async fn run(self) -> wry::Result<()> {
        self.platform_state
            .handle(PlatformBoundEvent::Initialize, &self.event_loop, &self.fig_id_map)
            .expect("Failed to initialize platform state");

        // TODO(mia): implement
        // tokio::spawn(figterm::clean_figterm_cache(self.figterm_state.clone()));

        tokio::spawn(local_ipc::start_local_ipc(
            self.platform_state.clone(),
            self.figterm_state.clone(),
            self.event_loop.create_proxy(),
        ));

        tokio::spawn(secure_ipc::start_secure_ipc(
            self.figterm_state.clone(),
            self.notifications_state.clone(),
            self.event_loop.create_proxy(),
        ));

        tokio::spawn(crate::figterm::clean_figterm_cache(self.figterm_state.clone()));

        let (api_handler_tx, mut api_handler_rx) = tokio::sync::mpsc::unbounded_channel::<(WindowId, String)>();
        let (sync_api_handler_tx, mut sync_api_handler_rx) = tokio::sync::mpsc::unbounded_channel::<(
            WindowId,
            fig_desktop_api::error::Result<ClientOriginatedMessage>,
        )>();

        {
            let sync_proxy = self.event_loop.create_proxy();
            let sync_debug_state = self.debug_state.clone();
            let sync_figterm_state = self.figterm_state.clone();
            let sync_intercept_state = self.intercept_state.clone();
            let sync_notifications_state = self.notifications_state.clone();
            let sync_platform_state = self.platform_state.clone();

            tokio::spawn(async move {
                while let Some((fig_id, message)) = sync_api_handler_rx.recv().await {
                    let proxy = sync_proxy.clone();
                    let debug_state = sync_debug_state.clone();
                    let figterm_state = sync_figterm_state.clone();
                    let intercept_state = sync_intercept_state.clone();
                    let notifications_state = sync_notifications_state.clone();
                    let platform_state = sync_platform_state.clone();
                    api_request(
                        fig_id,
                        message,
                        &debug_state,
                        &figterm_state,
                        &intercept_state,
                        &notifications_state,
                        &platform_state,
                        &proxy.clone(),
                    )
                    .await;
                }
            });

            let proxy = self.event_loop.create_proxy();
            let debug_state = self.debug_state.clone();
            let figterm_state = self.figterm_state.clone();
            let intercept_state = self.intercept_state.clone();
            let notifications_state = self.notifications_state.clone();
            let platform_state = self.platform_state.clone();

            tokio::spawn(async move {
                while let Some((fig_id, payload)) = api_handler_rx.recv().await {
                    let message = fig_desktop_api::handler::request_from_b64(&payload);
                    if matches!(
                        message,
                        Ok(ClientOriginatedMessage {
                            id: _,
                            submessage: Some(Submessage::PositionWindowRequest(_))
                        }) | Ok(ClientOriginatedMessage {
                            id: _,
                            submessage: Some(Submessage::WindowFocusRequest(_))
                        })
                    ) {
                        sync_api_handler_tx.send((fig_id, message)).ok();
                    } else {
                        let proxy = proxy.clone();
                        let debug_state = debug_state.clone();
                        let figterm_state = figterm_state.clone();
                        let intercept_state = intercept_state.clone();
                        let notifications_state = notifications_state.clone();
                        let platform_state = platform_state.clone();
                        tokio::spawn(async move {
                            api_request(
                                fig_id,
                                message,
                                &debug_state,
                                &figterm_state,
                                &intercept_state,
                                &notifications_state,
                                &platform_state,
                                &proxy.clone(),
                            )
                            .await;
                        });
                    }
                }
            });
        }

        file_watcher::user_data_listener(self.notifications_state.clone(), self.event_loop.create_proxy()).await;

        init_webview_notification_listeners(self.event_loop.create_proxy()).await;

        let tray_enabled = !fig_settings::settings::get_bool_or("app.hideMenubarIcon", false);
        let mut tray = if tray_enabled {
            Some(build_tray(&self.event_loop, &self.debug_state, &self.figterm_state).unwrap())
        } else {
            None
        };

        let proxy = self.event_loop.create_proxy();
        self.event_loop.run(move |event, window_target, control_flow| {
            *control_flow = ControlFlow::Wait;
            trace!(?event, "Main loop event");

            match event {
                WryEvent::NewEvents(StartCause::Init) => info!("Fig has started"),
                WryEvent::WindowEvent { event, window_id, .. } => {
                    if let Some(window_state) = self.window_id_map.get(&window_id) {
                        match event {
                            WryWindowEvent::CloseRequested => {
                                window_state.webview.window().set_visible(false);

                                if window_state.window_id == DASHBOARD_ID {
                                    match is_logged_in() {
                                        true => {
                                            proxy
                                                .send_event(Event::PlatformBoundEvent(
                                                    PlatformBoundEvent::AppWindowFocusChanged {
                                                        window_id: DASHBOARD_ID,
                                                        focused: false,
                                                        fullscreen: false,
                                                    },
                                                ))
                                                .ok();
                                        },
                                        false => *control_flow = ControlFlow::Exit,
                                    }
                                }
                            },
                            WryWindowEvent::ThemeChanged(theme) => window_state.set_theme(Some(theme)),

                            WryWindowEvent::Focused(focused) => {
                                if focused && window_state.window_id != AUTOCOMPLETE_ID {
                                    proxy
                                        .send_event(Event::WindowEvent {
                                            window_id: AUTOCOMPLETE_ID,
                                            window_event: WindowEvent::Hide,
                                        })
                                        .unwrap();
                                }

                                proxy
                                    .send_event(Event::PlatformBoundEvent(PlatformBoundEvent::AppWindowFocusChanged {
                                        window_id: window_state.window_id.clone(),
                                        focused,
                                        fullscreen: window_state.webview.window().fullscreen().is_some(),
                                    }))
                                    .unwrap();
                            },
                            _ => (),
                        }
                    }
                },
                WryEvent::MenuEvent { menu_id, origin, .. } => match origin {
                    MenuType::MenuBar => menu::handle_event(menu_id, &proxy),
                    MenuType::ContextMenu => {
                        if let Some(tray) = tray.as_mut() {
                            tray.set_menu(&get_context_menu());
                        }
                        tray::handle_event(menu_id, &proxy)
                    },
                    _ => {},
                },
                WryEvent::UserEvent(event) => {
                    match event {
                        Event::WindowEvent {
                            window_id,
                            window_event,
                        } => match self.fig_id_map.get(&window_id) {
                            Some(window_state) => {
                                if window_state.enabled() || window_event.is_allowed_while_disabled() {
                                    window_state.handle(
                                        window_event,
                                        &self.figterm_state,
                                        &self.platform_state,
                                        &self.notifications_state,
                                        window_target,
                                        &api_handler_tx,
                                    );
                                } else {
                                    trace!(
                                        window_id =% window_state.window_id,
                                        ?window_event,
                                        "Ignoring event for disabled window"
                                    );
                                }
                            },
                            None => {
                                // TODO(grant): figure out how to handle this gracefully
                                warn!("No window {window_id} available for event");
                                trace!(?window_event, "Event");
                            },
                        },
                        Event::WindowEventAll { window_event } => {
                            for window_state in self.window_id_map.iter() {
                                if window_state.enabled() || window_event.is_allowed_while_disabled() {
                                    window_state.handle(
                                        window_event.clone(),
                                        &self.figterm_state,
                                        &self.platform_state,
                                        &self.notifications_state,
                                        window_target,
                                        &api_handler_tx,
                                    );
                                } else {
                                    trace!(
                                        window_id =% window_state.window_id,
                                        ?window_event,
                                        "Ignoring event for disabled window"
                                    );
                                }
                            }
                        },
                        Event::ControlFlow(new_control_flow) => {
                            *control_flow = new_control_flow;
                        },
                        Event::ReloadTray => {
                            if let Some(tray) = tray.as_mut() {
                                tray.set_menu(&get_context_menu());
                            }
                        },
                        Event::ReloadCredentials => {
                            if let Some(tray) = tray.as_mut() {
                                tray.set_menu(&get_context_menu());
                            }

                            let autocomplete_enabled =
                                !fig_settings::settings::get_bool_or("autocomplete.disable", false)
                                    && PlatformState::accessibility_is_enabled().unwrap_or(true)
                                    && fig_request::auth::is_logged_in();

                            proxy
                                .send_event(Event::WindowEvent {
                                    window_id: AUTOCOMPLETE_ID,
                                    window_event: WindowEvent::SetEnabled(autocomplete_enabled),
                                })
                                .unwrap();
                        },
                        Event::ReloadAccessibility => {
                            if let Some(tray) = tray.as_mut() {
                                tray.set_menu(&get_context_menu());
                            }

                            let autocomplete_enabled =
                                !fig_settings::settings::get_bool_or("autocomplete.disable", false)
                                    && PlatformState::accessibility_is_enabled().unwrap_or(true)
                                    && fig_request::auth::is_logged_in();

                            proxy
                                .send_event(Event::WindowEvent {
                                    window_id: AUTOCOMPLETE_ID,
                                    window_event: WindowEvent::SetEnabled(autocomplete_enabled),
                                })
                                .unwrap();
                        },
                        Event::SetTrayEnabled(enabled) => {
                            if enabled {
                                if tray.is_none() {
                                    tray = Some(
                                        build_tray(window_target, &self.debug_state, &self.figterm_state).unwrap(),
                                    );
                                }
                            } else {
                                tray = None;
                            }
                        },
                        Event::PlatformBoundEvent(native_event) => {
                            if let Err(err) = self
                                .platform_state
                                .handle(native_event, window_target, &self.fig_id_map)
                            {
                                debug!(%err, "Failed to handle native event");
                            }
                        },
                        Event::ShowMessageNotification { title, body, parent } => {
                            let mut dialog = rfd::MessageDialog::new().set_title(&title).set_description(&body);

                            if let Some(parent) = parent {
                                if let Some(parent_window) = self.fig_id_map.get(&parent) {
                                    dialog = dialog.set_parent(parent_window.webview.window());
                                }
                            }

                            dialog.show();
                        },
                    }
                },
                WryEvent::MainEventsCleared | WryEvent::NewEvents(StartCause::WaitCancelled { .. }) => {},
                event => trace!(?event, "Unhandled event"),
            }

            if matches!(*control_flow, ControlFlow::Exit | ControlFlow::ExitWithCode(_)) {
                tokio::runtime::Handle::current().spawn(fig_telemetry::dispatch_emit_track(
                    fig_telemetry::TrackEvent::new(
                        fig_telemetry::TrackEventType::QuitApp,
                        fig_telemetry::TrackSource::Desktop,
                        env!("CARGO_PKG_VERSION").into(),
                        empty::<(&str, &str)>(),
                    ),
                    false,
                ));
            }
        });
    }
}

fn navigation_handler<I, S>(window_id: WindowId, exprs: I) -> impl Fn(String) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let regex_set = RegexSet::new(exprs);

    if let Err(ref err) = regex_set {
        error!("Failed to compile regex: {err}");
    }

    move |url: String| match regex_set.as_ref().ok().and_then(|r| {
        Url::parse(&url)
            .ok()
            .and_then(|url| url.domain().map(|domain| r.is_match(domain)))
    }) {
        Some(true) => {
            trace!("{window_id} allowed url: {url}");
            true
        },
        Some(false) | None => {
            warn!("{window_id} denied url: {url}");
            false
        },
    }
}

pub struct DashboardOptions {
    pub show_onboarding: bool,
    pub force_visible: bool,
    pub page: Option<String>,
}

pub fn build_dashboard(
    web_context: &mut WebContext,
    event_loop: &EventLoop,
    DashboardOptions {
        show_onboarding,
        force_visible,
        page,
    }: DashboardOptions,
) -> wry::Result<WebView> {
    let is_visible = !fig_request::auth::is_logged_in() || force_visible || show_onboarding;

    let mut window = WindowBuilder::new()
        .with_title("Fig Dashboard")
        .with_resizable(true)
        .with_visible(is_visible)
        .with_focused(is_visible)
        .with_always_on_top(false)
        .with_window_icon(Some(utils::ICON.clone()))
        .with_menu(menu::menu_bar())
        .with_theme(*THEME);

    match show_onboarding {
        true => window = window.with_inner_size(DASHBOARD_ONBOARDING_SIZE),
        false => {
            window = window
                .with_inner_size(DASHBOARD_INITIAL_SIZE)
                .with_min_inner_size(DASHBOARD_MINIMUM_SIZE)
        },
    }

    let window = window.build(event_loop)?;

    #[cfg(target_os = "linux")]
    {
        use gtk::traits::GtkWindowExt;
        use wry::application::platform::unix::WindowExtUnix;

        window.gtk_window().set_role("dashboard");
    }

    let proxy = event_loop.create_proxy();

    let base_url =
        fig_settings::settings::get_string_or("developer.mission-control.host", "https://desktop.fig.io".into());

    let url = if show_onboarding {
        format!("{base_url}{ONBOARDING_PATH}")
    } else {
        match page {
            Some(page) => format!("{base_url}/{page}"),
            None => base_url,
        }
    };

    let webview = WebViewBuilder::new(window)?
        .with_web_context(web_context)
        .with_url(url.as_str())?
        .with_ipc_handler(move |_window, payload| {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::Api {
                        payload: payload.into(),
                    },
                })
                .unwrap();
        })
        .with_devtools(true)
        .with_navigation_handler(navigation_handler(DASHBOARD_ID, &[
            // Main domain
            r"^desktop\.fig\.io$",
            // Dev domains
            r"^localhost$",
            r"^127\.0\.0\.1$",
            r"-withfig\.vercel\.app$",
        ]))
        .with_initialization_script(&javascript_init())
        .with_clipboard(true)
        .with_hotkeys_zoom(true)
        .build()?;

    Ok(webview)
}

pub struct AutocompleteOptions {}

pub fn build_autocomplete(
    web_context: &mut WebContext,
    event_loop: &EventLoop,
    _autocomplete_options: AutocompleteOptions,
) -> wry::Result<WebView> {
    let mut window_builder = WindowBuilder::new()
        .with_title(AUTOCOMPLETE_WINDOW_TITLE)
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top(true)
        .with_visible(false)
        .with_focused(false)
        .with_window_icon(Some(utils::ICON.clone()))
        .with_theme(*THEME);

    cfg_if!(
        if #[cfg(target_os = "linux")] {
            use wry::application::platform::unix::WindowBuilderExtUnix;
            window_builder = window_builder.with_resizable(true).with_skip_taskbar(true);
        } else if #[cfg(target_os = "macos")] {
            window_builder = window_builder.with_resizable(false);
        } else if #[cfg(target_os = "windows")] {
            use wry::application::platform::windows::WindowBuilderExtWindows;
            window_builder = window_builder.with_resizable(false).with_skip_taskbar(true);
        }
    );

    let window = window_builder.build(event_loop)?;

    #[cfg(target_os = "linux")]
    {
        use gtk::gdk::WindowTypeHint;
        use gtk::traits::{
            GtkWindowExt,
            WidgetExt,
        };
        use wry::application::platform::unix::WindowExtUnix;

        let gtk_window = window.gtk_window();
        gtk_window.set_role("autocomplete");
        gtk_window.set_type_hint(WindowTypeHint::Utility);
        gtk_window.set_accept_focus(false);
        gtk_window.set_decorated(false);
        if let Some(window) = gtk_window.window() {
            window.set_override_redirect(true);
        }
    }

    let proxy = event_loop.create_proxy();

    let webview = WebViewBuilder::new(window)?
        .with_web_context(web_context)
        .with_url(&fig_settings::settings::get_string_or(
            "developer.autocomplete.host",
            "https://autocomplete.fig.io/".into(),
        ))?
        .with_ipc_handler(move |_window, payload| {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: AUTOCOMPLETE_ID.clone(),
                    window_event: WindowEvent::Api {
                        payload: payload.into(),
                    },
                })
                .unwrap();
        })
        .with_custom_protocol("fig".into(), utils::wrap_custom_protocol(icons::handle))
        .with_devtools(true)
        .with_transparent(true)
        .with_initialization_script(&javascript_init())
        .with_navigation_handler(navigation_handler(AUTOCOMPLETE_ID, &[
            // Main domain
            r"^autocomplete\.fig\.io$",
            // Dev domains
            r"^localhost$",
            r"^127\.0\.0\.1$",
            r"-withfig\.vercel\.app$",
            // Legacy domains
            r"^app\.withfig\.com$",
            r"^staging\.withfig\.com$",
            r"^fig-autocomplete\.vercel\.app$",
        ]))
        .with_clipboard(true)
        .with_hotkeys_zoom(true)
        .build()?;

    Ok(webview)
}

async fn init_webview_notification_listeners(proxy: EventLoopProxy) {
    macro_rules! settings_watcher {
        ($name:expr, $on_update:expr) => {{
            let proxy = proxy.clone();
            tokio::spawn(async move {
                let mut rx = NOTIFICATION_BUS.subscribe_settings($name.into());
                loop {
                    let res = rx.recv().await;
                    match res {
                        Ok(val) => {
                            #[allow(clippy::redundant_closure_call)]
                            ($on_update)(val, &proxy);
                        },
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            warn!("Notification bus '{}' lagged by {n} messages", $name);
                        },
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });
        };};
    }

    // This one isnt working properly and permanently locks autocomplete :(
    settings_watcher!(
        "autocomplete.disable",
        |notification: JsonNotification, proxy: &EventLoopProxy| {
            let enabled = !notification.as_bool().unwrap_or(false);
            debug!(%enabled, "Autocomplete");
            proxy
                .send_event(Event::WindowEvent {
                    window_id: AUTOCOMPLETE_ID,
                    window_event: WindowEvent::SetEnabled(enabled),
                })
                .unwrap();
        }
    );

    settings_watcher!("app.theme", |notification: JsonNotification, proxy: &EventLoopProxy| {
        let theme = notification.as_string().as_deref().and_then(map_theme);
        debug!(?theme, "Theme changed");
        proxy
            .send_event(Event::WindowEventAll {
                window_event: WindowEvent::SetTheme(theme),
            })
            .unwrap();
    });

    settings_watcher!(
        "app.hideMenubarIcon",
        |notification: JsonNotification, proxy: &EventLoopProxy| {
            let enabled = !notification.as_bool().unwrap_or(false);
            debug!(%enabled, "Tray icon");
            proxy.send_event(Event::SetTrayEnabled(enabled)).unwrap();
        }
    );

    tokio::spawn(async move {
        let mut res = NOTIFICATION_BUS.subscribe_user_email();
        loop {
            match res.recv().await {
                Ok(Some(_)) => {
                    // Unclear what should happen here, navigation is probably wrong, should
                    // probably notify the web side that this happened
                },
                Ok(None) => {
                    proxy
                        .send_event(Event::WindowEvent {
                            window_id: DASHBOARD_ID,
                            window_event: WindowEvent::NavigateRelative {
                                path: "/onboarding/welcome".to_owned(),
                            },
                        })
                        .ok();

                    proxy
                        .send_event(Event::WindowEvent {
                            window_id: DASHBOARD_ID,
                            window_event: WindowEvent::Resize {
                                size: DASHBOARD_ONBOARDING_SIZE,
                            },
                        })
                        .ok();

                    proxy
                        .send_event(Event::WindowEvent {
                            window_id: DASHBOARD_ID,
                            window_event: WindowEvent::Center,
                        })
                        .ok();
                },
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    warn!("Notification bus 'userEmail' lagged by {n} messages");
                },
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}
