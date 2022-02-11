use crate::{plugins::lock::LockData, util::shell::{Shell, When}};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfileData {
    dotfile: String,
}

fn shell_init(shell: &Shell, when: &When) -> Result<String> {
    let dotfiles_sourced = std::env::var("FIG_DOTFILES_SOURCED")
        .unwrap_or_else(|_| "0".into());

    let mut to_source = String::new();
    if dotfiles_sourced == "1" {
        let raw = std::fs::read_to_string(
            shell
                .get_data_path()
                .context("Failed to get shell data path")?,
        )?;
        let source: DotfileData = serde_json::from_str(&raw)?;

        let dotfiles_source = match when {
            When::Pre => "",
            When::Post => &source.dotfile,
        };

        let source_guard = match shell {
            Shell::Fish => "set -gx FIG_DOTFILES_SOURCED 1",
            _ => "export FIG_DOTFILES_SOURCED=1",
        };

        to_source.push_str(source_guard);
        to_source.push_str(dotfiles_source);
    }

    let shell_integration_path = shell.get_fig_integration_path(when);
    let shell_integration_source = match shell_integration_path {
        Some(path) => {
            std::fs::read_to_string(path).unwrap_or_else(|_| String::new())
        }
        None => String::new()
    };

    to_source.push_str("\n");
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
