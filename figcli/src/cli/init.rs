use crate::{
    dotfiles::api::DotfileData,
    util::{
        shell::{Shell, When},
        terminal::Terminal,
    },
};
use anyhow::{Context, Result};
use crossterm::tty::IsTty;
use fig_auth::is_logged_in;
use std::{env, io::stdin};

fn guard_source<F: Fn() -> Option<String>>(
    shell: &Shell,
    export: bool,
    guard_var: impl AsRef<str>,
    get_source: F,
) -> Option<String> {
    match get_source() {
        Some(source) => {
            let mut output = Vec::new();

            output.push(match shell {
                Shell::Bash | Shell::Zsh => {
                    format!("if [ -z \"${{{}}}\" ]; then", guard_var.as_ref())
                }
                Shell::Fish => format!("if test -z \"${}\"", guard_var.as_ref()),
            });

            output.push(source);

            output.push(match (shell, export) {
                (Shell::Bash | Shell::Zsh, false) => format!("{}=1", guard_var.as_ref()),
                (Shell::Bash | Shell::Zsh, true) => format!("export {}=1", guard_var.as_ref()),
                (Shell::Fish, false) => format!("set -g {} 1", guard_var.as_ref()),
                (Shell::Fish, true) => format!("set -gx {} 1", guard_var.as_ref()),
            });

            output.push(match shell {
                Shell::Bash | Shell::Zsh => "fi\n".into(),
                Shell::Fish => "end\n".into(),
            });

            Some(output.join("\n"))
        }
        _ => None,
    }
}

fn shell_init(shell: &Shell, when: &When) -> Result<String> {
    let should_source = fig_settings::state::get_bool("shell-integrations.enabled")
        .ok()
        .flatten()
        .unwrap_or(true);

    if !should_source {
        if let Some(source) = guard_source(shell, false, "FIG_SHELL_INTEGRATION_DISABLED", || {
            Some("echo '[Debug]: fig shell integration is disabled.'".to_string())
        }) {
            return Ok(source);
        }
    }

    let mut to_source = String::new();
    if let When::Post = when {
        // Add dotfiles sourcing
        let get_dotfile_source = || {
            let raw = std::fs::read_to_string(
                shell
                    .get_data_path()
                    .context("Failed to get shell data path")
                    .ok()?,
            )
            .ok()?;
            let source: DotfileData = serde_json::from_str(&raw).ok()?;
            Some(source.dotfile)
        };

        if let Some(source) = guard_source(shell, false, "FIG_DOTFILES_SOURCED", get_dotfile_source)
        {
            to_source.push_str(&source);
        }

        if stdin().is_tty() && env::var("PROCESS_LAUNCHED_BY_FIG").is_err() {
            // if no value, assume that we have seen onboarding already.
            // this is explictly set in onboarding in macOS app.
            let has_see_onboarding: bool = fig_settings::state::get_bool("user.onboarding")
                .ok()
                .flatten()
                .unwrap_or(true);

            let terminal = Terminal::current_terminal();

            if is_logged_in()
                && !has_see_onboarding
                && [Some(Terminal::Iterm), Some(Terminal::TerminalApp)].contains(&terminal)
            {
                to_source.push_str("fig app onboarding")
            } else {
                // not showing onboarding
                if let Some(source) =
                    guard_source(shell, false, "FIG_CHECKED_PROMPTS", || match shell {
                        Shell::Bash | Shell::Zsh => Some("(fig app prompts &)".to_string()),
                        Shell::Fish => Some("begin; fig app prompts &; end".to_string()),
                    })
                {
                    to_source.push_str(&source);
                }
            }
        }
    }

    let shell_integration_source = shell.get_fig_integration_source(when);
    to_source.push('\n');
    to_source.push_str(shell_integration_source);

    Ok(to_source)
}

pub async fn shell_init_cli(shell: &Shell, when: &When) -> Result<()> {
    println!("# {:?} for {:?}", when, shell);
    match shell_init(shell, when) {
        Ok(source) => println!("{}", source),
        Err(err) => println!("# Could not load source: {}", err),
    }
    Ok(())
}
