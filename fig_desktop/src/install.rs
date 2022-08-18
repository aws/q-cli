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

        #[cfg(target_os = "linux")]
        tokio::spawn(async {
            use tokio::process::Command;
            match Command::new("fig").args(&["_", "install", "--daemon"]).output().await {
                Ok(std::process::Output { status, stderr, .. }) if !status.success() => {
                    error!(?status, stderr = %String::from_utf8_lossy(&stderr), "Failed to init fig daemon");
                },
                Err(err) => error!(%err, "Failed to init fig daemon"),
                Ok(_) => {},
            }
        });
    }

    if let Err(err) = set_previous_version(current_version()) {
        error!(%err, "Failed to set previous version");
    }

    #[cfg(target_os = "linux")]
    tokio::spawn(async {
        match dbus::ibus::ibus_connect().await {
            Ok(ibus_connection) => match dbus::ibus::ibus_proxy(&ibus_connection).await {
                Ok(ibus_proxy) => {
                    // TODO(grant): Write cache via dbus ?
                    match ibus_proxy.set_global_engine("fig").await {
                        Ok(()) => tracing::debug!("Set IBus engine to 'fig'"),
                        Err(err) => error!(%err, "Failed to set global engine 'fig'"),
                    }
                },
                Err(err) => error!(%err, "IBus failed to proxy"),
            },
            Err(err) => error!(%err, "IBus failed to connect"),
        }
    });

    #[cfg(target_os = "linux")]
    // todo(mia): make this part of onboarding
    tokio::spawn(async {
        use tokio::process::Command;
        match Command::new("sh")
            .arg("-c")
            .arg("ps x | grep gnome-shell | wc -l")
            .output()
            .await
        {
            Ok(output) => {
                match String::from_utf8_lossy(&output.stdout)
                    .trim_matches('\n')
                    .parse::<u32>()
                {
                    Ok(num) => {
                        if num > 1 {
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
                    },
                    Err(err) => error!(%err, "Failed parsing process list"),
                }
            },
            Err(err) => error!(%err, "Failed getting process list"),
        }
    });
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
