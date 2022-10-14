use std::io;

use cfg_if::cfg_if;

#[cfg(target_os = "macos")]
fn open_macos(url_str: &str) -> io::Result<bool> {
    use macos_accessibility_position::NSURL;
    use objc::runtime::{
        BOOL,
        NO,
    };
    use objc::{
        class,
        msg_send,
        sel,
        sel_impl,
    };

    let url = NSURL::from(url_str);
    let bool: BOOL = unsafe { msg_send![class!(NSWorkspace), openURL: url] };
    Ok(bool != NO)
}

#[cfg(target_os = "windows")]
fn open_command(url: impl AsRef<str>) -> std::process::Command {
    use std::os::windows::process::CommandExt;

    let detached = 0x8;
    let mut command = std::process::Command::new("cmd");
    command.creation_flags(detached);
    command.args(&["/c", "start", url.as_ref()]);
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
pub fn open_url(url: impl AsRef<str>) -> io::Result<bool> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            open_macos(url.as_ref())
        } else {
            open_command(url)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status().map(|status| status.success())
        }
    }
}

/// Returns bool indicating whether the URL was opened successfully
pub async fn open_url_async(url: impl AsRef<str>) -> io::Result<bool> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            open_macos(url.as_ref())
        } else {
            tokio::process::Command::from(open_command(url))
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status().await.map(|status| status.success())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_open_url() {
        assert!(open_url("https://fig.io").unwrap());
    }
}
