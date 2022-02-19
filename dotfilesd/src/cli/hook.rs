use crate::util::fig_dir;
use anyhow::{Context, Result};
use clap::Subcommand;
use crossterm::style::Stylize;
use fig_ipc::hook::{self, send_hook_to_socket};
use std::fs::OpenOptions;
use std::io::prelude::*;

#[derive(Debug, Subcommand)]
#[clap(hide = true)]
pub enum HookSubcommand {
    Editbuffer {
        session_id: String,
        integration: i32,
        tty: String,
        pid: i32,
        histno: i32,
        cursor: i32,
        text: String,
    },
    Event {
        event_name: String,
    },
    Hide,
    Init {
        pid: i32,
        tty: String,
    },
    IntegrationReady {
        integration: String,
    },
    KeyboardFocusChanged {
        app_identifier: String,
        focused_session_id: String,
    },
    PreExec {
        pid: i32,
        tty: String,
    },
    Prompt {
        pid: i32,
        tty: String,
    },
    Ssh {
        pid: i32,
        tty: String,
        control_path: String,
        remote_dest: String,
        #[clap(long)]
        prompt: bool,
    },
}

impl HookSubcommand {
    pub async fn execute(&self) -> Result<()> {
        let hook = match self {
            HookSubcommand::Editbuffer {
                session_id,
                integration,
                tty,
                pid,
                histno,
                cursor,
                text,
            } => hook::create_edit_buffer_hook(
                session_id,
                *integration,
                tty,
                *pid,
                i64::from(*histno),
                i64::from(*cursor),
                text,
            ),
            HookSubcommand::Event { event_name } => hook::create_event_hook(event_name),
            HookSubcommand::Hide => hook::create_hide_hook(),
            HookSubcommand::Init { pid, tty } => hook::create_init_hook(*pid, tty),
            HookSubcommand::IntegrationReady { integration } => {
                hook::create_integration_ready_hook(integration)
            }
            HookSubcommand::KeyboardFocusChanged {
                app_identifier,
                focused_session_id,
            } => hook::create_keyboard_focus_changed_hook(app_identifier, focused_session_id),
            HookSubcommand::PreExec { pid, tty } => hook::create_preexec_hook(*pid, tty),
            HookSubcommand::Prompt { pid, tty } => hook::create_prompt_hook(*pid, tty),
            HookSubcommand::Ssh {
                control_path,
                pid,
                tty,
                remote_dest,
                prompt,
            } => {
                if *prompt {
                    let installed_hosts_file = fig_dir()
                        .context("Can't get fig dir")?
                        .join("ssh_hostnames");
                    let mut installed_hosts = OpenOptions::new()
                        .create(true)
                        .read(true)
                        .append(true)
                        .open(installed_hosts_file)?;

                    let mut contents = String::new();
                    installed_hosts.read_to_string(&mut contents)?;

                    if !contents.contains(remote_dest) {
                        println!("To install SSH support for {}, run the following on your remote machine\
                                  \n\n  {} \n  source <(curl -Ls fig.io/ssh)\
                                  \n\n  {} \n  curl -Ls fig.io/ssh | source\n",
                                  "Fig".magenta(),
                                  "For bash/zsh:".bold().underlined(),
                                  "For Fish:".bold().underlined(),
                        );
                        let new_line = format!("\n{}", remote_dest);
                        installed_hosts.write_all(&new_line.into_bytes())?;
                    }
                }
                hook::create_ssh_hook(*pid, tty, control_path, remote_dest)
            }
        };
        let hook = hook.context("Invalid input for hook")?;
        send_hook_to_socket(hook).await.context(format!(
            "\n{}\nFig might not be running to launch Fig run: {}\n",
            "Unable to Connect to Fig:".bold(),
            "fig launch".magenta()
        ))
    }
}
