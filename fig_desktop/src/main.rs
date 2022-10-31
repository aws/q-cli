mod cli;
mod event;
mod figterm;
mod file_watcher;
mod icons;
mod install;
mod local_ipc;
pub mod notification_bus;
mod platform;
mod request;
mod secure_ipc;
mod tray;
mod update;
mod utils;
mod webview;

use std::iter::empty;
use std::process::exit;

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
use tracing::{
    error,
    warn,
};
use url::Url;
use webview::notification::WebviewNotificationsState;
use webview::{
    build_autocomplete,
    build_dashboard,
    AutocompleteOptions,
    DashboardOptions,
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

    let _sentry_guard = fig_telemetry::init_sentry(
        release_name!(),
        "https://4295cb4f204845958717e406b331948d@o436453.ingest.sentry.io/6432682",
        1.0,
        true,
    );

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
            if url.scheme() != "fig" {
                error!(scheme = %url.scheme(), %url, "Invalid scheme");
                exit(1)
            }

            url.host_str().and_then(|s| match s {
                "dashboard" => Some(url.path().to_owned()),
                "plugins" => Some(format!("plugins/{}", url.path())),
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
                        "Cannot execute Fig from within a readonly volume. Please move Fig to your applications folder and try again.",
                    )
                    .show();
            }

            return;
        }
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

    #[cfg(target_os = "macos")]
    migrate().await;

    install::run_install(cli.ignore_immediate_update).await;

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

    let accessibility_enabled = PlatformState::accessibility_is_enabled().unwrap_or(true);

    let autocomplete_enabled = !fig_settings::settings::get_bool_or("autocomplete.disable", false)
        && fig_request::auth::is_logged_in()
        && accessibility_enabled;

    let mut webview_manager = WebviewManager::new();
    webview_manager
        .build_webview(
            DASHBOARD_ID,
            build_dashboard,
            DashboardOptions {
                show_onboarding,
                force_visible: !cli.no_dashboard || page.is_some() || !accessibility_enabled,
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

/// Temp function to migrate existing users of Swift macOS app to new Rust app
#[cfg(target_os = "macos")]
async fn migrate() {
    use fig_install::uninstall_terminal_integrations;
    use macos_accessibility_position::{
        NSArrayRef,
        NSStringRef,
    };
    use objc::runtime::Object;
    use tracing::debug;

    fig_settings::state::remove_value("NEW_VERSION_AVAILABLE").ok();

    match fig_request::defaults::get_default("userEmail") {
        Ok(user) if user.is_empty() => {
            fig_request::defaults::remove_default("userEmail").ok();
            return;
        },
        Err(_) => return,
        _ => {},
    }

    // Set user as having completed onboarding
    fig_settings::state::set_value("desktop.completedOnboarding", true).ok();

    // Remove the old LaunchAgents
    if let Ok(home) = fig_util::directories::home_dir() {
        for file in ["io.fig.launcher.plist", "io.fig.uninstall.plist"] {
            let path = home.join("Library").join("LaunchAgents").join(file);
            if path.exists() {
                std::fs::remove_file(path).ok();
            }
        }
    }

    // Uninstall terminal integrations
    uninstall_terminal_integrations().await;

    // Kill the old input method
    let shared: *mut Object = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
    let running_app: NSArrayRef<*mut Object> = unsafe { msg_send![shared, runningApplications] };

    debug!("attempting to kill the old input method");
    running_app
        .iter()
        .filter(|app| {
            let name: NSStringRef = unsafe { msg_send![**app as *mut Object, bundleIdentifier] };
            tracing::trace!("found {:?} within running apps", name.as_str());
            name.as_str() == Some("io.fig.cursor")
        })
        .for_each(|app| {
            let _: () = unsafe { msg_send![*app as *mut Object, terminate] };
        });

    fig_request::defaults::remove_default("userEmail").ok();
}
