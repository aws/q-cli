pub mod scheduler;
pub mod socket_server;
pub mod system_handler;
pub mod websocket;

use eyre::Result;
use parking_lot::Mutex;

pub struct DaemonStatus {
    /// The time the daemon was started as a u64 timestamp in seconds since the epoch
    time_started: u64,
    settings_watcher_status: Result<()>,
    websocket_status: Result<()>,
    system_socket_status: Result<()>,
}

impl Default for DaemonStatus {
    fn default() -> Self {
        Self {
            time_started: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time set before unix epoch")
                .as_secs(),
            settings_watcher_status: Ok(()),
            websocket_status: Ok(()),
            system_socket_status: Ok(()),
        }
    }
}

pub static IS_RUNNING_DAEMON: Mutex<bool> = Mutex::new(false);

/// Spawn the daemon to listen for updates and dotfiles changes
pub async fn daemon() -> Result<()> {
    use std::sync::Arc;
    use std::time::Duration;

    use futures::{
        SinkExt,
        StreamExt,
    };
    use parking_lot::RwLock;
    use rand::distributions::Uniform;
    use rand::prelude::Distribution;
    use tokio::select;
    use tokio_tungstenite::tungstenite;
    use tracing::{
        debug,
        error,
        info,
    };

    use crate::daemon::system_handler::spawn_incoming_system_handler;
    use crate::daemon::websocket::process_websocket;
    use crate::util::backoff::Backoff;

    *IS_RUNNING_DAEMON.lock() = true;

    info!("Starting daemon...");

    let daemon_status = Arc::new(RwLock::new(DaemonStatus::default()));

    // Add small random element to the delay to avoid all clients from
    // sending the messages at the same time
    let delay = Uniform::new(59., 61.).sample(&mut rand::thread_rng());

    // Spawn task scheduler
    let (mut scheduler, scheduler_join) = scheduler::Scheduler::spawn().await;
    match fig_settings::state::get_value("dotfiles.all.lastUpdated")
        .ok()
        .flatten()
    {
        Some(_) => scheduler.schedule_random_delay(scheduler::SyncDotfiles, 60., 1260.),
        None => scheduler.schedule_random_delay(scheduler::SyncDotfiles, 0., 60.),
    }

    scheduler.schedule_random_delay(scheduler::SyncScripts, 0., 1260.);

    scheduler.schedule_random_delay(
        scheduler::RecurringTask::new(
            scheduler::SendQueuedTelemetryEvents,
            Duration::from_secs(60 * 60 * 4),
            None,
        ),
        0.,
        60. * 60.,
    );

    // Send dotfiles line counts at least once per day (could happen more often
    // if the daemon restarts)
    scheduler.schedule_random_delay(
        scheduler::RecurringTask::new(
            scheduler::SendDotfilesLineCountTelemetry,
            Duration::from_secs(60 * 60 * 24),
            None,
        ),
        0.,
        60.,
    );

    // Spawn the incoming handler
    let daemon_status_clone = daemon_status.clone();
    let unix_join = tokio::spawn(async move {
        let daemon_status = daemon_status_clone;
        let mut backoff = Backoff::new(Duration::from_secs_f64(0.25), Duration::from_secs_f64(120.));
        loop {
            match spawn_incoming_system_handler(daemon_status.clone()).await {
                Ok(handle) => {
                    daemon_status.write().system_socket_status = Ok(());
                    backoff.reset();
                    if let Err(err) = handle.await {
                        error!("Error on system handler join: {err:?}");
                        daemon_status.write().system_socket_status = Err(err.into());
                    }
                    return;
                },
                Err(err) => {
                    error!("Error spawning system handler: {err:?}");
                    daemon_status.write().system_socket_status = Err(err);
                },
            }
            backoff.sleep().await;
        }
    });

    // Spawn websocket handler
    let daemon_status_clone = daemon_status.clone();
    let websocket_join = tokio::spawn(async move {
        let daemon_status = daemon_status_clone;
        let mut backoff = Backoff::new(Duration::from_secs_f64(0.25), Duration::from_secs_f64(300.));
        let mut ping_interval = tokio::time::interval(Duration::from_secs_f64(delay));
        loop {
            match websocket::connect_to_fig_websocket().await {
                Ok(mut websocket_stream) => {
                    daemon_status.write().websocket_status = Ok(());
                    loop {
                        select! {
                            next = websocket_stream.next() => {
                                match process_websocket(&next, &mut scheduler).await {
                                    Ok(()) => backoff.reset(),
                                    Err(err) => {
                                        error!("Error while processing websocket message: {err}");
                                        daemon_status.write().websocket_status = Err(err);
                                        break;
                                    }
                                }
                            }
                            _ = ping_interval.tick() => {
                                debug!("Sending ping to websocket");
                                if let Err(err) = websocket_stream.send(tungstenite::Message::Ping(vec![])).await {
                                    error!("Error while sending ping to websocket: {err}");
                                    daemon_status.write().websocket_status = Err(err.into());
                                    break;
                                };
                            }
                        }
                    }
                },
                Err(err) => {
                    error!("Error while connecting to websocket: {err}");
                    daemon_status.write().websocket_status = Err(err);
                },
            }
            backoff.sleep().await;
        }
    });

    let websocket_listen_join = tokio::spawn(async {
        if let Err(err) = socket_server::spawn_socket().await {
            error!("Failed to spawn websocket server: {err}");
        }
    });

    info!("Daemon is now running");

    tokio::try_join!(scheduler_join, unix_join, websocket_join, websocket_listen_join)?;

    Ok(())
}
