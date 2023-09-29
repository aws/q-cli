mod cli;
mod event;
mod figterm;
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

use std::iter::empty;
use std::process::exit;

use clap::Parser;
use event::Event;
use fig_log::Logger;
use fig_telemetry::sentry::release_name;
use fig_util::consts::CODEWHISPERER_DESKTOP_PROCESS_NAME;
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
    FIG_PROTO_MESSAGE_RECEIVED,
};
use wry::application::event_loop::{
    EventLoop as WryEventLoop,
    EventLoopProxy as WryEventLoopProxy,
    EventLoopWindowTarget as WryEventLoopWindowTarget,
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

    let _sentry_guard = fig_telemetry::init_sentry(
        release_name!(),
        "https://4295cb4f204845958717e406b331948d@o436453.ingest.sentry.io/6432682",
        1.0,
        true,
    );

    if let Err(err) = fig_settings::settings::init_global() {
        error!(%err, "failed to init global settings");
        fig_telemetry::sentry::capture_error(&err);
    }

    if cli.is_startup && !fig_settings::settings::get_bool_or("app.launchOnStartup", true) {
        std::process::exit(0);
    }

    let page_and_data = cli
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
                "dashboard" => Some((url.path().to_owned(), None)),
                "plugins" => Some((format!("plugins/{}", url.path()), None)),
                "login" => {
                    let auth = utils::handle_login_deep_link(&url);
                    Some(("login".into(), auth))
                },
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
                            let exe = process.exe().display();
                            eprintln!("Killing instance: {exe} ({pid})");
                        } else {
                            let page_and_data = page_and_data.clone();
                            let on_match = async {
                                let exe = process.exe().display();

                                let mut extra = vec![format!("pid={pid}")];

                                if let Some(user_id) = process.user_id() {
                                    extra.push(format!("uid={}", **user_id));
                                }

                                if let Some(group_id) = process.group_id() {
                                    extra.push(format!("gid={}", *group_id));
                                }

                                eprintln!("CodeWhisperer is already running: {exe} ({})", extra.join(" "),);
                                let (page, auth_data) = match page_and_data {
                                    Some((page, auth_data)) => {
                                        eprintln!("Opening /{page}...");
                                        (Some(page), auth_data)
                                    },
                                    None => {
                                        eprintln!("Opening CodeWhisperer Window...");
                                        (None, None)
                                    },
                                };

                                if let Err(err) =
                                    fig_ipc::local::open_ui_element(fig_proto::local::UiElement::MissionControl, page)
                                        .await
                                {
                                    eprintln!("Failed to open Fig: {err}");
                                }

                                if let Some(auth) = auth_data {
                                    eprintln!("Sending auth: {auth}");
                                    if let Err(err) = fig_ipc::local::send_hook_to_socket(
                                        fig_proto::hooks::new_event_hook("dashboard.login", auth.to_string(), [
                                            "dashboard".to_owned(),
                                        ]),
                                    )
                                    .await
                                    {
                                        eprintln!("Failed to send auth: {err}");
                                    }
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

    let show_onboarding = !fig_request::auth::is_logged_in();

    if show_onboarding {
        tracing::info!("Showing onboarding");
    }

    let accessibility_enabled = PlatformState::accessibility_is_enabled().unwrap_or(true);
    let visible = !cli.no_dashboard;

    let autocomplete_enabled = !fig_settings::settings::get_bool_or("autocomplete.disable", false)
        && fig_request::auth::is_logged_in()
        && accessibility_enabled;

    let mut webview_manager = WebviewManager::new(visible);
    webview_manager
        .build_webview(
            DASHBOARD_ID,
            build_dashboard,
            DashboardOptions {
                show_onboarding,
                visible,
                page: page_and_data.map(|p| p.0),
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
}

/// Temp function to migrate existing users of Swift macOS app to new Rust app
#[cfg(target_os = "macos")]
async fn migrate() {
    use fig_install::uninstall_terminal_integrations;
    use fig_util::directories::home_dir;
    use macos_utils::{
        NSArrayRef,
        NSStringRef,
    };
    use objc::runtime::Object;
    use tracing::debug;

    fig_settings::state::remove_value("NEW_VERSION_AVAILABLE").ok();

    if let Ok(home) = home_dir() {
        for path in &[
            "Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.py",
            ".config/iterm2/AppSupport/Scripts/AutoLaunch/fig-iterm-integration.py",
            "Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.scpt",
        ] {
            tokio::fs::remove_file(home.join(path))
                .await
                .map_err(|err| warn!("Could not remove iTerm integration {path}: {err}"))
                .ok();
        }
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
}
