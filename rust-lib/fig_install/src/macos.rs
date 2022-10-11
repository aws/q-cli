use std::borrow::BorrowMut;
use std::env::current_exe;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::{
    Path,
    PathBuf,
};
use std::process::exit;

use fig_ipc::local::update_command;
use fig_util::{
    directories,
    launch_fig,
};
use regex::Regex;
use reqwest::IntoUrl;
use tempdir::TempDir;
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};
use tracing::warn;

use crate::index::UpdatePackage;
use crate::Error;

pub(crate) async fn update(update: UpdatePackage, deprecated: bool) -> Result<(), Error> {
    match option_env!("FIG_MACOS_BACKPORT") {
        Some(_) => {
            // Request and write the dmg to file
            let temp_dir = TempDir::new("fig")?;
            let dmg_path = temp_dir.path().join("Fig.dmg");
            download_dmg(update.download, &dmg_path).await?;

            // Shell out to hdiutil to mount the dmg
            let output = tokio::process::Command::new("hdiutil")
                .arg("attach")
                .arg(&dmg_path)
                .args(["-readonly", "-nobrowse", "-plist"])
                .output()
                .await?;
            if !output.status.success() {
                return Err(Error::UpdateFailed(String::from_utf8_lossy(&output.stderr).to_string()));
            }
            let plist = String::from_utf8_lossy(&output.stdout).to_string();

            // extract the app bundle
            let mut current_exe = current_exe()?;
            if current_exe.is_symlink() {
                current_exe = std::fs::read_link(current_exe)?;
            }

            let regex = Regex::new(r"<key>mount-point</key>\s*<\S+>([^<]+)</\S+>").unwrap();
            let mount_point = PathBuf::from(
                regex
                    .captures(&plist)
                    .unwrap()
                    .get(1)
                    .expect("mount-point will always exist")
                    .as_str(),
            );

            // /Applications/Fig.app/Contents/MacOS/{binary}
            let fig_app_path = current_exe.parent().unwrap().parent().unwrap().parent().unwrap();

            match fig_app_path.file_name() {
                Some(file_name) => {
                    if file_name != "Fig.app" {
                        return Err(Error::UpdateFailed(format!(
                            "the app bundle did not have the expected name, got {file_name:#?}, expected Fig.app"
                        )));
                    }
                },
                None => {
                    return Err(Error::UpdateFailed(
                        "the current binary is not within the expected app bundle".to_owned(),
                    ));
                },
            }

            tokio::fs::remove_dir_all(&fig_app_path).await?;

            let output = tokio::process::Command::new("ditto")
                .arg(mount_point.join("Fig.app"))
                .arg(&fig_app_path)
                .output()
                .await?;
            if !output.status.success() {
                return Err(Error::UpdateFailed(String::from_utf8_lossy(&output.stderr).to_string()));
            }

            // Shell out to unmount the dmg
            let output = tokio::process::Command::new("hdiutil")
                .arg("detach")
                .arg(&mount_point)
                .output()
                .await?;
            if !output.status.success() {
                return Err(Error::UpdateFailed(String::from_utf8_lossy(&output.stderr).to_string()));
            }

            let cli_path = current_exe.parent().unwrap().join("fig-darwin-universal");

            if !cli_path.exists() {
                return Err(Error::UpdateFailed(
                    "the update succeeded, but the cli did not have the expected name or was missing".to_owned(),
                ));
            }

            let mut command = OsString::new();
            command.push("sleep 2 && '");
            command.push(&cli_path);
            command.push("' restart app && '");
            command.push(&cli_path);
            command.push("' restart daemon");

            std::process::Command::new("/bin/bash")
                .process_group(0)
                .args(["--noediting", "--noprofile", "--norc", "-c"])
                .arg(command)
                .spawn()?;

            exit(0);
        },
        None => {
            // Let desktop app handle updates on macOS
            launch_fig(true, true)?;

            if update_command(deprecated).await.is_err() {
                return Err(Error::UpdateFailed(
                    "Unable to connect to Fig, it may not be running. To launch Fig, run 'fig launch'".to_owned(),
                ));
            }
        },
    }

    Ok(())
}

async fn remove_in_dir_with_prefix_unless(dir: &Path, prefix: &str, unless: impl Fn(&str) -> bool) {
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(prefix) && !unless(name) {
                    tokio::fs::remove_file(entry.path()).await.ok();
                    tokio::fs::remove_dir_all(entry.path()).await.ok();
                }
            }
        }
    }
}

pub(crate) async fn uninstall_desktop() -> Result<(), Error> {
    let app_path = PathBuf::from("Applications").join("Fig.app");
    if app_path.exists() {
        tokio::fs::remove_dir_all(&app_path)
            .await
            .map_err(|err| warn!("Failed to remove Fig.app: {err}"))
            .ok();
    }

    // Remove launch agents
    if let Ok(home) = directories::home_dir() {
        let launch_agents = home.join("Library").join("LaunchAgents");
        remove_in_dir_with_prefix_unless(&launch_agents, "io.fig.", |p| p.contains("daemon")).await;
    } else {
        warn!("Could not find home directory");
    }

    // Delete Fig defaults on macOS
    tokio::process::Command::new("defaults")
        .args(["delete", "com.mschrage.fig"])
        .output()
        .await
        .map_err(|err| warn!("Failed to delete defaults: {err}"))
        .ok();

    tokio::process::Command::new("defaults")
        .args(["delete", "com.mschrage.fig.shared"])
        .output()
        .await
        .map_err(|err| warn!("Failed to delete defaults: {err}"))
        .ok();

    // Delete data dir
    if let Ok(fig_data_dir) = directories::fig_data_dir() {
        tokio::fs::remove_dir_all(&fig_data_dir)
            .await
            .map_err(|err| warn!("Could not remove {}: {err}", fig_data_dir.display()))
            .ok();
    }

    // Delete the ~/.fig folder
    if let Ok(fig_dir) = directories::fig_dir() {
        tokio::fs::remove_dir_all(fig_dir)
            .await
            .map_err(|err| warn!("Could not remove ~/.fig folder: {err}"))
            .ok();
    } else {
        warn!("Could not find .fig folder");
    }

    uninstall_terminal_integrations().await;

    Ok(())
}

async fn uninstall_terminal_integrations() {
    // Delete integrations
    if let Ok(home) = directories::home_dir() {
        // Delete iTerm integration
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

        // Delete VSCode integration
        for (folder, prefix) in &[
            (".vscode/extensions", "withfig.fig-"),
            (".vscode-insiders/extensions", "withfig.fig-"),
            (".vscode-oss/extensions", "withfig.fig-"),
        ] {
            let folder = home.join(folder);
            remove_in_dir_with_prefix_unless(&folder, prefix, |_| false).await;
        }

        // Remove Hyper integration
        let hyper_path = home.join(".hyper.js");
        if hyper_path.exists() {
            // Read the config file
            match tokio::fs::File::open(&hyper_path).await {
                Ok(mut file) => {
                    let mut contents = String::new();
                    match file.read_to_string(&mut contents).await {
                        Ok(_) => {
                            contents = contents.replace("\"fig-hyper-integration\",", "");
                            contents = contents.replace("\"fig-hyper-integration\"", "");

                            // Write the config file
                            match tokio::fs::File::create(&hyper_path).await {
                                Ok(mut file) => {
                                    file.write_all(contents.as_bytes())
                                        .await
                                        .map_err(|err| warn!("Could not write to Hyper config: {err}"))
                                        .ok();
                                },
                                Err(err) => {
                                    warn!("Could not create Hyper config: {err}")
                                },
                            }
                        },
                        Err(err) => {
                            warn!("Could not read Hyper config: {err}");
                        },
                    }
                },
                Err(err) => {
                    warn!("Could not open Hyper config: {err}");
                },
            }
        }

        // Remove Kitty integration
        let kitty_path = home.join(".config").join("kitty").join("kitty.conf");
        if kitty_path.exists() {
            // Read the config file
            match tokio::fs::File::open(&kitty_path).await {
                Ok(mut file) => {
                    let mut contents = String::new();
                    match file.read_to_string(&mut contents).await {
                        Ok(_) => {
                            contents = contents.replace("watcher ${HOME}/.fig/tools/kitty-integration.py", "");
                            // Write the config file
                            match tokio::fs::File::create(&kitty_path).await {
                                Ok(mut file) => {
                                    file.write_all(contents.as_bytes())
                                        .await
                                        .map_err(|err| warn!("Could not write to Kitty config: {err}"))
                                        .ok();
                                },
                                Err(err) => {
                                    warn!("Could not create Kitty config: {err}")
                                },
                            }
                        },
                        Err(err) => {
                            warn!("Could not read Kitty config: {err}");
                        },
                    }
                },
                Err(err) => {
                    warn!("Could not open Kitty config: {err}");
                },
            }
        }
        // TODO: Add Jetbrains integration
    }
}

async fn download_dmg(src: impl IntoUrl, dst: impl AsRef<Path>) -> Result<(), Error> {
    let client = fig_request::client().expect("fig_request client must be instantiated on first request");
    let mut response = client.get(src).send().await?;

    let mut file = tokio::fs::File::create(&dst).await?;
    while let Some(mut bytes) = response.chunk().await? {
        file.write_all_buf(bytes.borrow_mut()).await?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_download_dmg() -> Result<(), Error> {
        let temp_dir = TempDir::new("fig")?;
        let dmg_path = temp_dir.path().join("Fig.dmg");
        download_dmg("https://desktop.docker.com/mac/main/arm64/Docker.dmg?utm_source=docker&utm_medium=webreferral&utm_campaign=docs-driven-download-mac-arm64", dmg_path).await
    }
}
