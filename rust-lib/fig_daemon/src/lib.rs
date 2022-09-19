#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use std::process::ExitStatus;

use camino::Utf8Path;
#[cfg(target_os = "linux")]
use linux as os;
#[cfg(target_os = "macos")]
use macos as os;
use thiserror::Error;
#[cfg(target_os = "windows")]
use windows as os;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Util(#[from] fig_util::Error),
    #[error(transparent)]
    Directory(#[from] fig_util::directories::DirectoryError),
    #[error("Failed to run '{command}' ({status}): {stderr}")]
    CommandFailed {
        command: String,
        status: ExitStatus,
        stderr: String,
    },
    #[cfg(target_os = "linux")]
    #[error("Unsupported init system: {0}")]
    UnsupportedInitSystem(linux::InitSystem),
}

#[derive(Debug, Default)]
pub struct Daemon {
    inner: os::Daemon,
}

impl Daemon {
    /// Install the daemon
    ///
    /// Note: This should NOT start the daemon.
    pub async fn install(&self, executable: &Utf8Path) -> Result<()> {
        self.inner.install(executable).await?;
        self.start().await
    }

    /// Uninstall the daemon
    ///
    /// This may return an error if the daemon is not installed.
    pub async fn uninstall(&self) -> Result<()> {
        self.stop().await.ok();
        self.inner.uninstall().await
    }

    /// Start the daemon
    ///
    /// This will return Ok even if the daemon is already running.
    pub async fn start(&self) -> Result<()> {
        self.inner.start().await
    }

    /// Stop the daemon.
    pub async fn stop(&self) -> Result<()> {
        self.inner.stop().await
    }

    /// Restart the daemon
    pub async fn restart(&self) -> Result<()> {
        self.inner.restart().await
    }

    /// Get the previous status of the daemon
    ///
    /// This is useful if the daemon has crashed.
    pub async fn status(&self) -> Result<Option<i32>> {
        self.inner.status().await
    }
}

#[cfg(all(test, feature = "unsafe-tests"))]
mod unsafe_tests {
    use super::*;

    /// Test that installation/uninstallation flow completes successfully,
    /// and that install and uninstall can be repeated consecutively with success
    #[tokio::test]
    pub fn test_install() -> Result<()> {
        install_daemon().await?;
        install_daemon().await?;
        install_daemon().await?;
        uninstall_daemon().await?;
        uninstall_daemon().await?;
        uninstall_daemon().await?;
        Ok(())
    }

    /// Test that start/stop flow completes successfully,
    /// and that start and stop can be repeated consecutively with success
    #[tokio::test]
    pub fn test_start() -> Result<()> {
        install_daemon().await?;
        start_daemon().await?;
        start_daemon().await?;
        start_daemon().await?;
        stop_daemon().await?;
        stop_daemon().await?;
        stop_daemon().await?;
        uninstall_daemon().await?;
        Ok(())
    }

    /// Test that restart completes successfully,
    /// and that restart can be repeated consecutively with success
    #[tokio::test]
    pub fn test_restart() -> Result<()> {
        install_daemon().await?;
        start_daemon().await?;
        restart_daemon().await?;
        restart_daemon().await?;
        restart_daemon().await?;
        stop_daemon().await?;
        uninstall_daemon().await?;
        Ok(())
    }
}
