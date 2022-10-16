use alacritty_terminal::event::{
    Event,
    EventListener,
};
use alacritty_terminal::term::{
    CommandInfo,
    ShellState,
};
use fig_proto::secure::Hostbound;
use fig_proto::secure_hooks::{
    hook_to_message,
    new_preexec_hook,
    new_prompt_hook,
};
use fig_telemetry::sentry::configure_scope;
use flume::Sender;
use tracing::level_filters::LevelFilter;
use tracing::{
    debug,
    error,
};

use crate::{
    shell_state_to_context,
    MainLoopEvent,
    EXECUTE_ON_NEW_CMD,
    INSERT_ON_NEW_CMD,
};

pub struct EventHandler {
    socket_sender: Sender<Hostbound>,
    history_sender: Sender<CommandInfo>,
    main_loop_sender: Sender<MainLoopEvent>,
}

impl EventHandler {
    pub fn new(
        socket_sender: Sender<Hostbound>,
        history_sender: Sender<CommandInfo>,
        main_loop_sender: Sender<MainLoopEvent>,
    ) -> Self {
        Self {
            socket_sender,
            history_sender,
            main_loop_sender,
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

                if let Some(cwd) = &shell_state.local_context.current_working_directory {
                    if cwd.exists() {
                        std::env::set_current_dir(cwd).ok();
                    }
                }

                if let Some(text) = insert_on_new_cmd {
                    let mut insert: Vec<u8> = text.into_bytes();
                    if execute_on_new_cmd {
                        insert.extend_from_slice(b"\r");
                    }
                    self.main_loop_sender
                        .send(MainLoopEvent::Insert { insert, unlock: false })
                        .unwrap();
                }

                if let Err(err) = self.socket_sender.send(message) {
                    error!(%err, "Sender error");
                }
            },
            Event::PreExec => {
                let context = shell_state_to_context(shell_state);
                let hook = new_preexec_hook(Some(context));
                let message = hook_to_message(hook);

                self.main_loop_sender.send(MainLoopEvent::UnlockInterception).unwrap();

                if let Err(err) = self.socket_sender.send(message) {
                    error!(%err, "Sender error");
                }
            },
            Event::CommandInfo(command_info) => {
                if let Err(err) = self.history_sender.send(command_info.clone()) {
                    error!(%err, "Sender error");
                }
            },
            Event::ShellChanged => {
                let shell = &shell_state.local_context.shell;
                configure_scope(|scope| {
                    if let Some(shell) = shell {
                        scope.set_tag("shell", shell);
                    }
                });
            },
        }
    }

    fn log_level_event(&self, level: Option<String>) {
        if let Err(err) = fig_log::set_fig_log_level(level.unwrap_or_else(|| LevelFilter::INFO.to_string())) {
            error!(%err, "Failed to set log level");
        }
    }
}
