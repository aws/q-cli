use tokio::sync::mpsc::Sender;
use vte::{Params, Perform};

use crate::{proto::{hooks::new_context, ShellContext}, fig_info::FigInfo};

struct ShellState {
    tty: Option<String>,
    pid: Option<i32>,
    session_id: Option<String>,
    hostname: Option<String>,

    shell: Option<String>,

    in_ssh: Option<bool>,
    in_docker: Option<bool>,

    preexec: Option<bool>,
    in_prompt: Option<bool>,
}

impl ShellState {
    fn new() -> ShellState {
        ShellState {
            tty: None,
            pid: None,
            session_id: None,
            hostname: None,
            shell: None,
            in_ssh: None,
            in_docker: None,
            preexec: None,
            in_prompt: None,
        }
    }
}

pub struct Figterm {
    sender: Sender<Vec<u8>>,

    fig_info: FigInfo,

    shell_state: ShellState,
    has_seen_prompt: bool,
}

impl Figterm {
    pub fn new(sender: Sender<Vec<u8>>, fig_info: FigInfo) -> Figterm {
        Figterm {
            sender,

            fig_info,

            has_seen_prompt: false,
            shell_state: ShellState::new(),
        }
    }

    pub fn get_context(&self) -> ShellContext {
        let context = new_context(
            self.shell_state.pid,
            self.shell_state.tty.clone(),
            self.shell_state.shell.clone(),
            None,
            None,
            None,
            None,
            None,
        );
        return context;
    }
}

impl Perform for Figterm {
    fn print(&mut self, c: char) {
        log::info!("[print] {:?}", c);
    }

    fn execute(&mut self, byte: u8) {
        log::info!("[execute] {:02x}", byte);
    }

    fn hook(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        log::info!(
            "[hook] params={:?}, intermediates={:?}, ignore={:?}, char={:?}",
            params,
            intermediates,
            ignore,
            c
        );
    }

    fn put(&mut self, byte: u8) {
        log::info!("[put] {:02x}", byte);
    }

    fn unhook(&mut self) {
        log::info!("[unhook]");
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        let params_print = params
            .into_iter()
            .map(|p| std::str::from_utf8(*p).unwrap_or("invalid utf-8"))
            .collect::<Vec<_>>();

        log::info!(
            "[osc_dispatch] params={:?} bell_terminated={}",
            params_print,
            bell_terminated
        );

        match params[0] {
            b"697" => match params[1] {
                b"NewCmd" => {}
                b"StartPrompt" => {
                    self.has_seen_prompt = true;
                }
                b"EndPrompt" => {}
                b"PreExec" => {}
                param => {
                    let eq_pos = param.iter().position(|b| *b == b'=');
                    if let Some(eq_index) = eq_pos {
                        let (key, val) = param.split_at(eq_index);
                        let val = &val[1..];

                        match key {
                            b"Dir" => {}
                            b"ExitCode" => {}
                            b"Shell" => {}
                            b"FishSuggestionColor" => {}
                            b"TTY" => {}
                            b"PID" => {}
                            b"SessionId" => {}
                            b"Docker" => {}
                            b"Hostname" => {}
                            b"Log" => {}
                            b"SSH" => {}
                            _ => {}
                        }
                    }
                }
            },
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        log::info!(
            "[csi_dispatch] params={:#?}, intermediates={:?}, ignore={:?}, char={:?}",
            params,
            intermediates,
            ignore,
            c
        );
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        log::info!(
            "[esc_dispatch] intermediates={:?}, ignore={:?}, byte={:02x}",
            intermediates,
            ignore,
            byte
        );
    }
}
