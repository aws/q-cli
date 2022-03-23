use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use fig_ipc::{connect_timeout, send_recv_message};
use fig_proto::daemon::{
    daemon_response::Response, sync_command::SyncType, sync_response::SyncStatus, DaemonResponse,
};

#[derive(Debug, Subcommand)]
pub enum PluginsSubcommands {
    /// Sync the current plugins
    Sync,
}

impl PluginsSubcommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            PluginsSubcommands::Sync => {
                println!();

                let spinner =
                    spinners::Spinner::new(spinners::Spinners::Dots, "Syncing plugins".into());

                // Get diagnostics from the daemon
                let socket_path = fig_ipc::daemon::get_daemon_socket_path();

                if !socket_path.exists() {
                    bail!("Could not find daemon socket, run `fig doctor` to diagnose");
                }

                let mut conn = match connect_timeout(&socket_path, Duration::from_secs(1)).await {
                    Ok(connection) => connection,
                    Err(_) => {
                        bail!("Could not connect to daemon socket, run `fig doctor` to diagnose");
                    }
                };

                let diagnostic_response_result: Option<fig_proto::daemon::DaemonResponse> =
                    send_recv_message(
                        &mut conn,
                        fig_proto::daemon::new_sync_message(SyncType::Plugins),
                        Duration::from_secs(10),
                    )
                    .await
                    .context("Could not get diagnostics from daemon")?;

                match diagnostic_response_result {
                    Some(DaemonResponse {
                        response: Some(Response::Sync(sync_result)),
                        ..
                    }) => match sync_result.status() {
                        SyncStatus::Ok => {
                            spinner.stop_with_message("✔️ Successfully synced plugins".into());
                            println!();
                            println!();
                        }
                        SyncStatus::Error => {
                            spinner.stop_with_message("❌ Failed to sync plugins".into());
                            bail!(sync_result.error().to_string());
                        }
                    },
                    _ => {
                        spinner.stop_with_message("❌ Failed to sync plugins".into());
                        bail!("Could not get diagnostics from daemon");
                    }
                }

                Ok(())
            }
        }
    }
}
