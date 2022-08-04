use std::fs::OpenOptions;
use std::io::prelude::*;
use std::process::exit;

use anyhow::{
    Context,
    Result,
};
use clap::Subcommand;
use crossterm::style::Stylize;
use fig_ipc::hook::send_hook_to_socket;
use fig_proto::hooks;
use fig_util::directories;
use fig_util::terminal::CURRENT_TERMINAL;
use once_cell::sync::Lazy;

#[derive(Debug, Subcommand)]
#[clap(hide = true)]
pub enum HookSubcommand {
    Editbuffer {
        #[clap(value_parser)]
        session_id: String,
        #[clap(value_parser)]
        integration: i32,
        #[clap(value_parser)]
        tty: String,
        #[clap(value_parser)]
        pid: i32,
        #[clap(value_parser)]
        histno: i64,
        #[clap(value_parser)]
        cursor: i64,
        #[clap(value_parser)]
        text: String,
    },
    Hide,
    Init {
        #[clap(value_parser)]
        pid: i32,
        #[clap(value_parser)]
        tty: String,
    },
    IntegrationReady {
        #[clap(value_parser)]
        integration: String,
    },
    KeyboardFocusChanged {
        #[clap(value_parser)]
        app_identifier: String,
        #[clap(value_parser)]
        focused_session_id: String,
    },
    PreExec {
        #[clap(value_parser)]
        pid: i32,
        #[clap(value_parser)]
        tty: String,
    },
    Prompt {
        #[clap(value_parser)]
        pid: i32,
        #[clap(value_parser)]
        tty: String,
    },
    Ssh {
        #[clap(value_parser)]
        pid: i32,
        #[clap(value_parser)]
        tty: String,
        #[clap(value_parser)]
        control_path: String,
        #[clap(value_parser)]
        remote_dest: String,
        #[clap(long, value_parser)]
        prompt: bool,
    },
}

static BASH_UNICODE: Lazy<String> = Lazy::new(|| {
    if let Some(terminal) = &*CURRENT_TERMINAL {
        if !terminal.supports_fancy_boxes() {
            return "$_".to_string();
        }
    }
    "\x1b[1m\x1b[3m$\x1b[0m\u{20de} ".to_string()
});

impl HookSubcommand {
    pub async fn execute(&self) -> Result<()> {
        // Hooks should exit silently on failure.
        match self.execute_hook().await {
            Ok(()) => Ok(()),
            Err(_) => exit(1),
        }
    }

    pub async fn execute_hook(&self) -> Result<()> {
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
                Ok(hooks::new_edit_buffer_hook(context, text, *histno, *cursor, None))
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
                if *prompt && !remote_dest.starts_with("git@") && !remote_dest.starts_with("aur@") {
                    let installed_hosts_file = directories::fig_dir()
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
                        let bar = format!("╞{}╡", (0..74).map(|_| '═').collect::<String>());
                        println!(
                            "{bar}\n  To install SSH support for {}, run the following on your remote machine\n\n    {} {} \n     \
                            source <(curl -Ls fig.io/install)\n\n    🐟 {} \n     curl -Ls fig.io/install | source\n{bar}",
                            "Fig".magenta(),
                            *BASH_UNICODE,
                            "Bash/zsh:".bold().underlined(),
                            "Fish:".bold().underlined(),
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
