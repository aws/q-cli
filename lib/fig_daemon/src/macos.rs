use camino::Utf8Path;
use fig_util::consts::FIG_BUNDLE_ID;
use fig_util::launchd_plist::{
    create_launch_agent,
    LaunchdPlist,
};
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::process::Command;

use crate::{
    Error,
    Result,
};

static DAEMON_NAME: &str = "io.fig.dotfiles-daemon";

/// Capture the exit status from the output of `launchctl list <daemon_name>`
///
/// Capturing the line that looks like this:
/// ```text
/// "LastExitStatus" = 0;
/// ```
static STATUS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#""LastExitStatus"\s*=\s*(\d+);"#).unwrap());

#[derive(Debug, Default)]
pub struct Daemon;

impl Daemon {
    pub async fn install(&self, executable: &Utf8Path) -> Result<(), Error> {
        let executable = executable.to_string();
        let daemon = LaunchdPlist::new(DAEMON_NAME)
            .program(&executable)
            .program_arguments([&executable, "daemon"])
            .keep_alive(true)
            .run_at_load(true)
            .throttle_interval(30)
            .associated_bundle_identifiers([FIG_BUNDLE_ID]);

        create_launch_agent(&daemon)?;

        Ok(())
    }

    pub async fn uninstall(&self) -> Result<()> {
        let path = LaunchdPlist::new(DAEMON_NAME).get_file_path()?;
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }

        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        let path = LaunchdPlist::new(DAEMON_NAME).get_file_path()?;
        let output = Command::new("launchctl")
            .args(["load", "-F"])
            .arg(&path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::CommandFailed {
                command: format!("launchctl load -F '{path}'"),
                status: output.status,
                stderr: String::from_utf8_lossy(&output.stderr).into(),
            });
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let path = LaunchdPlist::new(DAEMON_NAME).get_file_path()?;
        let output = Command::new("launchctl").args(["unload"]).arg(&path).output().await?;

        if !output.status.success() {
            return Err(Error::CommandFailed {
                command: format!("launchctl unload '{path}'"),
                status: output.status,
                stderr: String::from_utf8_lossy(&output.stderr).into(),
            });
        }

        Ok(())
    }

    pub async fn restart(&self) -> Result<()> {
        self.stop().await.ok();
        self.start().await
    }

    pub async fn status(&self) -> Result<Option<i32>> {
        let output = Command::new("launchctl").arg("list").arg(DAEMON_NAME).output().await?;

        if !output.status.success() {
            return Ok(None);
        }

        STATUS_REGEX
            .captures(&String::from_utf8_lossy(&output.stdout))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse::<i32>().ok())
            .map(Ok)
            .transpose()
    }
}
