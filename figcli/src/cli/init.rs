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
use std::{borrow::Cow, env, fmt::Display, io::stdin};

#[must_use]
fn guard_source<G, S>(shell: &Shell, export: bool, guard_var: G, source: S) -> String
where
    G: Display,
    S: Into<Cow<'static, str>>,
{
    let mut output: Vec<Cow<'static, str>> = Vec::new();

    output.push(match shell {
        Shell::Bash | Shell::Zsh => format!("if [ -z \"${{{guard_var}}}\" ]; then").into(),
        Shell::Fish => format!("if test -z \"${guard_var}\"").into(),
    });

    output.push(source.into());

    output.push(match (shell, export) {
        (Shell::Bash | Shell::Zsh, false) => format!("{guard_var}=1").into(),
        (Shell::Bash | Shell::Zsh, true) => format!("export {guard_var}=1").into(),
        (Shell::Fish, false) => format!("set -g {guard_var} 1").into(),
        (Shell::Fish, true) => format!("set -gx {guard_var} 1").into(),
    });

    output.push(match shell {
        Shell::Bash | Shell::Zsh => "fi\n".into(),
        Shell::Fish => "end\n".into(),
    });

    output.join("\n")
}

fn shell_init(shell: &Shell, when: &When) -> Result<String> {
    let should_source = fig_settings::state::get_bool("shell-integrations.enabled")
        .ok()
        .flatten()
        .unwrap_or(true);

    if !should_source {
        return Ok(guard_source(
            shell,
            false,
            "FIG_SHELL_INTEGRATION_DISABLED",
            "echo '[Debug]: fig shell integration is disabled.'",
        ));
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

        if let Some(source) = get_dotfile_source() {
            to_source.push_str(&guard_source(shell, false, "FIG_DOTFILES_SOURCED", source));
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
                to_source.push_str(match shell {
                    Shell::Bash | Shell::Zsh => "(fig restart daemon &> /dev/null &)\n",
                    Shell::Fish => "begin; fig restart daemon &> /dev/null &; end\n",
                });

                to_source.push_str("fig app onboarding\n")
            } else {
                // not showing onboarding
                to_source.push_str(&guard_source(
                    shell,
                    false,
                    "FIG_CHECKED_PROMPTS",
                    match shell {
                        Shell::Bash | Shell::Zsh => "(fig app prompts &)",
                        Shell::Fish => "begin; fig app prompts &; end",
                    },
                ));
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
