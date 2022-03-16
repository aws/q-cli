use crate::{
    dotfiles::api::DotfileData,
    util::shell::{Shell, When},
};
use anyhow::{Context, Result};
use crossterm::tty::IsTty;
use std::io::stdin;

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

        if stdin().is_tty() {
            let get_prompts_source = || -> Option<String> { Some("fig app prompts".into()) };

            if let Some(source) =
                guard_source(shell, true, "FIG_CHECKED_PROMPTS", get_prompts_source)
            {
                to_source.push_str(&source);
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
