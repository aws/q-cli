use camino::Utf8Path;
use fig_util::consts::FIG_BUNDLE_ID;
use fig_util::launchd_plist::{
    create_launch_agent,
    LaunchdPlist,
};
use tokio::process::Command;

use crate::{
    Error,
    Result,
};

static DAEMON_NAME: &str = "io.fig.dotfiles-daemon";

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
        let output = Command::new("launchctl").arg("list").output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let status = stdout
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .find(|line| line.get(2) == Some(&DAEMON_NAME))
            .and_then(|data| data.get(1).and_then(|v| v.parse::<i32>().ok()));

        Ok(status)
    }
}
