mod launchd_plist;

use camino::{
    Utf8Path,
    Utf8PathBuf,
};
use fig_util::directories;
use launchd_plist::LaunchdPlist;
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
        let log_path = directories::fig_dir()?.join("logs").join("daemon.log");
        let log_path_str = log_path.to_string_lossy();

        let daemon = LaunchdPlist::new(DAEMON_NAME)
            .program(executable.as_str())
            .program_arguments([executable.as_str(), "daemon"])
            .keep_alive(true)
            .run_at_load(true)
            .throttle_interval(20)
            .standard_out_path(&*log_path_str)
            .standard_error_path(&*log_path_str)
            .environment_variable("FIG_LOG_LEVEL", "debug")
            .plist();

        tokio::fs::create_dir_all(&daemon_dir()?).await?;
        tokio::fs::write(&daemon_path()?, daemon.as_bytes()).await?;

        Ok(())
    }

    pub async fn uninstall(&self) -> Result<()> {
        let path = daemon_path()?;
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }

        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        let path = daemon_path()?;
        let output = Command::new("launchctl").arg("load").arg(&path).output().await?;

        if !output.status.success() {
            return Err(Error::CommandFailed {
                command: format!("launchctl load {path}"),
                status: output.status,
                stderr: String::from_utf8_lossy(&output.stderr).into(),
            });
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let path = daemon_path()?;
        let output = Command::new("launchctl").arg("unload").arg(&path).output().await?;

        if !output.status.success() {
            return Err(Error::CommandFailed {
                command: format!("launchctl unload {path}"),
                status: output.status,
                stderr: String::from_utf8_lossy(&output.stderr).into(),
            });
        }

        Ok(())
    }

    pub async fn restart(&self) -> Result<()> {
        self.stop().await?;
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

fn daemon_dir() -> Result<Utf8PathBuf> {
    Ok(fig_util::directories::home_dir_utf8()?
        .join("Library")
        .join("LaunchAgents"))
}

fn daemon_path() -> Result<Utf8PathBuf> {
    Ok(daemon_dir()?.join(format!("{DAEMON_NAME}.plist")))
}
