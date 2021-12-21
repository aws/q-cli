use std::{
    env::{current_dir, set_current_dir},
    path::PathBuf,
    str::FromStr,
};

use log::LevelFilter;
use tokio::sync::mpsc::{Sender, UnboundedSender};
use vte::{Params, Perform};

use crate::{
    command_info::CommandInfo,
    fig_info::FigInfo,
    history::HistoryFile,
    new_history::History,
    proto::{
        hooks::{hook_to_message, new_context, new_preexec_hook, new_prompt_hook},
        ShellContext,
    },
    screen::FigtermScreen,
};

struct ShellState {
    tty: Option<String>,
    pid: Option<i32>,
    session_id: Option<String>,
    hostname: Option<String>,

    shell: Option<String>,

    in_ssh: bool,
    in_docker: bool,

    preexec: bool,
    in_prompt: bool,
}

impl ShellState {
    fn new() -> ShellState {
        ShellState {
            tty: None,
            pid: None,
            session_id: None,
            hostname: None,
            shell: None,
            in_ssh: false,
            in_docker: false,
            preexec: false,
            in_prompt: false,
        }
    }
}

pub struct Figterm {
    fig_info: FigInfo,
    shell_state: ShellState,
    pub screen: FigtermScreen,

    history: Option<History>,

    last_command: Option<CommandInfo>,
    has_seen_prompt: bool,

    unix_socket_sender: UnboundedSender<Vec<u8>>,
}

impl Figterm {
    pub fn new(sender: UnboundedSender<Vec<u8>>, fig_info: FigInfo) -> Figterm {
        Figterm {
            unix_socket_sender: sender,

            fig_info,
            screen: FigtermScreen::new(),
            shell_state: ShellState::new(),

            history: History::load().ok(),

            has_seen_prompt: false,
            last_command: None,
        }
    }

    pub fn get_context(&self) -> ShellContext {
        new_context(
            self.shell_state.pid,
            self.shell_state.tty.clone(),
            self.shell_state.shell.clone(),
            None,
            self.fig_info.term_session_id.clone(),
            self.fig_info.fig_integration_version.clone(),
            None,
            None,
        )
    }
}

impl Perform for Figterm {
    fn print(&mut self, c: char) {
        self.screen.write(c);
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        let params_print = params
            .iter()
            .map(|p| std::str::from_utf8(*p).unwrap_or("invalid utf-8"))
            .collect::<Vec<_>>();

        log::info!(
            "[osc_dispatch] params={:?} bell_terminated={}",
            params_print,
            bell_terminated
        );

        match params[0] {
            b"697" => match params[1] {
                b"NewCmd" => {
                    let context = self.get_context();
                    let hook = new_prompt_hook(Some(context));
                    let message = hook_to_message(hook).to_fig_pbuf();
                    match self.unix_socket_sender.send(message) {
                        Ok(_) => (),
                        Err(e) => log::error!("Failed to queue `Prompt` hook: {}", e),
                    }

                    log::info!("Prompt at position: {:?}", self.screen.cursor);

                    if let Some(command) = &self.last_command {
                        if let Some(history) = &mut self.history {
                            match history.insert_command_history(command.clone()) {
                                Ok(_) => {}
                                Err(e) => log::error!("Failed to insert command history: {}", e),
                            }
                        }
                        log::info!("{:?}", command);
                    }
                }
                b"StartPrompt" => {
                    self.has_seen_prompt = true;

                    self.shell_state.in_prompt = true;
                }
                b"EndPrompt" => {
                    self.shell_state.in_prompt = false;
                }
                b"PreExec" => {
                    // Send PreExec hook
                    let context = self.get_context();
                    let hook = new_preexec_hook(Some(context));
                    let message = hook_to_message(hook).to_fig_pbuf();
                    match self.unix_socket_sender.send(message) {
                        Ok(_) => {}
                        Err(e) => log::error!("Failed to queue `PreExec` hook: {}", e),
                    }

                    self.last_command = Some(CommandInfo {
                        command: String::new(),
                        shell: self.shell_state.shell.clone(),
                        pid: self.shell_state.pid.clone(),
                        session_id: self.fig_info.term_session_id.clone(),
                        cwd: current_dir().ok(),
                        time: std::time::SystemTime::now()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        in_ssh: self.shell_state.in_ssh,
                        in_docker: self.shell_state.in_docker,
                        hostname: self.shell_state.shell.clone(),
                        exit_code: None,
                    });

                    self.shell_state.preexec = true;
                }
                param => {
                    let eq_pos = param.iter().position(|b| *b == b'=');
                    if let Some(eq_index) = eq_pos {
                        let (key, val) = param.split_at(eq_index);
                        let val = String::from_utf8_lossy(&val[1..]).to_string();

                        match key {
                            b"Dir" => {
                                log::info!("In dir {}", val);
                                match set_current_dir(val) {
                                    Ok(_) => {}
                                    Err(e) => log::error!("Failed to set current dir: {}", e),
                                }
                            }
                            b"ExitCode" => {
                                if let Some(command) = self.last_command.as_mut() {
                                    command.exit_code = val.parse().ok();
                                }
                            }
                            b"Shell" => {
                                self.shell_state.shell = Some(val);
                            }
                            b"FishSuggestionColor" => {}
                            b"ZshAutosuggestionColor" => {}
                            b"TTY" => {
                                self.shell_state.tty = Some(val);
                            }
                            b"PID" => self.shell_state.pid = val.parse().ok(),
                            b"SessionId" => self.shell_state.session_id = Some(val),
                            b"Docker" => self.shell_state.in_docker = &val == "1",
                            b"Hostname" => self.shell_state.hostname = Some(val),
                            b"Log" => {
                                // log::set_max_level(LevelFilter::from_str(&val).unwrap())
                            }
                            b"SSH" => self.shell_state.in_ssh = &val == "1",
                            _ => {}
                        }
                    }
                }
            },
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: char) {
        if ignore || intermediates.len() > 0 {
            return;
        }

        match action {
            // Cursor up
            'A' => {}
            // Cursor down
            'B' => {}
            // cursor forward
            'C' => {}
            // Cursor back
            'D' => {}
            // Go to line
            'd' => {}
            // Go to col
            'G' | '`' => {}
            // Goto
            'H' | 'f' => {}
            // Scroll up
            'S' => {}
            // Scroll down
            'T' => {}
            // Save cursor pos
            's' => {}
            // Restore cursor pos
            'u' => {}
            _ => {}
        }
    }
}
