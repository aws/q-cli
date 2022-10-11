use std::iter::empty;

use cfg_if::cfg_if;
use semver::Version;
use tracing::error;

const PREVIOUS_VERSION_KEY: &str = "desktop.versionAtPreviousLaunch";

/// Run items at launch
pub async fn run_install() {
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

    if should_run_install_script() {
        // Add any items that are only once per version
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
            use fig_install::{
                install,
                InstallComponents,
            };
            install(InstallComponents::DAEMON).await.ok();
        });

        cfg_if!(
            if #[cfg(target_os = "macos")] {
                initialize_fig_dir().ok();
            }
        );
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
        } else if #[cfg(target_os = "windows")] {
            // Update if there's a newer version
            if !cfg!(debug_assertions) {
                tokio::spawn(async {
                    let seconds = fig_settings::settings::get_int_or("autoupdate.check-period", 60 * 60 * 3);
                    if seconds < 0 {
                        return;
                    }
                    let mut interval = tokio::time::interval(std::time::Duration::from_secs(seconds as u64));
                    loop {
                        interval.tick().await;
                        fig_install::update(true).await.ok();
                    }
                });
            }

            // remove the updater if it exists
            std::fs::remove_file(fig_util::directories::fig_dir().unwrap().join("fig_installer.exe")).ok();
        }
    );
}

#[cfg(target_os = "macos")]
pub fn initialize_fig_dir() -> anyhow::Result<()> {
    use std::path::Path;
    use std::{
        fs,
        io,
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
    fs::remove_dir_all(&bin_dir).ok();
    fs::create_dir_all(&bin_dir).ok();
    fs::create_dir_all(fig_dir.join("apps")).ok();

    if let Some(resources) = get_bundle_resource_path() {
        let source = resources.join("config");
        let dest = fig_dir.join("config");
        copy_dir_all(&source, &dest).ok();
    }

    if let Some(figterm_path) = get_bundle_path_for_executable("figterm") {
        let dest = bin_dir.join("figterm");
        std::os::unix::fs::symlink(&figterm_path, &dest).ok();
    }

    if let Some(fig_cli_path) = get_bundle_path_for_executable("fig-darwin-universal") {
        let dest = bin_dir.join("fig");
        std::os::unix::fs::symlink(&fig_cli_path, &dest).ok();

        if let Ok(home) = home_dir() {
            let local_bin = home.join(".local").join("bin");
            fs::create_dir_all(&local_bin).ok();
            let dest = local_bin.join("fig");
            std::os::unix::fs::symlink(&fig_cli_path, &dest).ok();
        }
    }

    if let Some(bundle_path) = get_bundle_path() {
        let resources = bundle_path.join("Contents").join("Resources");
        let script_path = resources.join("config").join("tools").join("uninstaller.scpt");

        let uninstall_agent = LaunchdPlist::new("io.fig.uninstall")
            .program_arguments([
                "osascript",
                &script_path.to_string_lossy(),
                &bundle_path.to_string_lossy(),
            ])
            .watch_paths(["~/.Trash"])
            .keep_alive(false);

        create_launch_agent(&uninstall_agent)?;

        let path = uninstall_agent.get_file_path()?;
        std::process::Command::new("launchctl")
            .arg("load")
            .arg(&path)
            .status()?;
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
        .args(&["--user", "restart", service.service_name()])
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
            .args(&["--user", "is-active", "gnome-session-initialized.target"])
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

    current_version > previous_version
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
