use std::iter::empty;

use cfg_if::cfg_if;
use fig_install::check_for_updates;
use fig_integrations::Integration;
use fig_util::directories;
use semver::Version;
use tracing::{
    error,
    info,
};

use crate::utils::is_cargo_debug_build;

const PREVIOUS_VERSION_KEY: &str = "desktop.versionAtPreviousLaunch";

/// Run items at launch
pub async fn run_install(ignore_immediate_update: bool) {
    // Create files needed for other parts of the app to run
    for (path_result, name, default) in [
        (fig_util::directories::settings_path(), "settings", "{}"),
        (fig_util::directories::state_path(), "state", "{}"),
    ] {
        match path_result {
            Ok(path) => {
                if let Some(path_parent) = path.parent() {
                    if !path_parent.exists() {
                        if let Err(err) = std::fs::create_dir_all(path_parent) {
                            error!(%err, "Failed to create {name} directory");
                        }
                    }
                }
                if !path.exists() {
                    if let Err(err) = std::fs::write(&path, default) {
                        error!(%err, "Failed to create {name} file");
                    }
                }
            },
            Err(err) => error!(%err, "Failed to get {name} path"),
        }
    }

    tokio::spawn(async {
        if let Err(err) = fig_sync::themes::clone_or_update().await {
            error!(%err, "Failed to clone or update themes");
        }
    });

    tokio::spawn(async {
        if let Err(err) = fig_sync::plugins::fetch_installed_plugins(false).await {
            error!(%err, "Failed to fetch installed plugins");
        }
    });

    tokio::spawn(async {
        if let Err(err) = fig_sync::dotfiles::download_and_notify(false).await {
            error!(%err, "Failed to download installed plugins");
        }
    });

    #[cfg(target_os = "macos")]
    initialize_fig_dir().ok();

    // Add any items that are only once per version
    if should_run_install_script() {
        tokio::spawn(async {
            fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
                fig_telemetry::TrackEventType::UpdatedApp,
                fig_telemetry::TrackSource::Desktop,
                env!("CARGO_PKG_VERSION").into(),
                empty::<(&str, &str)>(),
            ))
            .await
            .ok()
        });

        tokio::spawn(async {
            match directories::relative_cli_path() {
                Ok(cli_path) => {
                    if let Err(err) = fig_daemon::Daemon::default().install(&cli_path).await {
                        error!(%err, "Failed to install daemon");
                    };
                },
                Err(err) => error!(%err, "Failed to get CLI path"),
            }
        });

        if let Ok(target_bundle_path) = fig_integrations::input_method::InputMethod::default().target_bundle_path() {
            if target_bundle_path.exists() {
                if let Err(err) = fig_integrations::input_method::InputMethod::register(target_bundle_path) {
                    error!(%err, "Input method could not be registered");
                }
            }
        }

        // Delete old figterm instances
        #[cfg(target_os = "macos")]
        if let Ok(fig_dir) = directories::fig_dir() {
            let bins = fig_dir.join("bin");
            for entry in std::fs::read_dir(bins).ok().into_iter().flatten().flatten() {
                if entry.file_type().map_or(false, |f| f.is_file()) {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.contains("figterm") {
                            if let Err(err) = std::fs::remove_file(entry.path()) {
                                error!(%err, "Failed to delete old figterm instance");
                            }
                        }
                    }
                }
            }
        }
    }

    if let Err(err) = set_previous_version(current_version()) {
        error!(%err, "Failed to set previous version");
    }

    cfg_if!(
        if #[cfg(target_os = "linux")] {
            // todo(mia): make this part of onboarding
            tokio::spawn(async {
                use sysinfo::{
                    ProcessRefreshKind,
                    SystemExt,
                };
                let mut s = sysinfo::System::new();
                s.refresh_processes_specifics(ProcessRefreshKind::new());
                if s.processes_by_exact_name("/usr/bin/gnome-shell").next().is_some() {
                    drop(s);
                    match dbus::gnome_shell::has_extension().await {
                        Ok(true) => tracing::debug!("shell extension already installed"),
                        Ok(false) => {
                            if let Err(err) = dbus::gnome_shell::install_extension().await {
                                error!(%err, "Failed to install shell extension")
                            }
                        },
                        Err(err) => error!(%err, "Failed to check shell extensions"),
                    }
                }
            });

            // Has to be at the end of this function -- will block until ibus has launched.
            launch_ibus().await;
        } else {
            // Update if there's a newer version
            if !ignore_immediate_update && !is_cargo_debug_build() {
                use std::time::Duration;
                use tokio::time::timeout;
                // Check for updates but timeout after 3 seconds to avoid making the user wait too long
                // todo: don't download the index file twice
                match timeout(Duration::from_secs(3), check_for_updates(true)).await {
                    Ok(Ok(Some(_))) => { crate::update::check_for_update(true, true).await; },
                    Ok(Ok(None)) => error!("No update found"),
                    Ok(Err(err)) => error!(%err, "Failed to check for updates"),
                    Err(err) => error!(%err, "Update check timed out"),
                }

            }

            tokio::spawn(async {
                let seconds = fig_settings::settings::get_int_or("app.autoupdate.check-period", 60 * 60 * 3);
                if seconds < 0 {
                    return;
                }
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(seconds as u64));
                interval.tick().await;
                loop {
                    interval.tick().await;
                    // TODO: we need to determine if the dashboard is open here and pass that as the second bool
                    crate::update::check_for_update(false, false).await;
                }
            });

            // remove the updater if it exists
            #[cfg(target_os = "windows")]
            std::fs::remove_file(fig_util::directories::fig_dir().unwrap().join("fig_installer.exe")).ok();
        }
    );

    // install vscode
    for variant in fig_integrations::vscode::variants_installed() {
        let integration = fig_integrations::vscode::VSCodeIntegration { variant };
        if integration.is_installed().await.is_err() {
            info!(
                "Attempting to install vscode integration for variant {}",
                integration.variant.application_name
            );
            if let Err(err) = integration.install().await {
                error!(%err, "Failed installing vscode integration for variant {}", integration.variant.application_name);
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub fn initialize_fig_dir() -> anyhow::Result<()> {
    use std::path::Path;
    use std::{
        fs,
        io,
    };

    use fig_util::consts::{
        FIG_BUNDLE_ID,
        FIG_CLI_BINARY_NAME,
        FIG_DESKTOP_PROCESS_NAME,
    };
    use fig_util::directories::{
        fig_dir,
        home_dir,
    };
    use fig_util::launchd_plist::{
        create_launch_agent,
        LaunchdPlist,
    };
    use macos_accessibility_position::bundle::{
        get_bundle_path,
        get_bundle_path_for_executable,
        get_bundle_resource_path,
    };

    fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    let fig_dir = fig_dir()?;
    let bin_dir = fig_dir.join("bin");
    fs::create_dir_all(&bin_dir).ok();
    fs::create_dir_all(fig_dir.join("apps")).ok();

    if let Some(resources) = get_bundle_resource_path() {
        let source = resources.join("config");
        let dest = fig_dir.join("config");
        copy_dir_all(source, dest).ok();
    }

    if let Some(figterm_path) = get_bundle_path_for_executable("figterm") {
        let dest = bin_dir.join("figterm");
        std::os::unix::fs::symlink(figterm_path, dest).ok();
    }

    if let Some(fig_cli_path) = get_bundle_path_for_executable(FIG_CLI_BINARY_NAME) {
        let dest = bin_dir.join("fig");
        std::os::unix::fs::symlink(&fig_cli_path, dest).ok();

        if let Ok(home) = home_dir() {
            let local_bin = home.join(".local").join("bin");
            fs::create_dir_all(&local_bin).ok();
            let dest = local_bin.join("fig");
            std::os::unix::fs::symlink(&fig_cli_path, dest).ok();
        }
    }

    if let Some(bundle_path) = get_bundle_path() {
        let exe = bundle_path
            .join("Contents")
            .join("MacOS")
            .join(FIG_DESKTOP_PROCESS_NAME);
        let startup_launch_agent = LaunchdPlist::new("io.fig.launcher")
            .program_arguments([&exe.to_string_lossy(), "--is-startup", "--no-dashboard"])
            .associated_bundle_identifiers([FIG_BUNDLE_ID])
            .run_at_load(true);

        create_launch_agent(&startup_launch_agent)?;

        let path = startup_launch_agent.get_file_path()?;
        std::process::Command::new("launchctl")
            .arg("load")
            .arg(&path)
            .status()
            .ok();
    }

    if let Ok(home) = home_dir() {
        let iterm_integration_path = home
            .join("Library")
            .join("Application Support")
            .join("iTerm2")
            .join("Scripts")
            .join("AutoLaunch")
            .join("fig-iterm-integration.scpt");

        if iterm_integration_path.exists() {
            std::fs::remove_file(&iterm_integration_path).ok();
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
enum SystemdUserService {
    IBusGeneric,
    IBusGnome,
}

#[cfg(target_os = "linux")]
impl SystemdUserService {
    fn service_name(&self) -> &'static str {
        match self {
            SystemdUserService::IBusGeneric => "org.freedesktop.IBus.session.generic.service",
            SystemdUserService::IBusGnome => "org.freedesktop.IBus.session.GNOME.service",
        }
    }
}

#[cfg(target_os = "linux")]
impl std::fmt::Display for SystemdUserService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.service_name())
    }
}

#[cfg(target_os = "linux")]
async fn launch_systemd_user_service(service: SystemdUserService) -> anyhow::Result<()> {
    use tokio::process::Command;
    let output = Command::new("systemctl")
        .args(["--user", "restart", service.service_name()])
        .output()
        .await?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr))
    }
    Ok(())
}

#[cfg(target_os = "linux")]
async fn launch_ibus() {
    use sysinfo::{
        ProcessRefreshKind,
        RefreshKind,
        System,
        SystemExt,
    };
    use tokio::process::Command;
    use tracing::info;

    let system = tokio::task::block_in_place(|| {
        System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()))
    });
    if system.processes_by_name("ibus-daemon").next().is_none() {
        info!("Launching ibus via systemd");

        match Command::new("systemctl")
            .args(["--user", "is-active", "gnome-session-initialized.target"])
            .output()
            .await
        {
            Ok(gnome_session_output) => match std::str::from_utf8(&gnome_session_output.stdout).map(|s| s.trim()) {
                Ok("active") => match launch_systemd_user_service(SystemdUserService::IBusGnome).await {
                    Ok(_) => info!("Launched '{}", SystemdUserService::IBusGnome),
                    Err(err) => error!(%err, "Failed to launch '{}'", SystemdUserService::IBusGnome),
                },
                Ok("inactive") => match launch_systemd_user_service(SystemdUserService::IBusGeneric).await {
                    Ok(_) => info!("Launched '{}'", SystemdUserService::IBusGeneric),
                    Err(err) => error!(%err, "Failed to launch '{}'", SystemdUserService::IBusGeneric),
                },
                result => error!(
                    ?result,
                    "Failed to determine if gnome-session-initialized.target is running"
                ),
            },
            Err(err) => error!(%err, "Failed to run 'systemctl --user is-active gnome-session-initialized.target'"),
        }
    }

    // Wait up to 2 sec for ibus activation
    for _ in 0..10 {
        if dbus::ibus::ibus_address().await.is_ok() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    error!("Timed out after 2 sec waiting for ibus activation");
}

fn should_run_install_script() -> bool {
    let current_version = current_version();
    let previous_version = match previous_version() {
        Some(previous_version) => previous_version,
        None => return true,
    };

    !is_cargo_debug_build() && current_version > previous_version
}

/// The current version of the desktop app
fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
}

/// The previous version of the desktop app stored in local state
fn previous_version() -> Option<Version> {
    fig_settings::state::get_string(PREVIOUS_VERSION_KEY)
        .ok()
        .flatten()
        .and_then(|ref v| Version::parse(v).ok())
}

fn set_previous_version(version: Version) -> anyhow::Result<()> {
    fig_settings::state::set_value(PREVIOUS_VERSION_KEY, version.to_string())?;
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn current_version() {
        super::current_version();
    }
}
