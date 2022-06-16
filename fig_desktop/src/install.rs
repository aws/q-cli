use semver::Version;
use tracing::error;

const PREVIOUS_VERSION_KEY: &str = "desktop.versionAtPreviousLaunch";

/// Run items at launch
pub async fn run_install() {
    tokio::spawn(async {
        if let Err(err) = fig_install::themes::clone_or_update().await {
            error!("Failed to clone or update themes: {err}");
        }
    });

    tokio::spawn(async {
        if let Err(err) = fig_install::plugins::fetch_installed_plugins(false).await {
            error!("Failed to fetch installed plugins: {err}");
        }
    });

    tokio::spawn(async {
        if let Err(err) = fig_install::dotfiles::download_and_notify().await {
            error!("Failed to fetch installed plugins: {err}");
        }
    });

    if should_run_install_script() {
        // Add any items that are only once per version
    }

    if let Err(err) = set_previous_version(current_version()) {
        error!("Failed to set previous version: {err}");
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
