#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod api;
mod figterm;
mod icons;
mod local_ipc;
mod native;
mod tray;
mod utils;
mod window;

use std::sync::Arc;

use dashmap::DashMap;
use fig_proto::fig::NotificationType;
use figterm::FigtermState;
use native::NativeState;
use parking_lot::RwLock;
use tauri::plugin::{
    Builder,
    TauriPlugin,
};
use tauri::{
    Manager,
    Runtime,
    Window,
    WindowUrl,
};
use tokio::sync::mpsc;
use url::Url;
use window::WindowState;
use wry::application::dpi::PhysicalSize;
use wry::http::{
    Response,
    ResponseBuilder,
};

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
    subscriptions: DashMap<NotificationType, i64, fnv::FnvBuildHasher>,
}

fn constants_plugin<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("constants")
        .js_init_script(JAVASCRIPT_INIT.to_string())
        .build()
}

fn spawn_mission_control() -> wry::Result<()> {
    use wry::application::event::{
        Event,
        StartCause,
        WindowEvent,
    };
    use wry::application::event_loop::{
        ControlFlow,
        EventLoop,
    };
    use wry::application::window::WindowBuilder;
    use wry::webview::WebViewBuilder;

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Fig Mission Control")
        .with_always_on_top(true)
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new(window)?
        .with_url("https://app.fig.io")?
        .with_devtools(true)
        .with_custom_protocol("figipc".into(), |request| {
            println!("{:?}", request.headers().into_iter().collect::<Vec<_>>());
            println!("{request:?}");
            ResponseBuilder::new().status(200).body(b"OK".to_vec())
        })
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}

fn main() {
    fig_log::init_logger("fig_tauri.log").expect("Failed to initialize logger");

    spawn_mission_control().unwrap();

    let (send, recv) = mpsc::unbounded_channel();

    let debug_state = Arc::new(DebugState::default());
    let figterm_state = Arc::new(FigtermState::default());
    let intercept_state = InterceptState::default();
    let native_state = NativeState::new(send.clone());
    let notifications_state = Arc::new(NotificationsState::default());

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![api::handle_api_request])
        .setup({
            let figterm_state = figterm_state.clone();
            let notifications_state = notifications_state.clone();
            |app| {
                let window = app
                    .get_window("autocomplete")
                    .expect("Failed to acquire autocomplete window");

                // Window::builder(
                //    app,
                //    "mission-control",
                //    WindowUrl::External(Url::parse("https://app.fig.io").unwrap()),
                //)
                //.build()
                //.unwrap();

                window.set_always_on_top(true).expect("Failed putting window on top");
                let window_state = Arc::new(WindowState::new(&window, send));
                app.manage(window_state.clone());

                tauri::async_runtime::spawn(figterm::clean_figterm_cache(figterm_state.clone()));

                tauri::async_runtime::spawn(local_ipc::start_local_ipc(
                    figterm_state,
                    notifications_state,
                    window_state.clone(),
                ));

                tauri::async_runtime::spawn(window::handle_window(window, recv, window_state));

                Ok(())
            }
        })
        .plugin(constants_plugin())
        .system_tray(tray::create_tray())
        .on_system_tray_event({
            let debug_state = debug_state.clone();
            let figterm_state = figterm_state.clone();
            move |app, event| tray::handle_tray_event(app, event, debug_state.clone(), figterm_state.clone())
        })
        .register_uri_scheme_protocol("fig", icons::handle)
        .manage(debug_state)
        .manage(figterm_state)
        .manage(intercept_state)
        .manage(native_state)
        .manage(notifications_state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
