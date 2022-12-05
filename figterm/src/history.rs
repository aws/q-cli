use alacritty_terminal::term::CommandInfo;
use flume::{
    bounded,
    Sender,
};
use tracing::{
    error,
    trace,
};

pub async fn spawn_history_task() -> Sender<CommandInfo> {
    trace!("Spawning history task");

    let (sender, receiver) = bounded::<CommandInfo>(64);
    tokio::task::spawn(async move {
        let history_join = tokio::task::spawn_blocking(fig_history::History::load);

        match history_join.await {
            Ok(Ok(history)) => {
                while let Ok(command) = receiver.recv_async().await {
                    let command_info = fig_history::CommandInfo {
                        command: command.command,
                        shell: command.shell,
                        pid: command.pid,
                        session_id: command.session_id,
                        cwd: command.cwd,
                        start_time: command.start_time,
                        end_time: command.end_time,
                        hostname: command.hostname,
                        exit_code: command.exit_code,
                    };

                    if let Err(err) = history.insert_command_history(&command_info, true) {
                        error!(%err, "Failed to insert command into history");
                    }
                }
            },
            Ok(Err(err)) => {
                error!(%err, "Failed to load history");
            },
            Err(err) => {
                error!(%err, "Failed to join history thread");
            },
        }
    });

    sender
}
