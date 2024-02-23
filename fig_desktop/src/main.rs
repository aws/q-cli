mod cli;
mod event;
// mod figterm;
mod file_watcher;
mod install;
mod local_ipc;
pub mod notification_bus;
mod platform;
pub mod protocol;
mod remote_ipc;
mod request;
mod tray;
mod update;
mod utils;
mod webview;

use std::path::Path;
use std::process::exit;

use clap::Parser;
use event::Event;
use fig_log::Logger;
// use fig_telemetry::sentry::release_name;
use fig_util::consts::CODEWHISPERER_DESKTOP_PROCESS_NAME;
use parking_lot::RwLock;
use platform::PlatformState;
use sysinfo::{
    get_current_pid,
    ProcessRefreshKind,
    RefreshKind,
    System,
};
use tao::event_loop::{
    EventLoop as WryEventLoop,
    EventLoopProxy as WryEventLoopProxy,
    EventLoopWindowTarget as WryEventLoopWindowTarget,
};
use tracing::{
    error,
    warn,
};
use url::Url;
use webview::notification::WebviewNotificationsState;
use webview::{
    autocomplete,
    build_autocomplete,
    build_dashboard,
    dashboard,
    AutocompleteOptions,
    DashboardOptions,
    WebviewManager,
};
pub use webview::{
    AUTOCOMPLETE_ID,
    AUTOCOMPLETE_WINDOW_TITLE,
    DASHBOARD_ID,
};

// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

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

    // let _sentry_guard = fig_telemetry::init_sentry(
    //     release_name!(),
    //     "https://4295cb4f204845958717e406b331948d@o436453.ingest.sentry.io/6432682",
    //     1.0,
    //     true,
    // );

    if let Err(err) = fig_settings::settings::init_global() {
        error!(%err, "failed to init global settings");
        // fig_telemetry::sentry::capture_error(&err);
    }

    if cli.is_startup && !fig_settings::settings::get_bool_or("app.launchOnStartup", true) {
        std::process::exit(0);
    }

    let page = cli
        .url_link
        .and_then(|url| match Url::parse(&url) {
            Ok(url) => Some(url),
            Err(err) => {
                error!(%err, %url, "Failed to parse url");
                exit(1)
            },
        })
        .and_then(|url| {
            if url.scheme() != "codewhisperer" {
                error!(scheme = %url.scheme(), %url, "Invalid scheme");
                exit(1)
            }

            url.host_str().and_then(|s| match s {
                "dashboard" => Some(url.path().to_owned()),
                _ => {
                    error!("Invalid deep link");
                    None
                },
            })
        });

    if !cli.allow_multiple {
        match get_current_pid() {
            Ok(current_pid) => {
                let system = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
                let processes = system.processes_by_name(CODEWHISPERER_DESKTOP_PROCESS_NAME);

                cfg_if::cfg_if! {
                    if #[cfg(unix)] {
                        let current_user_id = Some(nix::unistd::getuid().as_raw());
                    } else {
                        let current_user_id = None;
                    }
                };

                for process in processes {
                    let pid = process.pid();
                    if current_pid != pid {
                        if cli.kill_old {
                            process.kill();
                            let exe = process.exe().unwrap_or(Path::new("")).display();
                            eprintln!("Killing instance: {exe} ({pid})");
                        } else {
                            let page = page.clone();
                            let on_match = async {
                                let exe = process.exe().unwrap_or(Path::new("")).display();

                                let mut extra = vec![format!("pid={pid}")];

                                if let Some(user_id) = process.user_id() {
                                    extra.push(format!("uid={}", **user_id));
                                }

                                if let Some(group_id) = process.group_id() {
                                    extra.push(format!("gid={}", *group_id));
                                }

                                eprintln!("CodeWhisperer is already running: {exe} ({})", extra.join(" "),);
                                match &page {
                                    Some(page) => {
                                        eprintln!("Opening /{page}...");
                                        Some(page)
                                    },
                                    None => {
                                        eprintln!("Opening CodeWhisperer Window...");
                                        None
                                    },
                                };

                                if let Err(err) =
                                    fig_ipc::local::open_ui_element(fig_proto::local::UiElement::MissionControl, page)
                                        .await
                                {
                                    eprintln!("Failed to open Fig: {err}");
                                }

                                exit(0);
                            };

                            match (process.user_id().map(|uid| uid as &u32), current_user_id.as_ref()) {
                                (Some(uid), Some(current_uid)) if uid == current_uid => {
                                    on_match.await;
                                },
                                (_, None) => {
                                    on_match.await;
                                },
                                _ => {},
                            }
                        }
                    }
                }
            },
            Err(err) => warn!(%err, "Failed to get pid"),
        }
    }

    #[cfg(target_os = "macos")]
    if let Ok(current_exe) = fig_util::current_exe_origin() {
        if let Ok(statvfs) = nix::sys::statvfs::statvfs(&current_exe) {
            if statvfs.flags().contains(nix::sys::statvfs::FsFlags::ST_RDONLY) {
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description(
                        "Cannot execute CodeWhisperer from within a readonly volume. Please move CodeWhisperer to your applications folder and try again.",
                    )
                    .show();

                return;
            }
        }
    }

    // tokio::spawn(async {
    //     fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
    //         fig_telemetry::TrackEventType::LaunchedApp,
    //         fig_telemetry::TrackSource::Desktop,
    //         env!("CARGO_PKG_VERSION").into(),
    //         empty::<(&str, &str)>(),
    //     ))
    //     .await
    //     .ok();
    // });

    install::run_install(cli.ignore_immediate_update).await;

    #[cfg(target_os = "linux")]
    {
        match std::env::var("CW_BACKEND").ok().as_deref() {
            Some("default") => {},
            Some(backend) => std::env::set_var("GDK_BACKEND", backend),
            None => std::env::set_var("GDK_BACKEND", "x11"),
        }

        platform::gtk::init().expect("Failed initializing GTK");
    }

    let is_logged_in = auth::is_logged_in().await;

    if !is_logged_in {
        tracing::info!("Showing onboarding");
    }

    let accessibility_enabled = PlatformState::accessibility_is_enabled().unwrap_or(true);
    let visible = !cli.no_dashboard;

    let autocomplete_enabled =
        !fig_settings::settings::get_bool_or("autocomplete.disable", false) && is_logged_in && accessibility_enabled;

    let mut webview_manager = WebviewManager::new(visible);
    webview_manager
        .build_webview(
            DASHBOARD_ID,
            build_dashboard,
            DashboardOptions {
                show_onboarding: !is_logged_in,
                visible,
                page,
            },
            true,
            dashboard::url,
        )
        .unwrap();
    webview_manager
        .build_webview(
            AUTOCOMPLETE_ID,
            build_autocomplete,
            AutocompleteOptions,
            autocomplete_enabled,
            autocomplete::url,
        )
        .unwrap();

    // webview_manager
    //     .build_webview(COMPANION_ID, build_companion, CompanionOptions, true, companion::url)
    //     .unwrap();

    webview_manager.run().await.unwrap();
    fig_telemetry::finish_telemetry().await;
}
