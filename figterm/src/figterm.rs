use std::{env::set_current_dir, str::FromStr};

use log::LevelFilter;
use tokio::sync::mpsc::{Sender, UnboundedSender};
use vte::{Params, Perform};

use crate::{
    fig_info::FigInfo,
    proto::{
        hooks::{hook_to_message, new_context, new_prompt_hook, new_preexec_hook},
        ShellContext,
    },
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
    sender: UnboundedSender<Vec<u8>>,

    fig_info: FigInfo,

    shell_state: ShellState,
    has_seen_prompt: bool,

    line: usize,
    col: usize,
}

impl Figterm {
    pub fn new(sender: UnboundedSender<Vec<u8>>, fig_info: FigInfo) -> Figterm {
        Figterm {
            sender,

            fig_info,

            has_seen_prompt: false,
            shell_state: ShellState::new(),

            line: 1,
            col: 1,
        }
    }

    pub fn get_context(&self) -> ShellContext {
        new_context(
            self.shell_state.pid,
            self.shell_state.tty.clone(),
            self.shell_state.shell.clone(),
            None,
            None,
            None,
            None,
            None,
        )
    }
}

impl Perform for Figterm {
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
                    self.sender.send(message).unwrap();
                }
                b"StartPrompt" => {
                    self.has_seen_prompt = true;
                }
                b"EndPrompt" => {}
                b"PreExec" => {
                    let context = self.get_context();
                    let hook = new_preexec_hook(Some(context));
                    let message = hook_to_message(hook).to_fig_pbuf();
                    self.sender.send(message).unwrap();
                }
                param => {
                    let eq_pos = param.iter().position(|b| *b == b'=');
                    if let Some(eq_index) = eq_pos {
                        let (key, val) = param.split_at(eq_index);
                        let val = String::from_utf8_lossy(&val[1..]).to_string();

                        match key {
                            b"Dir" => {
                                log::info!("In dir {}", val);
                                set_current_dir(val).unwrap();
                            }
                            b"ExitCode" => {}
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
            'A' => {
                // Cursor up
            }
            'B' => {
                // Cursor down
            }
            'C' => {
                // cursor forward
            }
            'D' => {
                // Cursor back
            }
            'd' => {
                // Go to line
            }
            'G' | '`' => {
                // Go to col
            }
            'H' | 'f' => {
                // Goto
            }
            'S' => {
                // Scroll up
            }
            'T' => {
                // Scroll down
            }
            's' => {
                // Save cursor pos
            }
            'u' => {
                // Restore cursor pos
            }
            _ => {}
        }
    }
}
