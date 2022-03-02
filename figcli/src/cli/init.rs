use crate::{
    plugins::lock::LockData,
    util::shell::{Shell, When},
};
use anyhow::{Context, Result};
use crossterm::tty::IsTty;
use serde::{Deserialize, Serialize};
use std::io::stdin;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfileData {
    dotfile: String,
}

fn guard_source<F: Fn() -> Option<String>>(
    shell: &Shell,
    guard_var: impl AsRef<str>,
    get_source: F,
) -> Option<String> {
    let already_sourced = std::env::var(guard_var.as_ref()).unwrap_or_else(|_| "0".into());

    if already_sourced != "1" {
        if let Some(source) = get_source() {
            let source_guard = match shell {
                Shell::Fish => format!("set -gx {} 1", guard_var.as_ref()),
                _ => format!("export {}=1", guard_var.as_ref()),
            };
            return Some(format!("\n{}\n{}\n", source, source_guard));
        }
    }

    None
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

        if let Some(source) = guard_source(shell, "FIG_DOTFILES_SOURCED", get_dotfile_source) {
            to_source.push_str(&source);
        }

        if stdin().is_tty() {
            let get_prompts_source = || -> Option<String> { Some("fig app prompts".into()) };

            if let Some(source) = guard_source(shell, "FIG_CHECKED_PROMPTS", get_prompts_source) {
                to_source.push_str(&source);
            }
        }
    }

    let shell_integration_source = shell.get_fig_integration_source(when);
    to_source.push('\n');
    to_source.push_str(&shell_integration_source);

    Ok(to_source)
}

pub async fn shell_init_cli(shell: &Shell, when: &When) -> Result<()> {
    println!("# {:?} for {:?}", when, shell);
    match shell_init(shell, when) {
        Ok(source) => println!("{}", source),
        Err(err) => println!("# Could not load source: {}", err),
    }

    if let Ok(lock_data) = LockData::load().await {
        for plugin in lock_data.get_entries() {
            if let Some(shell_install) = plugin.shell_install.get(shell) {
                match when {
                    When::Pre => {
                        if let Some(source) = &shell_install.pre {
                            for line in source {
                                println!("{}", line);
                            }
                        }
                    }
                    When::Post => {
                        if let Some(files) = &shell_install.use_files {
                            for file in files {
                                println!("source '{}'", file.display());
                            }
                        }

                        if let Some(source) = &shell_install.post {
                            for line in source {
                                println!("{}", line);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
