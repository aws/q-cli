use std::fs::OpenOptions;
use std::io::prelude::*;

use anyhow::{
    Context,
    Result,
};
use clap::Subcommand;
use crossterm::style::Stylize;
use fig_ipc::hook::send_hook_to_socket;
use fig_proto::hooks;

#[derive(Debug, Subcommand)]
#[clap(hide = true)]
pub enum HookSubcommand {
    Editbuffer {
        session_id: String,
        integration: i32,
        tty: String,
        pid: i32,
        histno: i64,
        cursor: i64,
        text: String,
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
            } => {
                let context = hooks::generate_shell_context(*pid, tty, session_id.clone(), *integration)?;
                Ok(hooks::new_edit_buffer_hook(context, text, *histno, *cursor))
            },
            HookSubcommand::Hide => Ok(hooks::new_hide_hook()),
            HookSubcommand::Init { pid, tty } => {
                let context = hooks::generate_shell_context(*pid, tty, None, None)?;
                hooks::new_init_hook(context)
            },
            HookSubcommand::IntegrationReady { integration } => Ok(hooks::new_integration_ready_hook(integration)),
            HookSubcommand::KeyboardFocusChanged {
                app_identifier,
                focused_session_id,
            } => Ok(hooks::new_keyboard_focus_changed_hook(
                app_identifier,
                focused_session_id,
            )),
            HookSubcommand::PreExec { pid, tty } => {
                let context = hooks::generate_shell_context(*pid, tty, None, None)?;
                Ok(hooks::new_preexec_hook(context))
            },
            HookSubcommand::Prompt { pid, tty } => {
                let context = hooks::generate_shell_context(*pid, tty, None, None)?;
                Ok(hooks::new_prompt_hook(context))
            },
            HookSubcommand::Ssh {
                control_path,
                pid,
                tty,
                remote_dest,
                prompt,
            } => {
                if *prompt && !remote_dest.starts_with("git@") {
                    let installed_hosts_file = fig_directories::fig_dir()
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
                        println!(
                            "To install SSH support for {}, run the following on your remote machine\n\n  {} \n  \
                             source <(curl -Ls fig.io/install)\n\n  {} \n  curl -Ls fig.io/install | source\n",
                            "Fig".magenta(),
                            "For bash/zsh:".bold().underlined(),
                            "For Fish:".bold().underlined(),
                        );
                        let new_line = format!("\n{}", remote_dest);
                        installed_hosts.write_all(&new_line.into_bytes())?;
                    }
                }
                let context = hooks::generate_shell_context(*pid, tty, None, None)?;
                hooks::new_ssh_hook(context, control_path, remote_dest)
            },
        };

        let hook = hook.context("Invalid input for hook")?;
        send_hook_to_socket(hook).await
    }
}
