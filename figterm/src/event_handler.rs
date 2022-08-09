use std::str::FromStr;

use alacritty_terminal::event::{
    Event,
    EventListener,
};
use alacritty_terminal::term::{
    CommandInfo,
    ShellState,
};
use bytes::Bytes;
use fig_proto::hooks::{
    hook_to_message,
    new_preexec_hook,
    new_prompt_hook,
};
use fig_proto::local::LocalMessage;
use fig_telemetry::sentry::configure_scope;
use flume::Sender;
use tracing::level_filters::LevelFilter;
use tracing::{
    debug,
    error,
};

use crate::{
    logger,
    shell_state_to_context,
    EXECUTE_ON_NEW_CMD,
    INSERT_ON_NEW_CMD,
};

pub struct EventHandler {
    socket_sender: Sender<LocalMessage>,
    history_sender: Sender<CommandInfo>,
    input_sender: Sender<Bytes>,
}

impl EventHandler {
    pub fn new(
        socket_sender: Sender<LocalMessage>,
        history_sender: Sender<CommandInfo>,
        input_sender: Sender<Bytes>,
    ) -> Self {
        Self {
            socket_sender,
            history_sender,
            input_sender,
        }
    }
}

impl EventListener for EventHandler {
    fn send_event(&self, event: Event, shell_state: &ShellState) {
        debug!("{event:?}");
        debug!("{shell_state:?}");
        match event {
            Event::Prompt => {
                let context = shell_state_to_context(shell_state);
                let hook = new_prompt_hook(Some(context));
                let message = hook_to_message(hook);

                let insert_on_new_cmd = INSERT_ON_NEW_CMD.lock().take();
                let execute_on_new_cmd = {
                    let mut lock = EXECUTE_ON_NEW_CMD.lock();
                    let lock_val = *lock;
                    *lock = false;
                    lock_val
                };

                if let Some(text) = insert_on_new_cmd {
                    self.input_sender.send(text.into_bytes().into()).unwrap();
                    if execute_on_new_cmd {
                        self.input_sender.send(Bytes::from_static(b"\r")).unwrap();
                    }
                }

                if let Err(err) = self.socket_sender.send(message) {
                    error!("Sender error: {err:?}");
                }
            },
            Event::PreExec => {
                let context = shell_state_to_context(shell_state);
                let hook = new_preexec_hook(Some(context));
                let message = hook_to_message(hook);
                if let Err(err) = self.socket_sender.send(message) {
                    error!("Sender error: {err:?}");
                }
            },
            Event::CommandInfo(command_info) => {
                if let Err(err) = self.history_sender.send(command_info.clone()) {
                    error!("Sender error: {err:?}");
                }
            },
            Event::ShellChanged => {
                let shell = if shell_state.in_ssh || shell_state.in_docker {
                    shell_state.remote_context.shell.as_ref()
                } else {
                    shell_state.local_context.shell.as_ref()
                };
                configure_scope(|scope| {
                    if let Some(shell) = shell {
                        scope.set_tag("shell", shell);
                    }
                });
            },
        }
    }

    fn log_level_event(&self, level: Option<String>) {
        logger::set_log_level(
            level
                .and_then(|level| LevelFilter::from_str(&level).ok())
                .unwrap_or(LevelFilter::INFO),
        );
    }
}
