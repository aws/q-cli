mod systemd_unit;

use camino::{
    Utf8Path,
    Utf8PathBuf,
};
use systemd_unit::SystemdUnit;
use tokio::process::Command;

use crate::{
    Error,
    Result,
};

static DAEMON_NAME: &str = "fig-daemon";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InitSystem {
    /// Most common Linux init system
    ///
    /// <https://systemd.io/>
    Systemd,
    /// Init system used by artix, void, etc
    ///
    /// <http://smarden.org/runit/>
    Runit,
    /// Init subsystem used by alpine, gentoo, etc
    ///
    /// <https://wiki.gentoo.org/wiki/Project:OpenRC>
    OpenRc,
    /// An unknown init system
    Unknown,
}

impl InitSystem {
    pub async fn install(&self, cli_path: impl AsRef<Utf8Path>) -> Result<()> {
        match self {
            InitSystem::Systemd => {
                let path = InitSystem::Systemd.daemon_path()?;

                let log_path = fig_util::directories::fig_dir_utf8()?.join("logs").join("daemon.log");
                let log_path_str = format!("file:{log_path}");

                let unit = SystemdUnit::new("Fig Daemon")
                    .exec_start(format!("{} daemon", cli_path.as_ref()))
                    .restart("always")
                    .restart_sec(5)
                    .wanted_by("default.target")
                    .standard_output(&*log_path_str)
                    .standard_error(&*log_path_str)
                    .unit();

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(&parent)?;
                }
                std::fs::write(&path, unit.as_bytes())?;

                Ok(())
            },
            init_system => Err(Error::UnsupportedInitSystem(*init_system)),
        }
    }

    pub async fn uninstall(&self) -> Result<(), Error> {
        self.stop().await.ok();
        let path = self.daemon_path()?;
        if path.exists() {
            // Remove the definition file
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub async fn start(&self) -> Result<(), Error> {
        match self {
            InitSystem::Systemd => {
                let path = InitSystem::Systemd.daemon_path()?;
                let output = Command::new("systemctl")
                    .arg("--user")
                    .arg("--now")
                    .arg("enable")
                    .arg(&path)
                    .output()
                    .await?;

                if !output.status.success() {
                    return Err(Error::CommandFailed {
                        command: format!("systemctl --user --now --enable {path}"),
                        status: output.status,
                        stderr: String::from_utf8_lossy(&output.stderr).into(),
                    });
                }

                Ok(())
            },
            init_system => Err(Error::UnsupportedInitSystem(*init_system)),
        }
    }

    pub async fn stop(&self) -> Result<(), Error> {
        match self {
            InitSystem::Systemd => {
                Command::new("systemctl")
                    .arg("--user")
                    .arg("--now")
                    .arg("disable")
                    .arg(InitSystem::Systemd.daemon_path()?)
                    .output()
                    .await?;
                Ok(())
            },
            init_system => Err(Error::UnsupportedInitSystem(*init_system)),
        }
    }

    pub async fn restart(&self) -> Result<(), Error> {
        match self {
            InitSystem::Systemd => {
                Command::new("systemctl")
                    .arg("--user")
                    .arg("restart")
                    .arg(InitSystem::Systemd.daemon_path()?)
                    .output()
                    .await?;
                Ok(())
            },
            init_system => Err(Error::UnsupportedInitSystem(*init_system)),
        }
    }

    pub async fn status(&self) -> Result<Option<i32>> {
        match self {
            InitSystem::Systemd => {
                let output = Command::new("systemctl")
                    .arg("--user")
                    .arg("show")
                    .arg("-pExecMainStatus")
                    .arg(format!("{DAEMON_NAME}.service"))
                    .output()
                    .await?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                let status = stdout.split('=').last().and_then(|s| s.trim().parse::<i32>().ok());
                Ok(status)
            },
            init_system => Err(Error::UnsupportedInitSystem(*init_system)),
        }
    }

    fn daemon_path(&self) -> Result<Utf8PathBuf> {
        match self {
            InitSystem::Systemd => Ok(fig_util::directories::home_dir_utf8()?
                .join(".config")
                .join("systemd")
                .join("user")
                .join(format!("{DAEMON_NAME}.service"))),
            init_system => Err(Error::UnsupportedInitSystem(*init_system)),
        }
    }
}

impl std::fmt::Display for InitSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            InitSystem::Systemd => "systemd",
            InitSystem::Runit => "runit",
            InitSystem::OpenRc => "openrc",
            InitSystem::Unknown => "<unknown>",
        })
    }
}

async fn get_init_system() -> Result<InitSystem> {
    let output = Command::new("ps").args(["-p1"]).output().await?;

    if !output.status.success() {
        return Err(Error::CommandFailed {
            command: "ps -p1".into(),
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        });
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    if output_str.contains("systemd") {
        Ok(InitSystem::Systemd)
    } else if output_str.contains("runit") {
        Ok(InitSystem::Runit)
    } else if output_str.contains("openrc") {
        Ok(InitSystem::OpenRc)
    } else {
        Ok(InitSystem::Unknown)
    }
}

#[derive(Debug, Default)]
pub struct Daemon;

impl Daemon {
    pub async fn install(&self, cli_path: &Utf8Path) -> Result<()> {
        get_init_system().await?.install(cli_path).await
    }

    pub async fn uninstall(&self) -> Result<()> {
        get_init_system().await?.uninstall().await
    }

    pub async fn start(&self) -> Result<()> {
        get_init_system().await?.start().await
    }

    pub async fn stop(&self) -> Result<()> {
        get_init_system().await?.stop().await
    }

    pub async fn restart(&self) -> Result<()> {
        get_init_system().await?.restart().await
    }

    pub async fn status(&self) -> Result<Option<i32>> {
        get_init_system().await?.status().await
    }
}
