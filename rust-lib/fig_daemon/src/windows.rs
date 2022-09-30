use std::os::windows::process::CommandExt;
use std::time::Duration;

use camino::Utf8Path;
use fig_ipc::SendRecvMessage;
use fig_util::directories;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

use crate::Result;

const DAEMON_NAME: &str = "FigDaemon";
const RUN_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

#[derive(Debug, Default)]
pub struct Daemon;

impl Daemon {
    pub async fn install(&self, executable: &Utf8Path) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (settings, _) = hkcu.create_subkey(RUN_PATH)?;
        settings.set_value(DAEMON_NAME, &format!("\"{}\" daemon", executable))?;
        Ok(())
    }

    pub async fn uninstall(&self) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (settings, _) = hkcu.create_subkey(RUN_PATH)?;
        settings.delete_value(DAEMON_NAME).ok();
        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        if let Ok(mut connection) = fig_ipc::BufferedUnixStream::connect_timeout(
            directories::daemon_socket_path()?,
            std::time::Duration::from_secs(1),
        )
        .await
        {
            if connection
                .send_recv_message_timeout::<_, fig_proto::daemon::DaemonResponse>(
                    fig_proto::daemon::new_ping_command(),
                    Duration::from_secs(1),
                )
                .await
                .is_ok()
            {
                return Ok(());
            }
        }

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu.open_subkey(RUN_PATH)?;
        let path: String = key.get_value(DAEMON_NAME)?;
        let command = path.split('"').nth(1).unwrap();

        std::process::Command::new(command)
            .arg("daemon")
            .creation_flags(0x8)
            .spawn()?;

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        // todo(chay): add a way to force quit all running daemon instances or something
        if let Ok(mut connection) =
            fig_ipc::BufferedUnixStream::connect_timeout(directories::daemon_socket_path()?, Duration::from_secs(1))
                .await
        {
            connection
                .send_recv_message_timeout::<_, fig_proto::daemon::DaemonResponse>(
                    fig_proto::daemon::new_quit_command(),
                    Duration::from_secs(1),
                )
                .await
                .ok();
        }

        Ok(())
    }

    pub async fn restart(&self) -> Result<()> {
        self.stop().await.ok();
        self.start().await
    }

    pub async fn status(&self) -> Result<Option<i32>> {
        if let Ok(mut connection) = fig_ipc::BufferedUnixStream::connect_timeout(
            directories::daemon_socket_path()?,
            std::time::Duration::from_secs(1),
        )
        .await
        {
            if connection
                .send_recv_message_timeout::<_, fig_proto::daemon::DaemonResponse>(
                    fig_proto::daemon::new_ping_command(),
                    Duration::from_secs(1),
                )
                .await
                .is_ok()
            {
                return Ok(Some(0));
            }
        }

        Ok(None)
    }
}
