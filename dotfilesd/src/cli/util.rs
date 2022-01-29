use anyhow::{Context, Result};
use std::{
    env,
    process::{exit, Command},
};

use dialoguer::theme::ColorfulTheme;

pub fn open_url(url: impl AsRef<str>) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url.as_ref())
            .output()
            .with_context(|| "Could not open url")?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url.as_ref())
            .output()
            .with_context(|| "Could not open url")?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg(url.as_ref())
            .output()
            .with_context(|| "Could not open url")?;

        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    unimplemented!();
}

/// Ensure the command is being run with root privileges.
/// If not, rexecute the command with sudo.
pub fn permission_guard() -> Result<()> {
    #[cfg(unix)]
    {
        use nix::unistd::geteuid;

        // Hack to persist the ZDOTDIR environment variable to the new process.
        if let Some(val) = env::var_os("ZDOTDIR") {
            if env::var_os("FIG_ZDOTDIR").is_none() {
                env::set_var("FIG_ZDOTDIR", val);
            }
        }

        let sudo_prompt = match env::var("USER") {
            Ok(user) => format!("Please enter your password for user {}: ", user),
            Err(_) => "Please enter your password: ".to_string(),
        };

        match geteuid().is_root() {
            true => Ok(()),
            false => {
                let mut child = Command::new("sudo")
                    .arg("-E")
                    .arg("-p")
                    .arg(sudo_prompt)
                    .args(env::args_os())
                    .spawn()?;

                let status = child.wait()?;

                exit(status.code().unwrap_or(1));
            }
        }
    }

    #[cfg(windows)]
    {
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(())
    }
}

pub fn dialoguer_theme() -> impl dialoguer::theme::Theme {
    ColorfulTheme {
        prompt_prefix: dialoguer::console::style("?".to_string())
            .for_stderr()
            .magenta(),
        ..ColorfulTheme::default()
    }
}
