use std::ffi::OsStr;
use std::io;
use std::process::Command;

use cfg_if::cfg_if;

pub fn command(url: impl AsRef<OsStr>) -> Command {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            let mut command = Command::new("open");
            command.arg(url);
            command
        } else if #[cfg(target_os = "linux")] {
            let mut command = Command::new("xdg-open");
            command.arg(url);
            command
        } else if #[cfg(windows)] {
            let mut command = Command::new("cmd");
            command.args(&["/c", "start", url.as_ref()]);
            command
        } else {
            compile_error!("Unsupported platform");
        }
    }
}

pub fn open_url(url: impl AsRef<OsStr>) -> io::Result<()> {
    command(url).output()?;
    Ok(())
}

pub async fn open_url_async(url: impl AsRef<OsStr>) -> tokio::io::Result<()> {
    tokio::process::Command::from(command(url)).output().await?;
    Ok(())
}
