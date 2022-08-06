use std::fmt::Display;
use std::io::stdout;
use std::process::Command;

use arboard::Clipboard;
use clap::Args;
use crossterm::style::Stylize;
use dialoguer::theme::ColorfulTheme;
use serde::{
    Deserialize,
    Serialize,
};

use crate::util::dialoguer_theme;
use crate::util::spinner::{
    Spinner,
    SpinnerComponent,
};

#[derive(Debug, Args)]
pub struct AiArgs {
    #[clap(value_parser)]
    input: Vec<String>,
    /// Number of completions to generate (must be <=10)
    #[clap(short, long, value_parser)]
    n: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompleteResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Clone)]
enum DialogActions {
    Execute {
        command: String,
        display: bool,
    },
    Edit {
        command: String,
        display: bool,
    },
    #[allow(dead_code)]
    Copy {
        command: String,
        display: bool,
    },
    Regenerate,
    Ask,
    Cancel,
}

impl Display for DialogActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DialogActions::Execute { command, display } => {
                if *display {
                    write!(f, "⚡ Execute {}", command.to_string().magenta())
                } else {
                    write!(f, "⚡ Execute command")
                }
            },
            DialogActions::Edit { command, display } => {
                if *display {
                    write!(f, "📝 Edit {}", command.to_string().magenta())
                } else {
                    write!(f, "📝 Edit command")
                }
            },
            DialogActions::Copy { command, display } => {
                if *display {
                    write!(f, "📋 Copy {}", command.to_string().magenta())
                } else {
                    write!(f, "📋 Copy to clipboard")
                }
            },
            DialogActions::Regenerate => write!(f, "🔄 Regenerate answer"),
            DialogActions::Ask => write!(f, "❓ Ask another question"),
            DialogActions::Cancel => write!(f, "❌ Cancel"),
        }
    }
}

fn theme() -> ColorfulTheme {
    ColorfulTheme {
        success_prefix: dialoguer::console::style(" ".into()),
        values_style: dialoguer::console::Style::new().magenta().bright(),
        ..dialoguer_theme()
    }
}

impl AiArgs {
    pub async fn execute(self) -> anyhow::Result<()> {
        // Product gate
        if !fig_settings::settings::get_bool_or("product-gate.ai.enabled", false) {
            anyhow::bail!("Fig AI is comming soon to Fig Pro");
        }

        let Self { input, n } = self;
        let mut input = if input.is_empty() { None } else { Some(input.join(" ")) };

        if n.map(|n| n > 10).unwrap_or_default() {
            anyhow::bail!("n must be <= 10");
        }

        tokio::spawn(async {
            tokio::signal::ctrl_c().await.unwrap();
            crossterm::execute!(stdout(), crossterm::cursor::Show).unwrap();
            std::process::exit(0);
        });

        println!();

        'ask_loop: loop {
            let question = match input {
                Some(ref input) => input.clone(),
                None => {
                    println!("{}", "Translate Text to Bash".bold());
                    println!();

                    dialoguer::Input::with_theme(&theme())
                        .with_prompt("Text")
                        .validate_with(|input: &String| -> Result<(), &str> {
                            if input.trim().len() > 120 {
                                Err("Input is >120 characters")
                            } else {
                                Ok(())
                            }
                        })
                        .interact_text()?
                },
            };

            let question = question.trim().replace('\n', " ");

            if question.len() > 120 {
                anyhow::bail!("input is >120 characters");
            }

            'generate_loop: loop {
                let spinner_text = format!("  {} {} ", "Bash".bold(), "·".grey());

                let mut spinner = Spinner::new(vec![
                    SpinnerComponent::Text(spinner_text.clone()),
                    SpinnerComponent::Spinner,
                ]);

                let response: CompleteResponse = fig_request::Request::post("/ai/translate-bash")
                    .body(serde_json::json!({
                        "question": question,
                        "n": n.unwrap_or(1),
                        "os": std::env::consts::OS
                    }))
                    .auth()
                    .deser_json()
                    .await?;

                let choices: Vec<_> = response
                    .choices
                    .iter()
                    .filter_map(|choice| choice.text.as_deref())
                    .collect();

                macro_rules! handle_action {
                    ($action:expr) => {
                        match $action {
                            Some(DialogActions::Execute { command, .. }) => {
                                println!(
                                    "{} Executing {}...",
                                    ">".bold(),
                                    command.to_string().magenta().bold()
                                );
                                Command::new("bash")
                                    .arg("-ic")
                                    .arg(command)
                                    .spawn()?
                                    .wait()?;
                                break 'ask_loop;
                            },
                            Some(DialogActions::Edit { command, .. }) => {
                                let command: String = dialoguer::Input::with_theme(&theme())
                                    .with_initial_text(command)
                                    .interact_text()?;
                                println!("Executing {}...", command.to_string().magenta().bold());
                                Command::new("bash")
                                    .arg("-ic")
                                    .arg(command)
                                    .spawn()?
                                    .wait()?;
                                break 'ask_loop;
                            },
                            Some(DialogActions::Copy { command, .. }) => {
                                if let Ok(mut clipboard) = Clipboard::new() {
                                    match clipboard.set_text(command.to_string()) {
                                        Ok(_) => println!("Copied!"),
                                        Err(err) => anyhow::bail!(err),
                                    }
                                }
                                break 'ask_loop;
                            },
                            Some(DialogActions::Regenerate) => {
                                continue 'generate_loop;
                            },
                            Some(DialogActions::Ask) => {
                                input = None;
                                continue 'ask_loop;
                            },
                            _ => break 'ask_loop,
                        }
                    };
                }

                match &choices[..] {
                    [] => anyhow::bail!("no valid completions were generated"),
                    [choice] => {
                        spinner.stop_with_message(format!("{spinner_text}{}", choice.magenta()));
                        println!();

                        let actions = [
                            DialogActions::Execute {
                                command: choice.to_string(),
                                display: false,
                            },
                            DialogActions::Edit {
                                command: choice.to_string(),
                                display: false,
                            },
                            DialogActions::Regenerate,
                            DialogActions::Ask,
                            DialogActions::Cancel,
                        ];

                        let selected = dialoguer::Select::with_theme(&dialoguer_theme())
                            .default(0)
                            .items(&actions)
                            .interact_opt()?;

                        handle_action!(selected.and_then(|i| actions.get(i)));
                    },
                    choices => {
                        spinner.stop_with_message("".into());

                        let mut actions: Vec<_> = choices
                            .iter()
                            .map(|choice| DialogActions::Execute {
                                command: choice.to_string(),
                                display: true,
                            })
                            .collect();

                        actions.extend_from_slice(&[
                            DialogActions::Regenerate,
                            DialogActions::Ask,
                            DialogActions::Cancel,
                        ]);

                        let selected = dialoguer::Select::with_theme(&dialoguer_theme())
                            .default(0)
                            .items(&actions)
                            .interact_opt()?;

                        handle_action!(selected.and_then(|i| actions.get(i)));
                    },
                }
            }
        }

        Ok(())
    }
}
