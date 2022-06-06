#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod api;
mod cli;
mod event;
mod figterm;
mod icons;
mod local_ipc;
mod native;
mod tray;
mod utils;
mod window;

use std::borrow::Cow;
use std::sync::Arc;

use api::{
    api_request,
    javascript_init,
};
use cfg_if::cfg_if;
use clap::Parser;
use dashmap::DashMap;
use event::{
    Event,
    WindowEvent,
};
use fig_proto::fig::NotificationType;
use figterm::FigtermState;
use fnv::FnvBuildHasher;
use native::NativeState;
use parking_lot::RwLock;
use regex::RegexSet;
use sysinfo::{
    get_current_pid,
    ProcessExt,
    ProcessRefreshKind,
    RefreshKind,
    System,
    SystemExt,
};
use tokio::runtime::Runtime;
use tracing::{
    info,
    trace,
    warn,
};
use tray::create_tray;
use url::Url;
use window::{
    WindowId,
    WindowState,
};
use wry::application::event::{
    Event as WryEvent,
    StartCause,
    WindowEvent as WryWindowEvent,
};
use wry::application::event_loop::{
    ControlFlow,
    EventLoop as WryEventLoop,
    EventLoopProxy as WryEventLoopProxy,
};
use wry::application::menu::MenuType;
use wry::application::window::{
    WindowBuilder,
    WindowId as WryWindowId,
};
use wry::webview::{
    WebView,
    WebViewBuilder,
};

const FIG_PROTO_MESSAGE_RECIEVED: &str = "FigProtoMessageRecieved";

const MISSION_CONTROL_ID: WindowId = WindowId(Cow::Borrowed("mission-control"));
const AUTOCOMPLETE_ID: WindowId = WindowId(Cow::Borrowed("autocomplete"));

#[derive(Debug, Default)]
pub struct DebugState {
    pub debug_lines: RwLock<Vec<String>>,
    pub color: RwLock<Option<String>>,
}

#[derive(Debug, Default)]
pub struct InterceptState {
    pub intercept_bound_keystrokes: RwLock<bool>,
    pub intercept_global_keystrokes: RwLock<bool>,
}

#[derive(Debug, Default)]
pub struct NotificationsState {
    subscriptions: DashMap<WindowId, DashMap<NotificationType, i64, FnvBuildHasher>, FnvBuildHasher>,
}

pub type EventLoop = WryEventLoop<Event>;
pub type EventLoopProxy = WryEventLoopProxy<Event>;

#[derive(Debug, Default)]
pub struct GlobalState {
    pub debug_state: DebugState,
    pub figterm_state: FigtermState,
    pub intercept_state: InterceptState,
    pub native_state: NativeState,
    pub notifications_state: NotificationsState,
}

struct WebviewManager {
    fig_id_map: DashMap<WindowId, Arc<WindowState>, FnvBuildHasher>,
    window_id_map: DashMap<WryWindowId, Arc<WindowState>, FnvBuildHasher>,
    event_loop: EventLoop,
    global_state: Arc<GlobalState>,
}

impl Default for WebviewManager {
    fn default() -> Self {
        Self {
            fig_id_map: Default::default(),
            window_id_map: Default::default(),
            event_loop: WryEventLoop::with_user_event(),
            global_state: Default::default(),
        }
    }
}

impl WebviewManager {
    fn new() -> Self {
        Self::default()
    }

    fn insert_webview(&mut self, window_id: WindowId, webview: WebView) {
        let webview_arc = Arc::new(WindowState::new(window_id.clone(), webview));
        self.fig_id_map.insert(window_id, webview_arc.clone());
        self.window_id_map
            .insert(webview_arc.webview.window().id(), webview_arc);
    }

    fn build_webview<T>(
        &mut self,
        window_id: WindowId,
        builder: impl Fn(&EventLoop, T) -> wry::Result<WebView>,
        options: T,
    ) -> wry::Result<()> {
        let webview = builder(&self.event_loop, options)?;
        self.insert_webview(window_id, webview);
        Ok(())
    }

    async fn run(self) -> wry::Result<()> {
        let (api_handler_tx, mut api_handler_rx) = tokio::sync::mpsc::unbounded_channel::<(WindowId, String)>();

        native::init(self.global_state.clone(), self.event_loop.create_proxy())
            .await
            .expect("Failed to initialize native integrations");

        tokio::spawn(figterm::clean_figterm_cache(self.global_state.clone()));

        tokio::spawn(local_ipc::start_local_ipc(
            self.global_state.clone(),
            self.event_loop.create_proxy(),
        ));

        let proxy = self.event_loop.create_proxy();
        let global_state = self.global_state.clone();
        tokio::spawn(async move {
            while let Some((fig_id, payload)) = api_handler_rx.recv().await {
                api_request(fig_id, payload, &global_state, &proxy).await;
            }
        });

        let tray = create_tray(&self.event_loop).unwrap();
        let proxy = self.event_loop.create_proxy();

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                WryEvent::NewEvents(StartCause::Init) => info!("Fig has started"),
                WryEvent::WindowEvent { event, window_id, .. } => {
                    if let Some(window_state) = self.window_id_map.get(&window_id) {
                        match event {
                            WryWindowEvent::Resized(_) => window_state.webview.resize().unwrap(),
                            WryWindowEvent::CloseRequested => window_state.webview.window().set_visible(false),
                            _ => (),
                        }
                    }
                },
                WryEvent::MenuEvent {
                    menu_id,
                    origin: MenuType::ContextMenu,
                    ..
                } => {
                    tray.handle_event(menu_id, &proxy);
                },
                WryEvent::UserEvent(event) => {
                    trace!("Executing user event: {event:?}");
                    match event {
                        Event::WindowEvent {
                            window_id,
                            window_event,
                        } => match self.fig_id_map.get(&window_id) {
                            Some(window_state) => {
                                window_state.handle(window_event, &self.global_state, &api_handler_tx);
                            },
                            None => todo!(),
                        },
                        Event::ControlFlow(new_control_flow) => {
                            *control_flow = new_control_flow;
                        },
                    }
                },
                WryEvent::MainEventsCleared | WryEvent::NewEvents(StartCause::WaitCancelled { .. }) => {},
                event => trace!("Unhandled event {event:?}"),
            }
        });
    }
}

struct MissionControlOptions {
    force_visible: bool,
}

fn build_mission_control(
    event_loop: &EventLoop,
    MissionControlOptions { force_visible }: MissionControlOptions,
) -> wry::Result<WebView> {
    let is_visible = !fig_auth::is_logged_in() || force_visible;

    let window = WindowBuilder::new()
        .with_resizable(true)
        .with_title("Fig Mission Control")
        .with_visible(is_visible)
        .build(event_loop)?;

    let proxy = event_loop.create_proxy();

    let webview = WebViewBuilder::new(window)?
        .with_url("https://desktop.fig.io")?
        .with_ipc_handler(move |_window, payload| {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: MISSION_CONTROL_ID.clone(),
                    window_event: WindowEvent::Api { payload },
                })
                .unwrap();
        })
        .with_devtools(true)
        .with_navigation_handler(|url| {
            match Url::parse(&url).ok().and_then(|url| {
                url.domain().and_then(|domain| {
                    RegexSet::new(&[r"^localhost$", r"^desktop\.fig\.io$", r"-withfig\.vercel\.app$"])
                        .ok()
                        .map(|r| r.is_match(domain))
                })
            }) {
                Some(true) => {
                    trace!("{MISSION_CONTROL_ID} allowed url: {url}");
                    true
                },
                Some(false) | None => {
                    warn!("{MISSION_CONTROL_ID} denyed url: {url}");
                    false
                },
            }
        })
        .with_initialization_script(&javascript_init())
        .build()?;

    Ok(webview)
}

struct AutocompleteOptions {}

fn build_autocomplete(event_loop: &EventLoop, _autocomplete_options: AutocompleteOptions) -> wry::Result<WebView> {
    let mut window_builder = WindowBuilder::new()
        .with_title("Fig Autocomplete")
        .with_transparent(false)
        .with_decorations(false)
        .with_resizable(false)
        .with_always_on_top(true)
        .with_visible(true);

    cfg_if!(
        if #[cfg(target_os = "linux")] {
            use wry::application::platform::unix::WindowBuilderExtUnix;
            window_builder = window_builder.with_resizable(true).with_skip_taskbar(true);
        } else if #[cfg(target_os = "windows")] {
            use wry::application::platform::windows::WindowBuilderExtWindows;
            window_builder = window_builder.with_resizable(false).with_skip_taskbar(true);
        } else {
            window_builder = window_builder.with_resizable(false);
        }
    );

    let window = window_builder.build(event_loop)?;

    #[cfg(target_os = "linux")]
    {
        use gtk::gdk::WindowTypeHint;
        use gtk::traits::GtkWindowExt;
        use wry::application::platform::unix::WindowExtUnix;

        window.gtk_window().set_type_hint(WindowTypeHint::Utility);
    }

    let proxy = event_loop.create_proxy();

    let webview = WebViewBuilder::new(window)?
        .with_url("https://staging.withfig.com/autocomplete/v9")?
        .with_ipc_handler(move |_window, payload| {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: AUTOCOMPLETE_ID.clone(),
                    window_event: WindowEvent::Api { payload },
                })
                .unwrap();
        })
        .with_custom_protocol("fig".into(), icons::handle)
        .with_devtools(true)
        .with_transparent(false)
        .with_initialization_script(&javascript_init())
        .with_navigation_handler(|url| {
            url.starts_with("http://localhost")
                || url.starts_with("https://staging.withfig.com/autocomplete")
                || url.starts_with("https://app.withfig.com/autocomplete")
        })
        .build()?;

    Ok(webview)
}

fn main() {
    let _sentry_guard =
        fig_telemetry::init_sentry("https://4295cb4f204845958717e406b331948d@o436453.ingest.sentry.io/6432682");
    let _logger_guard = fig_log::init_logger("fig_tauri.log").expect("Failed to initialize logger");

    let cli = cli::Cli::parse();

    if !cli.allow_multiple_instances {
        match get_current_pid() {
            Ok(current_pid) => {
                let system = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
                let processes = system.processes_by_exact_name("fig_desktop");
                for process in processes {
                    let pid = process.pid();
                    if current_pid != pid {
                        if cli.kill_instance {
                            process.kill();
                            let exe = process.exe().display();
                            eprintln!("Killing instance: {exe} ({pid})");
                        } else {
                            let exe = process.exe().display();
                            eprintln!("Fig is already running: {exe} ({pid})");
                            return;
                        }
                    }
                }
            },
            Err(err) => warn!("Failed to get pid: {err}"),
        }
    }

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut webview_manager = WebviewManager::new();
        webview_manager
            .build_webview(MISSION_CONTROL_ID, build_mission_control, MissionControlOptions {
                force_visible: cli.mission_control_open,
            })
            .unwrap();
        webview_manager
            .build_webview(AUTOCOMPLETE_ID, build_autocomplete, AutocompleteOptions {})
            .unwrap();
        webview_manager.run().await.unwrap();
    });
}
