mod api;
mod cli;
mod event;
mod figterm;
mod file_watcher;
mod icons;
mod install;
mod local_ipc;
pub mod notification_bus;
mod platform;
mod secure_ipc;
mod tray;
mod utils;
mod webview;

use std::iter::empty;

use camino::Utf8PathBuf;
use clap::Parser;
use event::Event;
use fig_log::Logger;
use fig_telemetry::sentry::release_name;
use fig_util::consts::FIG_DESKTOP_PROCESS_NAME;
use parking_lot::RwLock;
use platform::PlatformState;
use sysinfo::{
    get_current_pid,
    ProcessExt,
    ProcessRefreshKind,
    RefreshKind,
    System,
    SystemExt,
};
use tracing::warn;
use url::Url;
use webview::notification::WebviewNotificationsState;
use webview::{
    build_autocomplete,
    build_dashboard,
    AutocompleteOptions,
    MissionControlOptions,
    WebviewManager,
};
pub use webview::{
    AUTOCOMPLETE_ID,
    AUTOCOMPLETE_WINDOW_TITLE,
    DASHBOARD_ID,
    FIG_PROTO_MESSAGE_RECEIVED,
};
use wry::application::event_loop::{
    EventLoop as WryEventLoop,
    EventLoopProxy as WryEventLoopProxy,
    EventLoopWindowTarget as WryEventLoopWindowTarget,
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
pub type EventLoopWindowTarget = WryEventLoopWindowTarget<Event>;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();

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

    let page = cli.url_link.and_then(|url| {
        let url = Url::parse(&url).unwrap();
        assert_eq!(url.scheme(), "fig");

        url.host_str().and_then(|s| match s {
            "dashboard" => Some(url.path().to_owned()),
            "plugins" => Some(format!("plugins/{}", url.path())),
            _ => {
                warn!("Invalid deep link");
                None
            },
        })
    });

    if !cli.allow_multiple {
        match get_current_pid() {
            Ok(current_pid) => {
                let system = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
                let processes = system.processes_by_name(FIG_DESKTOP_PROCESS_NAME);
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
                            match page {
                                Some(ref page) => eprintln!("Opening /{page}..."),
                                None => eprintln!("Opening Fig Window..."),
                            }
                            if let Err(err) =
                                fig_ipc::local::open_ui_element(fig_proto::local::UiElement::MissionControl, page).await
                            {
                                eprintln!("Failed to open Fig: {err}");
                            }
                            return;
                        }
                    }
                }
            },
            Err(err) => warn!("Failed to get pid: {err}"),
        }
    }

    #[cfg(target_os = "macos")]
    if let Some(true) = std::env::current_exe().ok().and_then(|bin| {
        bin.canonicalize().ok().and_then(|bin| {
            Utf8PathBuf::from_path_buf(bin)
                .ok()
                .map(|bin| bin.as_str().contains(".dmg"))
        })
    }) {
        eprintln!("Cannot execute Fig from within a DMG. Please move Fig to your applications folder and try again.");
        return;
    }

    tokio::spawn(async {
        fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
            fig_telemetry::TrackEventType::LaunchedApp,
            fig_telemetry::TrackSource::Desktop,
            env!("CARGO_PKG_VERSION").into(),
            empty::<(&str, &str)>(),
        ))
        .await
        .ok();
    });

    install::run_install().await;

    #[cfg(target_os = "linux")]
    {
        match std::env::var("FIG_BACKEND").ok().as_deref() {
            Some("default") => {},
            Some(backend) => std::env::set_var("GDK_BACKEND", backend),
            None => std::env::set_var("GDK_BACKEND", "x11"),
        }

        platform::gtk::init().expect("Failed initializing GTK");
    }

    let show_onboarding =
        !fig_settings::state::get_bool_or("desktop.completedOnboarding", false) || !fig_request::auth::is_logged_in();

    if show_onboarding {
        tracing::info!("Showing onboarding");
    }

    let autocomplete_enabled = !fig_settings::settings::get_bool_or("autocomplete.disable", false)
        && PlatformState::accessibility_is_enabled().unwrap_or(true)
        && fig_request::auth::is_logged_in();

    let mut webview_manager = WebviewManager::new();
    webview_manager
        .build_webview(
            DASHBOARD_ID,
            build_dashboard,
            MissionControlOptions {
                show_onboarding,
                force_visible: !cli.no_dashboard || page.is_some(),
                page,
            },
            true,
        )
        .unwrap();
    webview_manager
        .build_webview(
            AUTOCOMPLETE_ID,
            build_autocomplete,
            AutocompleteOptions {},
            autocomplete_enabled,
        )
        .unwrap();
    webview_manager.run().await.unwrap();
}
