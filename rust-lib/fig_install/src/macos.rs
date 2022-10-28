use std::borrow::BorrowMut;
use std::ffi::{
    CStr,
    CString,
    OsStr,
};
use std::os::unix::prelude::{
    OsStrExt,
    PermissionsExt,
};
use std::os::unix::process::CommandExt;
use std::path::{
    Path,
    PathBuf,
};
use std::process::exit;
use std::time::Duration;

use fig_ipc::local::update_command;
use fig_util::consts::{
    FIG_BUNDLE_ID,
    FIG_CLI_BINARY_NAME,
};
use fig_util::{
    directories,
    launch_fig_desktop,
};
use regex::Regex;
use reqwest::IntoUrl;
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};
use tokio::sync::mpsc::Sender;
use tracing::{
    debug,
    error,
    warn,
};

use crate::index::UpdatePackage;
use crate::{
    Error,
    UpdateStatus,
};

pub(crate) async fn update(update: UpdatePackage, deprecated: bool, tx: Sender<UpdateStatus>) -> Result<(), Error> {
    match option_env!("FIG_MACOS_BACKPORT") {
        Some(_) => {
            debug!("starting update");

            // Get all of the paths up front so we can get an error early if something is wrong

            let fig_app_path = fig_util::fig_bundle()
                .ok_or_else(|| Error::UpdateFailed("binary invoked does not reside in a valid app bundle.".into()))?;

            let temp_dir = tempfile::Builder::new().prefix("fig-download").tempdir()?;

            let dmg_path = temp_dir.path().join("Fig.dmg");
            let temp_bundle_path = temp_dir.path().join("Fig.app");

            let fig_app_cstr = CString::new(fig_app_path.as_os_str().as_bytes())?;
            let temp_bundle_cstr = CString::new(temp_bundle_path.as_os_str().as_bytes())?;

            // Set the permissions to 700 so that only the user can read and write
            let permissions = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(temp_dir.path(), permissions)?;

            debug!(?dmg_path, "downloading dmg");

            download_dmg(update.download, &dmg_path, update.size, tx.clone()).await?;

            tx.send(UpdateStatus::Message("Unpacking update...".into())).await.ok();

            // Shell out to hdiutil to mount the dmg
            let hdiutil_attach_output = tokio::process::Command::new("hdiutil")
                .arg("attach")
                .arg(&dmg_path)
                .args(["-readonly", "-nobrowse", "-plist"])
                .output()
                .await?;

            if !hdiutil_attach_output.status.success() {
                return Err(Error::UpdateFailed(
                    String::from_utf8_lossy(&hdiutil_attach_output.stderr).to_string(),
                ));
            }

            debug!("mounted dmg");

            let plist = String::from_utf8_lossy(&hdiutil_attach_output.stdout).to_string();

            let regex = Regex::new(r"<key>mount-point</key>\s*<\S+>([^<]+)</\S+>").unwrap();
            let mount_point = PathBuf::from(
                regex
                    .captures(&plist)
                    .unwrap()
                    .get(1)
                    .expect("mount-point will always exist")
                    .as_str(),
            );

            let ditto_output = tokio::process::Command::new("ditto")
                .arg(mount_point.join("Fig.app"))
                .arg(&temp_bundle_path)
                .output()
                .await?;

            if !ditto_output.status.success() {
                return Err(Error::UpdateFailed(
                    String::from_utf8_lossy(&ditto_output.stderr).to_string(),
                ));
            }

            tx.send(UpdateStatus::Message("Installing update...".into())).await.ok();

            let cli_path = fig_app_path.join("Contents").join("MacOS").join(FIG_CLI_BINARY_NAME);

            if !cli_path.exists() {
                return Err(Error::UpdateFailed(format!(
                    "the current app bundle is missing the CLI with the correct name {FIG_CLI_BINARY_NAME}"
                )));
            }

            match swap(&temp_bundle_cstr, &fig_app_cstr) {
                Ok(()) => {
                    debug!("swapped app bundle")
                },
                Err(err) => {
                    // TODO: Only fallback if the update is "interactive"

                    error!(?err, "failed to swap app bundle, trying to elevate permissions");

                    let mut file = {
                        let rights = security_framework::authorization::AuthorizationItemSetBuilder::new()
                            .add_right("system.privilege.admin")?
                            .build();

                        let auth = security_framework::authorization::Authorization::new(
                            Some(rights),
                            None,
                            security_framework::authorization::Flags::DEFAULTS
                                | security_framework::authorization::Flags::INTERACTION_ALLOWED
                                | security_framework::authorization::Flags::PREAUTHORIZE
                                | security_framework::authorization::Flags::EXTEND_RIGHTS,
                        )?;

                        let file = auth.execute_with_privileges_piped(
                            &cli_path,
                            &[
                                OsStr::new("_"),
                                OsStr::new("swap-files"),
                                temp_bundle_path.as_os_str(),
                                fig_app_path.as_os_str(),
                            ],
                            security_framework::authorization::Flags::DEFAULTS,
                        )?;

                        tokio::fs::File::from_std(file)
                    };

                    let mut out = String::new();
                    file.read_to_string(&mut out).await?;

                    match out.trim() {
                        "success" => {
                            debug!("swapped app bundle")
                        },
                        other => {
                            return Err(Error::UpdateFailed(other.to_owned()));
                        },
                    }
                },
            }

            // Shell out to unmount the dmg
            let output = tokio::process::Command::new("hdiutil")
                .arg("detach")
                .arg(&mount_point)
                .output()
                .await?;

            if !output.status.success() {
                error!(command =% String::from_utf8_lossy(&output.stderr).to_string(), "the update succeeded, but fig failed to unmount the dmg");
            } else {
                debug!("unmounted dmg");
            }

            if !cli_path.exists() {
                return Err(Error::UpdateFailed(format!(
                    "the update succeeded, but the cli did not have the expected name or was missing, expected {FIG_CLI_BINARY_NAME}"
                )));
            }

            debug!(?cli_path, "using cli at path");

            tx.send(UpdateStatus::Message("Relaunching...".into())).await.ok();

            debug!("restarting fig");
            std::process::Command::new(&cli_path)
                .process_group(0)
                .args(["_", "finish-update"])
                .spawn()?;

            tx.send(UpdateStatus::Exit).await.ok();

            exit(0);
        },
        None => {
            // Let desktop app handle updates on macOS
            launch_fig_desktop(true, true)?;

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
    // TODO(sean)
    // 1. Set title of running ttys "Restart this terminal to finish uninstalling Fig..."
    // 2. Delete webview cache

    // Remove launch agents
    if let Ok(home) = directories::home_dir() {
        let launch_agents = home.join("Library").join("LaunchAgents");
        remove_in_dir_with_prefix_unless(&launch_agents, "io.fig.", |p| p.contains("daemon")).await;
    } else {
        warn!("Could not find home directory");
    }

    // Delete Fig defaults on macOS
    tokio::process::Command::new("defaults")
        .args(["delete", FIG_BUNDLE_ID])
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

    let app_path = PathBuf::from("/Applications/Fig.app");
    if app_path.exists() {
        tokio::fs::remove_dir_all(&app_path)
            .await
            .map_err(|err| warn!("Failed to remove Fig.app: {err}"))
            .ok();
    }

    Ok(())
}

pub async fn uninstall_terminal_integrations() {
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

async fn download_dmg(
    src: impl IntoUrl,
    dst: impl AsRef<Path>,
    size: u64,
    tx: Sender<UpdateStatus>,
) -> Result<(), Error> {
    let client = fig_request::client().expect("fig_request client must be instantiated on first request");
    let mut response = client.get(src).timeout(Duration::from_secs(30 * 60)).send().await?;

    let mut bytes_downloaded = 0;
    let mut file = tokio::fs::File::create(&dst).await?;
    while let Some(mut bytes) = response.chunk().await? {
        bytes_downloaded += bytes.len() as u64;

        tx.send(UpdateStatus::Percent(bytes_downloaded as f32 / size as f32 * 100.0))
            .await
            .ok();

        tx.send(UpdateStatus::Message(format!(
            "Downloading ({:.2}/{:.2} MB)",
            bytes_downloaded as f32 / 1_000_000.0,
            size as f32 / 1_000_000.0
        )))
        .await
        .ok();

        file.write_all_buf(bytes.borrow_mut()).await?;
    }

    tx.send(UpdateStatus::Percent(100.0)).await.ok();

    Ok(())
}

pub fn swap(src: impl AsRef<CStr>, dst: impl AsRef<CStr>) -> Result<(), Error> {
    // We want to swap the app bundles, like sparkle does
    // https://github.com/sparkle-project/Sparkle/blob/863f85b5f5398c03553f2544668b95816b2860db/Sparkle/SUFileManager.m#L235
    let status = unsafe { libc::renamex_np(src.as_ref().as_ptr(), dst.as_ref().as_ptr(), libc::RENAME_SWAP) };

    if status != 0 {
        let err = std::io::Error::last_os_error();

        error!(%err, "failed to swap app bundle");

        if matches!(err.kind(), std::io::ErrorKind::PermissionDenied) {
            return Err(Error::UpdateFailed(
                "Failed to swap app bundle dur to permission denied. Please try running `sudo fig update`.".into(),
            ));
        } else {
            return Err(Error::UpdateFailed(format!("Failed to swap app bundle: {err}")));
        }
    }

    Ok(())
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[ignore]
//     #[tokio::test]
//     async fn test_download_dmg() -> Result<(), Error> {
//         let temp_dir = TempDir::new("fig")?;
//         let dmg_path = temp_dir.path().join("Fig.dmg");
//         download_dmg("https://desktop.docker.com/mac/main/arm64/Docker.dmg?utm_source=docker&utm_medium=webreferral&utm_campaign=docs-driven-download-mac-arm64", dmg_path).await
//     }
// }
