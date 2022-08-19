use std::io;
use std::process::Command;

use cfg_if::cfg_if;

use crate::wsl;

pub fn command(url: impl AsRef<str>) -> Command {
    cfg_if! {
        if #[cfg(target_os = "linux")] {
            let executable = if wsl::is_wsl() {
                "wslview"
            } else {
                "xdg-open"
            };

            let mut command = Command::new(executable);
            command.arg(url.as_ref());
            command
        } else if #[cfg(target_os = "macos")] {
            let mut command = Command::new("open");
            command.arg(url.as_ref());
            command
        } else if #[cfg(target_os = "windows")] {
            use std::os::windows::process::CommandExt;

            let detached = 0x8;
            let mut command = Command::new("cmd");
            command.creation_flags(detached);
            command.args(&["/c", "start", url.as_ref()]);
            command
        } else {
            compile_error!("Unsupported platform");
        }
    }
}

pub fn open_url(url: impl AsRef<str>) -> io::Result<()> {
    command(url).output()?;
    Ok(())
}

pub async fn open_url_async(url: impl AsRef<str>) -> tokio::io::Result<()> {
    tokio::process::Command::from(command(url)).output().await?;
    Ok(())
}
