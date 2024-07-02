use crossterm::style::Stylize;
use fig_settings::settings::get_bool_or;
use fig_telemetry::{
    get_install_method,
    InstallMethod,
};
use fig_util::system_info::get_platform;
use fig_util::CLI_BINARY_NAME;
use semver::Version;
use tracing::warn;

const UPDATE_AVAILABLE_KEY: &str = "update.new-version-available";

fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
}

fn print_update_message(version: &Version) {
    println!(
        "\nA new version of {} is available: {}\nRun {} to update to the new version\n",
        CLI_BINARY_NAME.bold(),
        version.to_string().bold(),
        format!("{CLI_BINARY_NAME} update").magenta().bold()
    );
}

pub fn check_for_update() {
    // only show on Linux
    if get_platform() != "linux" {
        return;
    }

    // If updates are disabled, don't check for updates
    if !get_bool_or("app.disableAutoupdates", true) {
        return;
    }

    if get_install_method() == InstallMethod::Toolbox {
        return;
    }

    tokio::spawn(async {
        match fig_install::check_for_updates(false).await {
            Ok(Some(pkg)) => {
                if let Err(err) = fig_settings::state::set_value(UPDATE_AVAILABLE_KEY, pkg.version.to_string()) {
                    warn!(?err, "Error setting {UPDATE_AVAILABLE_KEY}: {err}");
                }
            },
            Ok(None) => {},
            Err(err) => {
                warn!(?err, "Error checking for updates: {err}");
            },
        };
    });

    match fig_settings::state::get_string(UPDATE_AVAILABLE_KEY) {
        Ok(Some(version)) => match Version::parse(&version) {
            Ok(version) => {
                let current_version = current_version();
                if version > current_version {
                    print_update_message(&version);
                }
            },
            Err(err) => {
                warn!(?err, "Error parsing {UPDATE_AVAILABLE_KEY}: {err}");
                let _ = fig_settings::state::remove_value(UPDATE_AVAILABLE_KEY);
            },
        },
        Ok(None) => {},
        Err(err) => {
            warn!(?err, "Error getting {UPDATE_AVAILABLE_KEY}: {err}");
            let _ = fig_settings::state::remove_value(UPDATE_AVAILABLE_KEY);
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version() {
        let version = current_version();
        println!("Crate version: {version}");
    }

    #[test]
    fn test_print_update_message() {
        let version = Version::parse("1.2.3").unwrap();
        println!("===");
        print_update_message(&version);
        println!("===");
    }
}
