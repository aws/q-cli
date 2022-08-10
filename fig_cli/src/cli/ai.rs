use std::fmt::Display;
use std::io::stdout;

use arboard::Clipboard;
use clap::Args;
use color_eyre::owo_colors::OwoColorize;
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

const SEEN_ONBOARDING_KEY: &str = "ai.seen-onboarding";
const IS_FIG_PRO_KEY: &str = "user.account.is-fig-pro";

#[derive(Debug, Args)]
pub struct AiArgs {
    #[clap(value_parser)]
    input: Vec<String>,
    /// Number of completions to generate (must be <=5)
    #[clap(short, long, value_parser, hide = true)]
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
                    write!(f, "⚡ Execute {}", command.bright_magenta())
                } else {
                    write!(f, "⚡ Execute command")
                }
            },
            DialogActions::Edit { command, display } => {
                if *display {
                    write!(f, "📝 Edit {}", command.bright_magenta())
                } else {
                    write!(f, "📝 Edit command")
                }
            },
            DialogActions::Copy { command, display } => {
                if *display {
                    write!(f, "📋 Copy {}", command.bright_magenta())
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

async fn send_figterm(text: String, execute: bool) -> eyre::Result<()> {
    let session_id = std::env::var("TERM_SESSION_ID")?;
    let mut conn = fig_ipc::connect(fig_util::directories::figterm_socket_path(&session_id)?).await?;
    fig_ipc::send_message(&mut conn, fig_proto::figterm::FigtermMessage {
        command: Some(fig_proto::figterm::figterm_message::Command::InsertOnNewCmdCommand(
            fig_proto::figterm::InsertOnNewCmdCommand {
                text: format!("\x1b[200~{text}\x1b[201~"),
                execute,
            },
        )),
    })
    .await?;
    Ok(())
}

impl AiArgs {
    pub async fn execute(self) -> eyre::Result<()> {
        // Spawn task to get `fig pro` status
        tokio::spawn(async {
            let is_pro = fig_api_client::user::plans()
                .await
                .map(|plan| plan.highest_plan())
                .unwrap_or_default()
                .is_pro();
            fig_settings::state::set_value(IS_FIG_PRO_KEY, is_pro).ok();
        });

        // show onboarding if it hasnt been seen, show fig pro to non pro users
        let seen_onboarding = fig_settings::state::get_bool_or(SEEN_ONBOARDING_KEY, false);
        let is_fig_pro = fig_settings::state::get_bool_or(IS_FIG_PRO_KEY, false);

        if !seen_onboarding {
            println!();
            println!(
                "  Translate {} to {} commands. Run in any shell.",
                "English".bold(),
                "Bash".bold()
            );
            println!();
            println!(
                "  {} translates your English instructions to Bash syntax and commands.",
                "fig ai".bright_magenta().bold()
            );
            println!("  You can run the command in any shell.");

            fig_settings::state::set_value(SEEN_ONBOARDING_KEY, true).ok();
        }

        if !seen_onboarding || !is_fig_pro {
            println!();
            println!(
                "  {} is currently in beta for {} users.",
                "fig ai".bright_magenta().bold(),
                "Fig Pro".bold()
            );
            println!("  Run {} to learn more...", "fig pro".bright_magenta().bold());
        }

        println!();

        let Self { input, n } = self;
        let mut input = if input.is_empty() { None } else { Some(input.join(" ")) };

        if n.map(|n| n > 5).unwrap_or_default() {
            eyre::bail!("n must be <= 5");
        }

        // hack to show cursor which dialoguer eats
        tokio::spawn(async {
            tokio::signal::ctrl_c().await.unwrap();
            crossterm::execute!(stdout(), crossterm::cursor::Show).unwrap();
            std::process::exit(0);
        });

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
                eyre::bail!("input is >120 characters");
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
                                if let Err(err) = send_figterm(command.to_owned(), true).await {
                                    println!("{} {err}", "Failed to execute command:".bright_red().bold());
                                    println!();
                                    println!("Command: {command}");
                                }
                                break 'ask_loop;
                            },
                            Some(DialogActions::Edit { command, .. }) => {
                                if let Err(err) = send_figterm(command.to_owned(), false).await {
                                    println!("{} {err}", "Failed to insert command:".bright_red().bold());
                                    println!();
                                    println!("Command: {command}");
                                }
                                break 'ask_loop;
                            },
                            Some(DialogActions::Copy { command, .. }) => {
                                if let Ok(mut clipboard) = Clipboard::new() {
                                    match clipboard.set_text(command.to_string()) {
                                        Ok(_) => println!("Copied!"),
                                        Err(err) => eyre::bail!(err),
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
                    [] => {
                        spinner.stop_with_message(format!("{spinner_text}❌"));
                        eyre::bail!("no valid completions were generated");
                    },
                    [choice] => {
                        spinner.stop_with_message(format!("{spinner_text}{}", choice.bright_magenta()));
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
                        spinner.stop_with_message(format!("{spinner_text}{}", "<multiple options>".dark_grey()));
                        println!();

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
