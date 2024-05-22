use std::fmt::Write;
use std::process::ExitCode;

use anstream::println;
use clap::Subcommand;
use crossterm::style::Stylize;
use eyre::Result;
use fig_api_client::ai::list_customizations;
use fig_api_client::Customization;
use fig_settings::settings;

use super::OutputFormat;

const INLINE_ENABLED_SETTINGS_KEY: &str = "inline.enabled";

#[derive(Debug, Clone, PartialEq, Subcommand)]
pub enum InlineSubcommand {
    /// Enables inline
    Enable,
    /// Disables inline
    Disable,
    /// Shows the status of inline
    Status,
    /// Select a customization if you have any available
    SetCustomization {
        /// The arn of the customization to use
        arn: Option<String>,
    },
    /// Show the available customizations
    ShowCustomizations {
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
}

impl InlineSubcommand {
    pub async fn execute(&self) -> Result<ExitCode> {
        match self {
            InlineSubcommand::Enable => {
                settings::set_value(INLINE_ENABLED_SETTINGS_KEY, true)?;
                println!("{}", "Inline enabled".magenta());
            },
            InlineSubcommand::Disable => {
                settings::set_value(INLINE_ENABLED_SETTINGS_KEY, false)?;
                println!("{}", "Inline disabled".magenta());
            },
            InlineSubcommand::Status => {
                let enabled = settings::get_bool(INLINE_ENABLED_SETTINGS_KEY)?.unwrap_or(true);
                println!("Inline is {}", if enabled { "enabled" } else { "disabled" }.bold());
            },
            InlineSubcommand::SetCustomization { arn } => {
                let customizations = list_customizations().await?;
                if customizations.is_empty() {
                    println!("No customizations found");
                    return Ok(ExitCode::FAILURE);
                }

                // if the user has specified an arn, use it
                if let Some(arn) = arn {
                    let Some(customization) = customizations.iter().find(|c| c.arn == *arn) else {
                        println!("Customization not found");
                        return Ok(ExitCode::FAILURE);
                    };

                    customization.save_selected()?;
                    println!(
                        "Customization {} selected",
                        customization.name.as_deref().unwrap_or_default().bold()
                    );
                    return Ok(ExitCode::SUCCESS);
                }

                let names = customizations
                    .iter()
                    .map(|c| {
                        format!(
                            "{} - {}",
                            c.name.as_deref().unwrap_or_default().bold(),
                            c.description.as_deref().unwrap_or_default()
                        )
                    })
                    .chain(["None".bold().to_string()])
                    .collect::<Vec<_>>();

                let select = crate::util::choose("Select a customization", &names)?;

                if select == customizations.len() {
                    Customization::delete_selected()?;
                    println!("Customization unset");
                } else {
                    customizations[select].save_selected()?;
                    println!(
                        "Customization {} selected",
                        customizations[select].name.as_deref().unwrap_or_default().bold()
                    );
                }
            },
            InlineSubcommand::ShowCustomizations { format } => {
                let customizations = list_customizations().await?;
                format.print(
                    || {
                        if customizations.is_empty() {
                            "No customizations found".into()
                        } else {
                            let mut s = String::new();
                            for customization in &customizations {
                                writeln!(s, "{}", customization.name.as_deref().unwrap_or_default().bold()).unwrap();
                                if let Some(description) = &customization.description {
                                    s.push_str(description);
                                }
                                s.push('\n');
                            }
                            s
                        }
                    },
                    || &customizations,
                );
            },
        }
        Ok(ExitCode::SUCCESS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]

    async fn test_subcommands() {
        let old_setting = settings::get_bool(INLINE_ENABLED_SETTINGS_KEY).unwrap();

        InlineSubcommand::Enable.execute().await.unwrap();
        assert!(settings::get_bool(INLINE_ENABLED_SETTINGS_KEY).unwrap().unwrap());
        InlineSubcommand::Status.execute().await.unwrap();

        InlineSubcommand::Disable.execute().await.unwrap();
        assert!(!settings::get_bool(INLINE_ENABLED_SETTINGS_KEY).unwrap().unwrap());
        InlineSubcommand::Status.execute().await.unwrap();

        // Testing customizations is not possible since we dont have auth in CI

        settings::set_value(INLINE_ENABLED_SETTINGS_KEY, old_setting).unwrap();
    }
}
