use std::fs::OpenOptions;
use std::io::prelude::*;
use std::process::exit;

use clap::Subcommand;
use crossterm::style::Stylize;
use eyre::{
    Result,
    WrapErr,
};
use fig_ipc::local::send_hook_to_socket;
use fig_proto::hooks;
use fig_util::directories;
use fig_util::terminal::CURRENT_TERMINAL;
use once_cell::sync::Lazy;

#[derive(Debug, PartialEq, Eq, Subcommand)]
#[command(hide = true)]
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
        #[arg(long)]
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
        let session_id = std::env::var("CWTERM_SESSION_ID").ok();

        let hook = match self {
            HookSubcommand::Editbuffer {
                session_id,
                tty,
                pid,
                histno,
                cursor,
                text,
                ..
            } => {
                let context = hooks::generate_shell_context(*pid, tty, Some(session_id.clone()))?;
                Ok(hooks::new_edit_buffer_hook(context, text, *histno, *cursor, None))
            },
            HookSubcommand::Hide => Ok(hooks::new_hide_hook()),
            HookSubcommand::Init { pid, tty } => {
                let context = hooks::generate_shell_context(*pid, tty, session_id)?;
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
                let context = hooks::generate_shell_context(*pid, tty, session_id)?;
                Ok(hooks::new_preexec_hook(context))
            },
            HookSubcommand::Prompt { pid, tty } => {
                let context = hooks::generate_shell_context(*pid, tty, session_id)?;
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
                    let installed_hosts_file = directories::fig_data_dir()
                        .context("Can't get fig dir")?
                        .join("ssh_hostnames");
                    let mut installed_hosts = OpenOptions::new()
                        .create(true)
                        .read(true)
                        .append(true)
                        .open(installed_hosts_file)?;

                    let mut contents = String::new();
                    #[allow(clippy::verbose_file_reads)]
                    installed_hosts.read_to_string(&mut contents)?;

                    if !contents.contains(remote_dest) {
                        let bar = format!("╞{}╡", (0..74).map(|_| '═').collect::<String>());
                        println!(
                            "{bar}\n  To install SSH support for {}, run the following on your remote machine\n\n    {} {} \n     \
                            source <(curl -Ls fig.io/install)\n\n    🐟 {} \n     curl -Ls fig.io/install | bash; and exec fish\n{bar}",
                            "Fig".magenta(),
                            *BASH_UNICODE,
                            "Bash/zsh:".bold().underlined(),
                            "Fish:".bold().underlined(),
                        );
                        let new_line = format!("\n{remote_dest}");
                        installed_hosts.write_all(&new_line.into_bytes())?;
                    }
                }
                let context = hooks::generate_shell_context(*pid, tty, session_id)?;
                hooks::new_ssh_hook(context, control_path, remote_dest)
            },
        };

        let hook = hook.context("Invalid input for hook")?;
        Ok(send_hook_to_socket(hook).await?)
    }
}
