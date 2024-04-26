use cfg_if::cfg_if;
#[cfg(not(target_os = "linux"))]
use fig_install::check_for_updates;
use fig_integrations::ssh::SshIntegration;
use fig_integrations::Integration;
use fig_util::directories::fig_data_dir;
#[cfg(target_os = "macos")]
use macos_utils::bundle::get_bundle_path_for_executable;
use semver::Version;
use tracing::{
    error,
    info,
};

use crate::utils::is_cargo_debug_build;

const PREVIOUS_VERSION_KEY: &str = "desktop.versionAtPreviousLaunch";
const MIGRATED_KEY: &str = "desktop.migratedFromFig";

#[cfg(target_os = "macos")]
pub async fn migrate_data_dir() {
    // Migrate the user data dir
    if let (Ok(old), Ok(new)) = (fig_util::directories::old_fig_data_dir(), fig_data_dir()) {
        if !old.is_symlink() && old.is_dir() && !new.is_dir() {
            match tokio::fs::rename(&old, &new).await {
                Ok(()) => {
                    if let Err(err) = symlink(&new, &old).await {
                        error!(%err, "Failed to symlink old user data dir");
                    }
                },
                Err(err) => {
                    error!(%err, "Failed to migrate user data dir");
                },
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn run_input_method_migration() {
    use fig_integrations::input_method::InputMethod;
    use tokio::time::{
        sleep,
        Duration,
    };
    use tracing::warn;

    let input_method = InputMethod::default();
    match input_method.target_bundle_path() {
        Ok(target_bundle_path) if target_bundle_path.exists() => {
            tokio::spawn(async move {
                input_method.terminate().ok();
                if let Err(err) = input_method.migrate().await {
                    warn!(%err, "Failed to migrate input method");
                }

                sleep(Duration::from_secs(1)).await;
                input_method.launch();
            });
        },
        Ok(_) => warn!("Input method bundle path does not exist"),
        Err(err) => warn!(%err, "Failed to get input method bundle path"),
    }
}

/// Run items at launch
pub async fn run_install(_ignore_immediate_update: bool) {
    #[cfg(not(target_os = "linux"))]
    let ignore_immediate_update = _ignore_immediate_update;

    #[cfg(target_os = "macos")]
    {
        initialize_fig_dir().await.ok();

        if fig_util::directories::home_dir()
            .map(|home| home.join("Library/Application Support/fig/credentials.json"))
            .is_ok_and(|path| path.exists())
            && !fig_settings::state::get_bool_or(MIGRATED_KEY, false)
        {
            let set = fig_settings::state::set_value(MIGRATED_KEY, true);
            if set.is_ok() {
                fig_telemetry::send_fig_user_migrated().await;
            }
        }
    }

    #[cfg(target_os = "macos")]
    // Add any items that are only once per version
    if should_run_install_script() {
        run_input_method_migration();
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
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
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

    // install vscode integration
    #[cfg(target_os = "macos")]
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

    // install intellij integration
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    match fig_integrations::intellij::variants_installed().await {
        Ok(variants) => {
            for integration in variants {
                if integration.is_installed().await.is_err() {
                    info!(
                        "Attempting to install intellij integration for variant {}",
                        integration.variant.application_name()
                    );
                    if let Err(err) = integration.install().await {
                        error!(%err, "Failed installing intellij integration for variant {}", integration.variant.application_name());
                    }
                }
            }
        },
        Err(err) => error!(%err, "Failed getting installed intellij variants"),
    }

    // update ssh integration
    if let Ok(ssh_integration) = SshIntegration::new() {
        if let Err(err) = ssh_integration.reinstall().await {
            error!(%err, "Failed updating ssh integration");
        }
    }
}

/// Symlink, and overwrite if it already exists and is invalid or not a symlink
#[cfg(target_os = "macos")]
async fn symlink(src: impl AsRef<std::path::Path>, dst: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
    use std::io::ErrorKind;

    let src = src.as_ref();
    let dst = dst.as_ref();

    // Check if the link already exists
    match tokio::fs::symlink_metadata(dst).await {
        Ok(metadata) => {
            // If it's a symlink, check if it points to the right place
            if metadata.file_type().is_symlink() {
                if let Ok(read_link) = tokio::fs::read_link(dst).await {
                    if read_link == src {
                        return Ok(());
                    }
                }
            }

            // If it's not a symlink or it points to the wrong place, delete it
            tokio::fs::remove_file(dst).await?;
        },
        Err(err) if err.kind() == ErrorKind::NotFound => {},
        Err(err) => return Err(err),
    }

    // Create the symlink
    tokio::fs::symlink(src, dst).await
}

#[cfg(target_os = "macos")]
pub async fn initialize_fig_dir() -> anyhow::Result<()> {
    use std::fs;

    use fig_integrations::shell::ShellExt;
    use fig_util::consts::{
        APP_BUNDLE_ID,
        APP_PROCESS_NAME,
        CLI_BINARY_NAME,
        PTY_BINARY_NAME,
    };
    use fig_util::directories::home_dir;
    use fig_util::launchd_plist::{
        create_launch_agent,
        LaunchdPlist,
    };
    use fig_util::{
        Shell,
        OLD_CLI_BINARY_NAME,
        OLD_PTY_BINARY_NAME,
    };
    use macos_utils::bundle::get_bundle_path;
    use tracing::warn;

    let local_bin = fig_util::directories::home_local_bin()?;
    if let Err(err) = fs::create_dir_all(&local_bin) {
        error!(%err, "Failed to create {local_bin:?}");
    }

    // Install figterm to ~/.local/bin
    match get_bundle_path_for_executable(PTY_BINARY_NAME) {
        Some(pty_path) => {
            let link = local_bin.join(PTY_BINARY_NAME);
            if let Err(err) = symlink(&pty_path, link).await {
                error!(%err, "Failed to symlink for {PTY_BINARY_NAME}: {pty_path:?}");
            }

            let legacy_pty_link = local_bin.join(OLD_PTY_BINARY_NAME);
            if legacy_pty_link.exists() {
                if let Err(err) = fs::remove_file(&legacy_pty_link) {
                    warn!(%err, "Failed to remove {OLD_PTY_BINARY_NAME}: {legacy_pty_link:?}");
                }
            }

            for shell in Shell::all() {
                let pty_shell_cpy = local_bin.join(format!("{shell} ({PTY_BINARY_NAME})"));
                let pty_path = pty_path.clone();

                tokio::spawn(async move {
                    // Check version if copy already exists, this is because everytime a copy is made the first start is
                    // kinda slow and we want to avoid that
                    if pty_shell_cpy.exists() {
                        let output = tokio::process::Command::new(&pty_shell_cpy)
                            .arg("--version")
                            .output()
                            .await
                            .ok();

                        let version = output
                            .as_ref()
                            .and_then(|output| std::str::from_utf8(&output.stdout).ok())
                            .map(|s| {
                                match s.strip_prefix(PTY_BINARY_NAME) {
                                    Some(s) => s,
                                    None => s,
                                }
                                .trim()
                            });

                        if version == Some(env!("CARGO_PKG_VERSION")) {
                            return;
                        }
                    }

                    if let Err(err) = tokio::fs::remove_file(&pty_shell_cpy).await {
                        error!(%err, "Failed to remove {PTY_BINARY_NAME} shell {shell:?} copy");
                    }
                    if let Err(err) = tokio::fs::copy(&pty_path, &pty_shell_cpy).await {
                        error!(%err, "Failed to copy {PTY_BINARY_NAME} to {}", pty_shell_cpy.display());
                    }
                });

                // Remove legacy pty shell copies
                let old_pty_cpy = local_bin.join(format!("{shell} ({OLD_PTY_BINARY_NAME})"));
                if old_pty_cpy.exists() {
                    if let Err(err) = tokio::fs::remove_file(&old_pty_cpy).await {
                        warn!(%err, "Failed to remove legacy pty: {old_pty_cpy:?}");
                    }
                }
            }
        },
        None => error!("Failed to find {PTY_BINARY_NAME} in bundle"),
    }

    // install the cli to ~/.local/bin
    match get_bundle_path_for_executable(CLI_BINARY_NAME) {
        Some(q_cli_path) => {
            let dest = local_bin.join(CLI_BINARY_NAME);
            if let Err(err) = symlink(&q_cli_path, dest).await {
                error!(%err, "Failed to symlink {CLI_BINARY_NAME}");
            }

            let legacy_cli_link = local_bin.join(OLD_CLI_BINARY_NAME);
            if legacy_cli_link.is_symlink() {
                if let Err(err) = symlink(q_cli_path, &legacy_cli_link).await {
                    warn!(%err, "Failed to symlink legacy CLI: {legacy_cli_link:?}");
                }
            }
        },
        None => error!("Failed to find {CLI_BINARY_NAME} in bundle"),
    }

    if let Some(bundle_path) = get_bundle_path() {
        let exe = bundle_path.join("Contents").join("MacOS").join(APP_PROCESS_NAME);
        let startup_launch_agent = LaunchdPlist::new("com.amazon.codewhisperer.launcher")
            .program_arguments([&exe.to_string_lossy(), "--is-startup", "--no-dashboard"])
            .associated_bundle_identifiers([APP_BUNDLE_ID])
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

    // Init the shell directory
    std::fs::create_dir(fig_data_dir()?.join("shell")).ok();
    for shell in fig_util::Shell::all().iter() {
        for script_integration in shell.get_script_integrations().unwrap_or_default() {
            if let Err(err) = script_integration.install().await {
                error!(%err, "Failed installing shell integration {}", script_integration.describe());
            }
        }

        for shell_integration in shell.get_shell_integrations().unwrap_or_default() {
            if let Err(err) = shell_integration.migrate().await {
                error!(%err, "Failed installing shell integration {}", shell_integration.describe());
            }
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
    use super::*;

    #[test]
    fn test_current_version() {
        current_version();
    }

    #[tokio::test]
    async fn test_symlink() {
        use tempfile::tempdir;

        let tmp_dir = tempdir().unwrap();
        let tmp_dir = tmp_dir.path();

        // folders
        let src_dir_1 = tmp_dir.join("dir_1");
        let src_dir_2 = tmp_dir.join("dir_2");
        let dst_dir = tmp_dir.join("dst");

        std::fs::create_dir_all(&src_dir_1).unwrap();
        std::fs::create_dir_all(&src_dir_2).unwrap();

        // Check that a new symlink is created
        assert!(!dst_dir.exists());
        symlink(&src_dir_1, &dst_dir).await.unwrap();
        assert!(dst_dir.exists());
        assert_eq!(dst_dir.read_link().unwrap(), src_dir_1);

        // Check that the symlink is updated
        symlink(&src_dir_2, &dst_dir).await.unwrap();
        assert!(dst_dir.exists());
        assert_eq!(dst_dir.read_link().unwrap(), src_dir_2);

        // files
        let src_file_1 = src_dir_1.join("file_1");
        let src_file_2 = src_dir_2.join("file_2");
        let dst_file = dst_dir.join("file");

        std::fs::write(&src_file_1, "content 1").unwrap();
        std::fs::write(&src_file_2, "content 2").unwrap();

        // Check that a new symlink is created
        assert!(!dst_file.exists());
        symlink(&src_file_1, &dst_file).await.unwrap();
        assert!(dst_file.exists());
        assert_eq!(std::fs::read_to_string(&dst_file).unwrap(), "content 1");

        // Check that the symlink is updated
        symlink(&src_file_2, &dst_file).await.unwrap();
        assert!(dst_file.exists());
        assert_eq!(std::fs::read_to_string(&dst_file).unwrap(), "content 2");
    }
}
