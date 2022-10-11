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
use fig_util::launch_fig;
use regex::Regex;
use reqwest::IntoUrl;
use tempdir::TempDir;
use tokio::io::AsyncWriteExt;

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
