mod util;

use std::borrow::Cow;
use std::iter::empty;
use std::sync::Arc;

use cfg_if::cfg_if;
use dashmap::DashMap;
use fig_util::directories;
use fnv::FnvBuildHasher;
use regex::RegexSet;
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};
use url::Url;
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
    WindowBuilder,
    WindowId as WryWindowId,
};
use wry::webview::{
    WebContext,
    WebView,
    WebViewBuilder,
};

use crate::api::{
    api_request,
    javascript_init,
};
use crate::event::{
    Event,
    WindowEvent,
};
use crate::figterm::FigtermState;
use crate::native::NativeState;
use crate::notification::NotificationsState;
use crate::tray::{
    self,
    build_tray,
};
use crate::window::{
    WindowId,
    WindowState,
};
use crate::{
    figterm,
    icons,
    local_ipc,
    native,
    settings,
    DebugState,
    EventLoop,
    InterceptState,
};

pub const FIG_PROTO_MESSAGE_RECIEVED: &str = "FigProtoMessageRecieved";

pub const MISSION_CONTROL_ID: WindowId = WindowId(Cow::Borrowed("mission-control"));
pub const AUTOCOMPLETE_ID: WindowId = WindowId(Cow::Borrowed("autocomplete"));

pub struct WebviewManager {
    fig_id_map: DashMap<WindowId, Arc<WindowState>, FnvBuildHasher>,
    window_id_map: DashMap<WryWindowId, Arc<WindowState>, FnvBuildHasher>,
    event_loop: EventLoop,
    debug_state: Arc<DebugState>,
    figterm_state: Arc<FigtermState>,
    intercept_state: Arc<InterceptState>,
    native_state: NativeState,
    notifications_state: Arc<NotificationsState>,
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
            native_state: NativeState::new(proxy),
            notifications_state: Arc::new(NotificationsState::default()),
        }
    }
}

impl WebviewManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert_webview(&mut self, window_id: WindowId, webview: WebView) {
        let webview_arc = Arc::new(WindowState::new(window_id.clone(), webview));
        self.fig_id_map.insert(window_id, webview_arc.clone());
        self.window_id_map
            .insert(webview_arc.webview.window().id(), webview_arc);
    }

    pub fn build_webview<T>(
        &mut self,
        window_id: WindowId,
        builder: impl Fn(&mut WebContext, &EventLoop, T) -> wry::Result<WebView>,
        options: T,
    ) -> wry::Result<()> {
        let webview = builder(
            &mut WebContext::new(directories::fig_data_dir().ok()),
            &self.event_loop,
            options,
        )?;
        self.insert_webview(window_id, webview);
        Ok(())
    }

    pub async fn run(self) -> wry::Result<()> {
        let (api_handler_tx, mut api_handler_rx) = tokio::sync::mpsc::unbounded_channel::<(WindowId, String)>();

        native::init(self.event_loop.create_proxy())
            .await
            .expect("Failed to initialize native integrations");

        tokio::spawn(figterm::clean_figterm_cache(self.figterm_state.clone()));

        tokio::spawn(local_ipc::start_local_ipc(
            self.figterm_state.clone(),
            self.notifications_state.clone(),
            self.event_loop.create_proxy(),
        ));

        {
            let proxy = self.event_loop.create_proxy();
            let debug_state = self.debug_state.clone();
            let figterm_state = self.figterm_state.clone();
            let intercept_state = self.intercept_state.clone();
            let notifications_state = self.notifications_state.clone();
            tokio::spawn(async move {
                while let Some((fig_id, payload)) = api_handler_rx.recv().await {
                    api_request(
                        fig_id,
                        payload,
                        &debug_state,
                        &figterm_state,
                        &intercept_state,
                        &notifications_state,
                        &proxy,
                    )
                    .await;
                }
            });
        }

        settings::settings_listener(self.notifications_state.clone(), self.event_loop.create_proxy()).await;

        let _tray = build_tray(&self.event_loop, &self.debug_state, &self.figterm_state).unwrap();

        let proxy = self.event_loop.create_proxy();
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                WryEvent::NewEvents(StartCause::Init) => info!("Fig has started"),
                WryEvent::WindowEvent { event, window_id, .. } => {
                    if let Some(window_state) = self.window_id_map.get(&window_id) {
                        match event {
                            WryWindowEvent::CloseRequested => window_state.webview.window().set_visible(false),
                            WryWindowEvent::ThemeChanged(_) => {
                                // TODO: handle this
                            },
                            _ => (),
                        }
                    }
                },
                WryEvent::MenuEvent {
                    menu_id,
                    origin: MenuType::ContextMenu,
                    ..
                } => tray::handle_event(menu_id, &proxy),
                WryEvent::UserEvent(event) => {
                    trace!("Executing user event: {event:?}");
                    match event {
                        Event::WindowEvent {
                            window_id,
                            window_event,
                        } => match self.fig_id_map.get(&window_id) {
                            Some(window_state) => {
                                window_state.handle(window_event, &self.figterm_state, &api_handler_tx);
                            },
                            None => todo!(),
                        },
                        Event::ControlFlow(new_control_flow) => {
                            *control_flow = new_control_flow;
                        },
                        Event::RefreshDebugger => {
                            // TODO(grant): Refresh the debugger
                        },
                        Event::NativeEvent(native_event) => {
                            if let Err(e) = self.native_state.handle(native_event) {
                                debug!("Failed to handle native event: {e}");
                            }
                        },
                    }
                },
                WryEvent::MainEventsCleared | WryEvent::NewEvents(StartCause::WaitCancelled { .. }) => {},
                event => trace!("Unhandled event {event:?}"),
            }

            if matches!(*control_flow, ControlFlow::Exit | ControlFlow::ExitWithCode(_)) {
                tokio::runtime::Handle::current().spawn(fig_telemetry::dispatch_emit_track(
                    fig_telemetry::TrackEvent::new(
                        fig_telemetry::TrackEventType::QuitApp,
                        fig_telemetry::TrackSource::App,
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
            warn!("{window_id} denyed url: {url}");
            false
        },
    }
}

pub struct MissionControlOptions {
    pub show_onboarding: bool,
    pub force_visible: bool,
}

pub fn build_mission_control(
    web_context: &mut WebContext,
    event_loop: &EventLoop,
    MissionControlOptions {
        show_onboarding,
        force_visible,
    }: MissionControlOptions,
) -> wry::Result<WebView> {
    let is_visible = !fig_auth::is_logged_in() || force_visible || show_onboarding;

    let window = WindowBuilder::new()
        .with_resizable(true)
        .with_title("Fig")
        .with_visible(is_visible)
        .with_always_on_top(false)
        .build(event_loop)?;

    #[cfg(target_os = "linux")]
    {
        use gtk::traits::GtkWindowExt;
        use wry::application::platform::unix::WindowExtUnix;

        window.gtk_window().set_role("mission_control");
    }

    let proxy = event_loop.create_proxy();

    let webview = WebViewBuilder::new(window)?
        .with_web_context(web_context)
        .with_url(if show_onboarding {
            "https://desktop.fig.io/onboarding/1"
        } else {
            "https://desktop.fig.io"
        })?
        .with_ipc_handler(move |_window, payload| {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: MISSION_CONTROL_ID.clone(),
                    window_event: WindowEvent::Api { payload },
                })
                .unwrap();
        })
        .with_devtools(true)
        .with_navigation_handler(navigation_handler(MISSION_CONTROL_ID, &[
            r"^localhost$",
            r"^127\.0\.0\.1$",
            r"^desktop\.fig\.io$",
            r"-withfig\.vercel\.app$",
        ]))
        .with_initialization_script(&javascript_init())
        .with_clipboard(true)
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
        .with_title("Fig Autocomplete")
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top(true)
        .with_visible(false);

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
        use gtk::traits::GtkWindowExt;
        use wry::application::platform::unix::WindowExtUnix;

        window.gtk_window().set_type_hint(WindowTypeHint::Utility);
        window.gtk_window().set_role("autocomplete");
    }

    let proxy = event_loop.create_proxy();

    let webview = WebViewBuilder::new(window)?
        .with_web_context(web_context)
        .with_url("https://app.withfig.com/autocomplete/v9")?
        .with_ipc_handler(move |_window, payload| {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: AUTOCOMPLETE_ID.clone(),
                    window_event: WindowEvent::Api { payload },
                })
                .unwrap();
        })
        .with_custom_protocol("fig".into(), util::wrap_custom_protocol(icons::handle))
        .with_devtools(true)
        .with_transparent(true)
        .with_initialization_script(&javascript_init())
        .with_navigation_handler(navigation_handler(AUTOCOMPLETE_ID, &[
            r"^localhost$",
            r"^127\.0\.0\.1$",
            r"^staging\.withfig\.com$",
            r"^app\.withfig\.com$",
            r"-withfig\.vercel\.app$",
        ]))
        .with_clipboard(true)
        .build()?;

    Ok(webview)
}
