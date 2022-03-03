use fig_directories::home_dir;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info};

use crate::cli::installation::{uninstall_cli, InstallComponents};

async fn remove_in_dir_with_prefix(dir: &Path, prefix: &str) {
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(entry) = entries.next_entry().await {
            if let Some(entry) = entry {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(prefix) {
                        tokio::fs::remove_file(entry.path()).await.ok();
                        tokio::fs::remove_dir_all(entry.path()).await.ok();
                    }
                }
            }
        }
    }
}

pub async fn uninstall_mac_app() {
    // Send uninstall telemetry event
    let tel_join = tokio::task::spawn(async {
        match fig_telemetry::SegmentEvent::new("Uninstall App") {
            Ok(mut event) => {
                if let Err(err) = event.add_default_properties() {
                    error!(
                        "Could not add default properties to telemetry event: {}",
                        err
                    );
                }

                if let Err(err) = event.send_event().await {
                    error!("Could not send telemetry event: {}", err);
                }
            }
            Err(err) => {
                error!("Could not send uninstall app telemetry: {}", err);
            }
        }
    });

    // Delete the .fig folder
    if let Some(fig_dir) = fig_directories::fig_dir() {
        match tokio::fs::remove_dir_all(fig_dir).await {
            Ok(_) => info!("Removed .fig folder"),
            Err(err) => error!("Could not remove .fig folder: {}", err),
        }
    }

    // Delete fig defaults
    let uuid = fig_auth::get_default("uuid").unwrap_or_default();
    tokio::process::Command::new("defaults")
        .args(["delete", "com.mschrage.fig"])
        .output()
        .await
        .ok();
    tokio::process::Command::new("defaults")
        .args(["delete", "com.mschrage.fig.shared"])
        .output()
        .await
        .ok();
    tokio::process::Command::new("defaults")
        .args(["write", "uuid", &uuid])
        .output()
        .await
        .ok();

    info!("Deleted fig defaults");

    let home = home_dir().unwrap();

    // Delete iTerm integration
    for path in [
        "Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.py",
        ".config/iterm2/AppSupport/Scripts/AutoLaunch/fig-iterm-integration.py",
        "Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.scpt",
    ] {
        tokio::fs::remove_file(home.join(path)).await.ok();
    }

    info!("Deleted iTerm2 integration");

    // Delete VSCode integration
    for (folder, prefix) in &[
        (".vscode/extensions", "withfig.fig-"),
        (".vscode-insiders/extensions", "withfig.fig-"),
        (".vscode-oss/extensions", "withfig.fig-"),
    ] {
        let folder = home.join(folder);
        remove_in_dir_with_prefix(&folder, prefix).await;
    }

    info!("Deleted VSCode integration");

    // Remove Hyper integration
    let hyper_path = home.join(".hyper.js");
    if hyper_path.exists() {
        // Read the config file
        let file = tokio::fs::File::open(&hyper_path).await.ok();
        if let Some(mut file) = file {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).await.is_ok() {
                contents = contents.replace("\"fig-hyper-integration\",", "");
                contents = contents.replace("\"fig-hyper-integration\"", "");

                // Write the config file
                if let Ok(mut file) = tokio::fs::File::create(&hyper_path).await {
                    if file.write_all(contents.as_bytes()).await.is_ok() {
                        info!("Deleted Hyper integration");
                    }
                }
            }
        }
    }

    // Remove Kitty integration
    let kitty_path = home.join(".config").join("kitty").join("kitty.conf");
    if kitty_path.exists() {
        // Read the config file
        let file = tokio::fs::File::open(&kitty_path).await.ok();
        if let Some(mut file) = file {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).await.is_ok() {
                contents = contents.replace("watcher ${HOME}/.fig/tools/kitty-integration.py", "");
                // Write the config file
                if let Ok(mut file) = tokio::fs::File::create(&kitty_path).await {
                    if file.write_all(contents.as_bytes()).await.is_ok() {
                        info!("Deleted Kitty integration");
                    }
                }
            }
        }
    }

    // TODO: Add Jetbrains integration
    // SOOON tm

    // Remove launch agents
    let launch_agents = home.join("Library").join("LaunchAgents");
    remove_in_dir_with_prefix(&launch_agents, "io.fig.").await;

    info!("Deleted launch agents");

    // Remove the app
    let fig_input_method_app = home
        .join("Library")
        .join("Input Methods")
        .join("FigInputMethod.app");
    if fig_input_method_app.exists() {
        tokio::fs::remove_dir_all(fig_input_method_app).await.ok();
    }

    let app_path = PathBuf::from("Applications").join("Fig.app");
    if app_path.exists() {
        tokio::fs::remove_dir_all(&app_path).await.ok();
    }

    info!("Deleted app");

    // Uninstall dotfiles, daemon, and CLI
    uninstall_cli(InstallComponents::all()).ok();

    tel_join.await.ok();

    // Remove the ~/.fig folder one last time incase any logs were left
    if let Some(fig_dir) = fig_directories::fig_dir() {
        tokio::fs::remove_dir_all(fig_dir).await.ok();
    }
}
