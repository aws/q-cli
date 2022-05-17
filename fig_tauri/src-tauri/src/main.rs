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
use native::NativeState;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::debug;
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
}

pub struct GlobalState {
    pub debug_state: DebugState,
    pub figterm_state: FigtermState,
    pub intercept_state: InterceptState,
    pub native_state: NativeState,
    pub notifications_state: NotificationsState,
}

pub type WindowsState = DashMap<FigId, WindowState, FnvBuildHasher>;

struct WindowData {}

struct WebviewManager {
    fig_id_map: DashMap<FigId, Arc<WebView>, FnvBuildHasher>,
    window_id_map: DashMap<WindowId, Arc<WebView>, FnvBuildHasher>,
    event_loop: EventLoop<FigEvent>,
    global_state: Arc<GlobalState>,
    window_state_map: WindowsState,
}

#[derive(Debug)]
struct ApiRequest {
    fig_id: FigId,
    payload: String,
}

impl WebviewManager {
    fn new() -> Self {
        let (send, recv) = mpsc::unbounded_channel();
        Self {
            fig_id_map: Default::default(),
            window_id_map: Default::default(),
            event_loop: EventLoop::with_user_event(),
            global_state: Arc::new(GlobalState {
                debug_state: DebugState::default(),
                figterm_state: FigtermState::default(),
                intercept_state: InterceptState::default(),
                native_state: NativeState::new(send.clone()),
                notifications_state: NotificationsState::default(),
            }),
            window_state_map: Default::default(),
        }
    }

    fn insert_webview(&mut self, fig_id: FigId, webview: WebView) {
        let webview_arc = Arc::new(webview);
        self.fig_id_map.insert(fig_id, webview_arc.clone());
        self.window_id_map.insert(webview_arc.window().id(), webview_arc);
    }

    fn build_webview(
        &mut self,
        fig_id: FigId,
        builder: impl Fn(&EventLoop<FigEvent>) -> wry::Result<WebView>,
    ) -> wry::Result<()> {
        let webview = builder(&self.event_loop)?;
        self.insert_webview(fig_id, webview);
        Ok(())
    }

    async fn run(self) -> wry::Result<()> {
        let (api_handler_tx, mut api_handler_rx) = tokio::sync::mpsc::unbounded_channel::<ApiRequest>();
        let proxy = self.event_loop.create_proxy();

        // tokio::spawn(figterm::clean_figterm_cache(self.global_state.figterm_state.clone()));

        // let window_state = Arc::new(WindowState::new(&window, send));

        tokio::spawn(local_ipc::start_local_ipc(
            &self.global_state.figterm_state,
            notifications_state,
            window_state.clone(),
        ));

        tokio::spawn(async move {
            while let Some(ApiRequest { fig_id, payload }) = api_handler_rx.recv().await {
                api_request(
                    fig_id.clone(),
                    |event: String, payload: String| {
                        proxy
                            .send_event(FigEvent::WindowEvent {
                                fig_id,
                                window_event: FigWindowEvent::Emit { event, payload },
                            })
                            .unwrap();
                    },
                    payload,
                    &*self.global_state,
                    self.window_state_map,
                )
                .await;
            }
        });

        create_tray(&self.event_loop);

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                    ..
                } => {
                    if let Some(webview) = self.window_id_map.get(&window_id) {
                        webview.window().set_visible(false);
                    }
                },
                Event::UserEvent(event) => match event {
                    FigEvent::WindowEvent { fig_id, window_event } => match self.fig_id_map.get(&fig_id) {
                        Some(window) => {
                            window_event.handle(&window, state);
                        },
                        None => todo!(),
                    },
                },

                // match s {
                //    FigEvent::Api { fig_id, b64 } => tx.send((fig_id, b64)).unwrap(),
                //},
                _ => (),
            }
        });
    }
}

fn build_mission_control(event_loop: &EventLoop<FigEvent>) -> wry::Result<WebView> {
    let window = WindowBuilder::new()
        .with_title("Fig Mission Control")
        .with_always_on_top(true)
        .build(&event_loop)?;

    let proxy = event_loop.create_proxy();

    let webview = WebViewBuilder::new(window)?
        .with_url("http://localhost:3000")?
        .with_ipc_handler(move |_window, payload| {
            proxy
                .send_event(FigEvent::WindowEvent {
                    fig_id: MISSION_CONTROL_ID.clone(),
                    window_event: FigWindowEvent::Api { payload },
                })
                .unwrap();
        })
        .with_devtools(true)
        .with_initialization_script(JAVASCRIPT_INIT)
        .build()?;

    Ok(webview)
}

fn build_autocomplete(event_loop: &EventLoop<FigEvent>) -> wry::Result<WebView> {
    let window = WindowBuilder::new()
        .with_title("Fig Autocomplete")
        .with_always_on_top(true)
        .build(&event_loop)?;

    let proxy = event_loop.create_proxy();

    let webview = WebViewBuilder::new(window)?
        .with_url("http://localhost:3124")?
        .with_ipc_handler(move |_window, payload| {
            proxy
                .send_event(FigEvent::WindowEvent {
                    fig_id: AUTOCOMPLETE_ID.clone(),
                    window_event: FigWindowEvent::Api { payload },
                })
                .unwrap();
        })
        .with_devtools(true)
        .with_initialization_script(JAVASCRIPT_INIT)
        .build()?;

    Ok(webview)
}

#[tokio::main]
async fn main() {
    fig_log::init_logger("fig_tauri.log").expect("Failed to initialize logger");

    let mut webview_manager = WebviewManager::new();
    webview_manager
        .build_webview(MISSION_CONTROL_ID.into(), build_mission_control)
        .unwrap();
    webview_manager
        .build_webview(AUTOCOMPLETE_ID.into(), build_autocomplete)
        .unwrap();

    webview_manager.run().await.unwrap();

    //    tauri::Builder::default()
    //        .invoke_handler(tauri::generate_handler![api::handle_api_request])
    //        .setup({
    //            let figterm_state = figterm_state.clone();
    //            let notifications_state = notifications_state.clone();
    //            |app| {
    //                let window = app
    //                    .get_window("autocomplete")
    //                    .expect("Failed to acquire autocomplete window");
    //
    //                window.set_always_on_top(true).expect("Failed putting window on top");
    //                let window_state = Arc::new(WindowState::new(&window, send));
    //                app.manage(window_state.clone());
    //
    //
    // tauri::async_runtime::spawn(figterm::clean_figterm_cache(figterm_state.clone()));
    //
    //                tauri::async_runtime::spawn(local_ipc::start_local_ipc(
    //                    figterm_state,
    //                    notifications_state,
    //                    window_state.clone(),
    //                ));
    //
    //                tauri::async_runtime::spawn(window::handle_window(window, recv,
    // window_state));
    //
    //                Ok(())
    //            }
    //        })
    //        .plugin(constants_plugin())
    //        .system_tray(tray::create_tray())
    //        .on_system_tray_event({
    //            let debug_state = debug_state.clone();
    //            let figterm_state = figterm_state.clone();
    //            move |app, event| tray::handle_tray_event(app, event, debug_state.clone(),
    // figterm_state.clone())        })
    //        .register_uri_scheme_protocol("fig", icons::handle)
    //        .manage(debug_state)
    //        .manage(figterm_state)
    //        .manage(intercept_state)
    //        .manage(native_state)
    //        .manage(notifications_state)
    //        .run(tauri::generate_context!())
    //        .expect("error while running tauri application");
}
