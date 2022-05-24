#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod api;
mod figterm;
mod icons;
mod local_ipc;
mod native;
mod tray;
mod utils;
mod window;

use std::borrow::Cow;
use std::sync::Arc;

use dashmap::DashMap;
use fig_proto::fig::NotificationType;
use figterm::FigtermState;
use fnv::FnvBuildHasher;
use gtk::gdk::WindowTypeHint;
use gtk::traits::GtkWindowExt;
use native::NativeState;
use parking_lot::RwLock;
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
    debug,
    info,
    warn,
};
use tray::create_tray;
use window::{
    FigWindowEvent,
    WindowState,
};
use wry::application::event::{
    Event,
    StartCause,
    WindowEvent,
};
use wry::application::event_loop::{
    ControlFlow,
    EventLoop,
};
use wry::application::menu::MenuType;
use wry::application::platform::unix::{
    WindowBuilderExtUnix,
    WindowExtUnix,
};
use wry::application::window::{
    WindowBuilder,
    WindowId,
};
use wry::webview::{
    WebView,
    WebViewBuilder,
};

use crate::api::api_request;

const FIG_PROTO_MESSAGE_RECIEVED: &str = "FigProtoMessageRecieved";
// TODO: Add constants
const JAVASCRIPT_INIT: &str = r#"
console.log("[fig] declaring constants...")

if (!window.fig) {
    window.fig = {}
}

if (!window.fig.constants) {
    window.fig.constants = {}
}
"#;

const MISSION_CONTROL_ID: FigId = FigId(Cow::Borrowed("mission-control"));
const AUTOCOMPLETE_ID: FigId = FigId(Cow::Borrowed("autocomplete"));

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FigId(pub Cow<'static, str>);

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
    subscriptions: DashMap<FigId, DashMap<NotificationType, i64, FnvBuildHasher>, FnvBuildHasher>,
}

#[derive(Debug)]
pub enum FigEvent {
    WindowEvent {
        fig_id: FigId,
        window_event: FigWindowEvent,
    },
    ControlFlow(ControlFlow),
}

pub type FigEventLoop = EventLoop<FigEvent>;

#[derive(Debug, Default)]
pub struct GlobalState {
    pub debug_state: DebugState,
    pub figterm_state: FigtermState,
    pub intercept_state: InterceptState,
    pub native_state: NativeState,
    pub notifications_state: NotificationsState,
}

struct WebviewManager {
    fig_id_map: DashMap<FigId, Arc<WindowState>, FnvBuildHasher>,
    window_id_map: DashMap<WindowId, Arc<WindowState>, FnvBuildHasher>,
    event_loop: FigEventLoop,
    global_state: Arc<GlobalState>,
}

impl WebviewManager {
    fn new() -> Self {
        let event_loop = EventLoop::with_user_event();
        let proxy = event_loop.create_proxy();
        Self {
            fig_id_map: Default::default(),
            window_id_map: Default::default(),
            event_loop,
            global_state: Arc::new(Default::default()),
        }
    }

    fn insert_webview(&mut self, fig_id: FigId, webview: WebView) {
        let webview_arc = Arc::new(WindowState::new(fig_id.clone(), webview));
        self.fig_id_map.insert(fig_id, webview_arc.clone());
        self.window_id_map
            .insert(webview_arc.webview.window().id(), webview_arc);
    }

    fn build_webview(
        &mut self,
        fig_id: FigId,
        builder: impl Fn(&FigEventLoop) -> wry::Result<WebView>,
    ) -> wry::Result<()> {
        let webview = builder(&self.event_loop)?;
        self.insert_webview(fig_id, webview);
        Ok(())
    }

    async fn run(self) -> wry::Result<()> {
        let (api_handler_tx, mut api_handler_rx) = tokio::sync::mpsc::unbounded_channel::<(FigId, String)>();

        native::NativeState::execute(self.global_state.clone(), self.event_loop.create_proxy()).await;

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
                Event::NewEvents(StartCause::Init) => info!("Fig has started"),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                    ..
                } => {
                    if let Some(window_state) = self.window_id_map.get(&window_id) {
                        window_state.webview.window().set_visible(false);
                    }
                },
                Event::MenuEvent {
                    menu_id,
                    origin: MenuType::ContextMenu,
                    ..
                } => {
                    tray.handle_event(menu_id, &proxy);
                },
                Event::UserEvent(event) => {
                    debug!("Executing user event: {event:?}");
                    match event {
                        FigEvent::WindowEvent { fig_id, window_event } => match self.fig_id_map.get(&fig_id) {
                            Some(window_state) => {
                                window_state.handle(window_event, &self.global_state, &api_handler_tx);
                            },
                            None => todo!(),
                        },
                        FigEvent::ControlFlow(new_control_flow) => {
                            *control_flow = new_control_flow;
                        },
                    }
                },
                Event::MainEventsCleared | Event::NewEvents(StartCause::WaitCancelled { .. }) => {},
                event => warn!("Unhandled event {event:?}"),
            }
        });
    }
}

fn build_mission_control(event_loop: &FigEventLoop) -> wry::Result<WebView> {
    let window = WindowBuilder::new()
        .with_title("Fig Mission Control")
        .with_visible(false)
        .build(event_loop)?;

    let proxy = event_loop.create_proxy();

    let webview = WebViewBuilder::new(window)?
        .with_url("https://desktop.fig.io")?
        .with_ipc_handler(move |_window, payload| {
            proxy
                .send_event(FigEvent::WindowEvent {
                    fig_id: MISSION_CONTROL_ID.clone(),
                    window_event: FigWindowEvent::Api { payload },
                })
                .unwrap();
        })
        .with_devtools(true)
        .with_navigation_handler(|url| url.starts_with("http://localhost") || url.starts_with("https://desktop.fig.io"))
        .with_initialization_script(JAVASCRIPT_INIT)
        .build()?;

    Ok(webview)
}

fn build_autocomplete(event_loop: &FigEventLoop) -> wry::Result<WebView> {
    let window = WindowBuilder::new()
        .with_title("Fig Autocomplete")
        .with_transparent(true)
        .with_decorations(false)
        .with_skip_taskbar(true)
        .with_resizable(true)
        .with_always_on_top(true)
        //.with_inner_size(PhysicalSize { width: 1, height: 1 })
        .build(event_loop)?;

    window.set_visible(false);

    window.gtk_window().set_type_hint(WindowTypeHint::Utility);

    let proxy = event_loop.create_proxy();

    let webview = WebViewBuilder::new(window)?
        .with_url("https://staging.withfig.com/autocomplete/v9")?
        .with_ipc_handler(move |_window, payload| {
            proxy
                .send_event(FigEvent::WindowEvent {
                    fig_id: AUTOCOMPLETE_ID.clone(),
                    window_event: FigWindowEvent::Api { payload },
                })
                .unwrap();
        })
        .with_custom_protocol("fig".into(), icons::handle)
        .with_devtools(true)
        .with_transparent(true)
        .with_initialization_script(JAVASCRIPT_INIT)
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

    match get_current_pid() {
        Ok(current_pid) => {
            let system = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
            let processes = system.processes_by_exact_name("fig_desktop");
            for process in processes.into_iter() {
                let pid = process.pid();
                if current_pid != pid {
                    let exe = process.exe().display();
                    eprintln!("Fig is already running: {exe} ({pid})");
                    return;
                }
            }
        },
        Err(err) => warn!("Failed to get pid: {err}"),
    }

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        fig_log::init_logger("fig_tauri.log").expect("Failed to initialize logger");
        native::init().expect("Failed to initialize native integrations");

        let mut webview_manager = WebviewManager::new();
        webview_manager
            .build_webview(MISSION_CONTROL_ID, build_mission_control)
            .unwrap();
        webview_manager
            .build_webview(AUTOCOMPLETE_ID, build_autocomplete)
            .unwrap();
        webview_manager.run().await.unwrap();
    });
}
