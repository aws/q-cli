use std::iter::empty;

use semver::Version;
use tracing::error;

const PREVIOUS_VERSION_KEY: &str = "desktop.versionAtPreviousLaunch";

/// Run items at launch
pub async fn run_install() {
    tokio::spawn(async {
        if let Err(err) = fig_install::themes::clone_or_update().await {
            error!(%err, "Failed to clone or update themes");
        }
    });

    tokio::spawn(async {
        if let Err(err) = fig_install::plugins::fetch_installed_plugins(false).await {
            error!(%err, "Failed to fetch installed plugins");
        }
    });

    tokio::spawn(async {
        if let Err(err) = fig_install::dotfiles::download_and_notify(false).await {
            error!(%err, "Failed to download installed plugins");
        }
    });

    if should_run_install_script() {
        // Add any items that are only once per version
        tokio::spawn(async {
            fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
                fig_telemetry::TrackEventType::UpdatedApp,
                fig_telemetry::TrackSource::App,
                empty::<(&str, &str)>(),
            ))
            .await
            .ok()
        });
    }

    if let Err(err) = set_previous_version(current_version()) {
        error!(%err, "Failed to set previous version");
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Output;
        use std::time::Duration;

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
            info!("Launching 'ibus-daemon'");
            match Command::new("ibus-daemon").arg("-drxR").output().await {
                Ok(Output { status, stdout, stderr }) if !status.success() => {
                    error!({
                        ?status,
                        stdout = %String::from_utf8_lossy(&stdout),
                        stderr = %String::from_utf8_lossy(&stderr)
                    }, "Failed to run 'ibus-daemon -drxR'");
                },
                Err(err) => error!(%err, "Failed to run 'ibus-daemon -drxR'"),
                Ok(_) => {},
            };
        }

        tokio::spawn(async {
            // Try for up to 10 attempts until it succeeds or stop
            for _ in 0..10 {
                // Sleep for a short duration to allow ibus engine to start
                // before it can handle our requests
                tokio::time::sleep(Duration::from_secs(1)).await;

                match ibus::ibus_connect().await {
                    Ok(ibus_connection) => match ibus::ibus_proxy(&ibus_connection).await {
                        Ok(ibus_proxy) => match ibus_proxy.set_global_engine("fig").await {
                            Ok(()) => {
                                tracing::debug!("Set IBus engine to 'fig'");
                                break;
                            },
                            Err(err) => error!(%err, "Failed to set global engine 'fig'"),
                        },
                        Err(err) => error!(%err, "IBus failed to proxy"),
                    },
                    Err(err) => error!(%err, "IBus failed to connect"),
                }
            }
        });
    }
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
