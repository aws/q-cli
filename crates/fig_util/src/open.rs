use cfg_if::cfg_if;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Failed to open URL")]
    Failed,
}

#[cfg(target_os = "macos")]
fn open_macos(url_str: impl AsRef<str>) -> Result<(), Error> {
    use macos_utils::NSURL;
    use objc::runtime::{
        BOOL,
        NO,
        Object,
    };
    use objc::{
        class,
        msg_send,
        sel,
        sel_impl,
    };

    let url = NSURL::from(url_str.as_ref());
    let res: BOOL = unsafe {
        let shared: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
        msg_send![shared, openURL: url]
    };
    if res != NO { Ok(()) } else { Err(Error::Failed) }
}

#[cfg(target_os = "windows")]
fn open_command(url: impl AsRef<str>) -> std::process::Command {
    use std::os::windows::process::CommandExt;

    let detached = 0x8;
    let mut command = std::process::Command::new("cmd");
    command.creation_flags(detached);
    command.args(["/c", "start", url.as_ref()]);
    command
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn open_command(url: impl AsRef<str>) -> std::process::Command {
    let executable = if crate::system_info::in_wsl() {
        "wslview"
    } else {
        "xdg-open"
    };

    let mut command = std::process::Command::new(executable);
    command.arg(url.as_ref());
    command
}

/// Returns bool indicating whether the URL was opened successfully
pub fn open_url(url: impl AsRef<str>) -> Result<(), Error> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            open_macos(url)
        } else {
            match open_command(url)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
            {
                Ok(status) if status.success() => Ok(()),
                Ok(_) => Err(Error::Failed),
                Err(err) => Err(err.into()),
            }
        }
    }
}

/// Returns bool indicating whether the URL was opened successfully
pub async fn open_url_async(url: impl AsRef<str>) -> Result<(), Error> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            open_macos(url)
        } else {
            match tokio::process::Command::from(open_command(url))
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .await
            {
                Ok(status) if status.success() => Ok(()),
                Ok(_) => Err(Error::Failed),
                Err(err) => Err(err.into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_open_url() {
        open_url("https://fig.io").unwrap();
    }
}
