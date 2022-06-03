use std::ffi::OsStr;
use std::io;
use std::process::Command;

use cfg_if::cfg_if;

pub fn open_url(url: impl AsRef<OsStr>) -> io::Result<()> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            Command::new("open")
                .arg(url)
                .output()?;
            Ok(())
        } else if #[cfg(target_os = "linux")] {
            Command::new("xdg-open")
                .arg(url)
                .output()?;
            Ok(())
        } else if #[cfg(windows)] {
            Command::new("cmd")
                .arg("/c")
                .arg("start")
                .arg(url)
                .output();
            Ok(())
        } else {
            compile_error!("Unsupported platform");
        }
    }
}
