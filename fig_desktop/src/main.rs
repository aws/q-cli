#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod api;
mod cli;
mod event;
mod figterm;
mod icons;
mod install;
mod local_ipc;
mod native;
mod notification;
mod settings;
mod tray;
mod utils;
mod webview;
mod window;

use std::iter::empty;
use std::time::Duration;

use clap::Parser;
use event::Event;
use fig_log::Logger;
use fig_telemetry::sentry::release_name;
use figterm::FigtermState;
use native::NativeState;
use notification::NotificationsState;
use parking_lot::RwLock;
use sysinfo::{
    get_current_pid,
    ProcessExt,
    ProcessRefreshKind,
    RefreshKind,
    System,
    SystemExt,
};
use tracing::warn;
use webview::{
    build_autocomplete,
    build_mission_control,
    AutocompleteOptions,
    MissionControlOptions,
    WebviewManager,
};
pub use webview::{
    AUTOCOMPLETE_ID,
    FIG_PROTO_MESSAGE_RECIEVED,
    MISSION_CONTROL_ID,
};
use wry::application::event_loop::{
    EventLoop as WryEventLoop,
    EventLoopProxy as WryEventLoopProxy,
};

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

#[tokio::main]
async fn main() {
    let _logger_guard = Logger::new()
        .with_stdout()
        .with_file("fig_desktop.log")
        .init()
        .expect("Failed to init logger");

    let _sentry_guard = fig_telemetry::init_sentry(
        release_name!(),
        "https://4295cb4f204845958717e406b331948d@o436453.ingest.sentry.io/6432682",
        1.0,
        true,
    );

    utils::update_check().await;

    tokio::spawn(async {
        let seconds = fig_settings::settings::get_int_or("autoupdate.checkPeriod", 60 * 60 * 3);
        if seconds < 0 {
            return;
        }
        let mut interval = tokio::time::interval(Duration::from_secs(seconds as u64));
        interval.tick().await; // first tick is completed immediately
        loop {
            interval.tick().await;
            utils::update_check().await;
        }
    });

    let cli = cli::Cli::parse();

    if let Some(url) = cli.url_link {
        println!("Opening {url}");
        return;
    }

    if !cli.allow_multiple {
        match get_current_pid() {
            Ok(current_pid) => {
                let system = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
                let processes = system.processes_by_exact_name("fig_desktop");
                for process in processes {
                    let pid = process.pid();
                    if current_pid != pid {
                        if cli.kill_old {
                            process.kill();
                            let exe = process.exe().display();
                            eprintln!("Killing instance: {exe} ({pid})");
                        } else {
                            let exe = process.exe().display();
                            eprintln!("Fig is already running: {exe} ({pid})");
                            eprintln!("Opening Fig Window...");
                            fig_ipc::command::open_ui_element(fig_proto::local::UiElement::MissionControl, None)
                                .await
                                .unwrap();
                            return;
                        }
                    }
                }
            },
            Err(err) => warn!("Failed to get pid: {err}"),
        }
    }

    tokio::spawn(async {
        fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
            fig_telemetry::TrackEventType::LaunchedApp,
            fig_telemetry::TrackSource::App,
            empty::<(&str, &str)>(),
        ))
        .await
        .ok();
    });

    install::run_install().await;

    let show_onboarding =
        std::env::consts::OS != "windows" && !fig_settings::state::get_bool_or("desktop.completedOnboarding", false);

    #[cfg(target_os = "linux")]
    gtk::init().expect("Failed initializing GTK");

    let mut webview_manager = WebviewManager::new();
    webview_manager
        .build_webview(MISSION_CONTROL_ID, build_mission_control, MissionControlOptions {
            show_onboarding,
            force_visible: cli.mission_control,
        })
        .unwrap();
    webview_manager
        .build_webview(AUTOCOMPLETE_ID, build_autocomplete, AutocompleteOptions {})
        .unwrap();
    webview_manager.run().await.unwrap();
}
