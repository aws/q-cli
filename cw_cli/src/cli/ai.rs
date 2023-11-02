use std::fmt::Display;
use std::io::{
    stdout,
    IsTerminal,
};

use arboard::Clipboard;
use clap::Args;
use color_eyre::owo_colors::OwoColorize;
use crossterm::style::Stylize;
use dialoguer::theme::ColorfulTheme;
use fig_api_client::ai::{
    request_cw,
    CodewhipererFileContext,
    CodewhipererRequest,
    LanguageName,
    ProgrammingLanguage,
};
use fig_ipc::{
    BufferedUnixStream,
    SendMessage,
};
use once_cell::sync::Lazy;
use regex::{
    Captures,
    Regex,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::util::spinner::{
    Spinner,
    SpinnerComponent,
};

const SEEN_ONBOARDING_KEY: &str = "ai.seen-onboarding";

#[derive(Debug, Args, PartialEq, Eq)]
pub struct AiArgs {
    input: Vec<String>,
    /// Number of completions to generate (must be <=5)
    #[arg(short, long, hide = true)]
    n: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Choice {
    text: Option<String>,
    additional_message: Option<String>,
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
        ..crate::util::dialoguer_theme()
    }
}

async fn send_figterm(text: String, execute: bool) -> eyre::Result<()> {
    let session_id = std::env::var("CWTERM_SESSION_ID")?;
    let mut conn = BufferedUnixStream::connect(fig_util::directories::figterm_socket_path(&session_id)?).await?;
    conn.send_message(fig_proto::figterm::FigtermRequestMessage {
        request: Some(fig_proto::figterm::figterm_request_message::Request::InsertOnNewCmd(
            fig_proto::figterm::InsertOnNewCmdRequest {
                text,
                execute,
                bracketed: true,
            },
        )),
    })
    .await?;
    Ok(())
}

async fn generate_response(question: &str, n: i32) -> eyre::Result<Vec<String>> {
    let response = request_cw(CodewhipererRequest {
        file_context: CodewhipererFileContext {
            left_file_content: format!(
                "# A collection of macOS shell one-liners that can be run interactively

# what is the capital of australia
# UNIMPLEMENTED: not related to terminal

# ddos the ip address 192.0.1.15
# UNIMPLEMENTED: harmful

# write a loop to list files
# UNIMPLEMENTED: multiple lines

# List files
ls -l

# Count files in current directory
ls -l | wc -l

# Disk space used by home directory
du ~

# Replace foo with bar in all .py files
sed 's/foo/bar/g' *.py

# Add all files to git and create a commit with the message \"feat: add new route\"
git add -A && git commit -m 'feat: add new route'

# Add all files to git and create a commit
git add -A && git commit -m \"$MESSAGE\"

# Delete the models subdirectory
rm -rf ./models

# Delete a subdirectory
rm -rf $DIRECTORY

# What folder am I in?
pwd

# install vscode
brew install visual-studio-code

# list all files on my desktop
ls ~/Desktop

# list all installed applications
find / -iname '*.app'

# hide all icons on desktop
defaults write com.apple.finder CreateDesktop -bool false && killall Finder

# transform a file to mp4 with ffmpeg
ffmpeg -i \"$IN_FILE\" \"$OUT_NAME.mp4\"

# edit the main.c file with vim
vim main.c

# {question}\n"
            ),
            right_file_content: "".into(),
            filename: "commands.sh".into(),
            programming_language: ProgrammingLanguage {
                language_name: LanguageName::Shell,
            },
        },
        max_results: n,
        next_token: None,
    })
    .await?;

    // println!("{:?}", response);

    Ok(response
        .completions
        .unwrap_or_default()
        .into_iter()
        .filter_map(|c| c.content)
        .collect())
}

fn warning_message(content: &str) {
    #[allow(clippy::type_complexity)]
    let warnings: &[(Regex, fn(&Captures) -> String)] = &[
        (Regex::new(r"\bsudo\b").unwrap(), |_m| {
            "⚠️ Warning: this command contains sudo which will run the command as admin, please make sure you know what you are doing before you run this...".into()
        }),
        (
            Regex::new(r"\s+(--hard|--force|-rf|--no-preserve-root)\b").unwrap(),
            |m| {
                format!(
                    "⚠️ Warning: this command contains an irreversible flag ({}), please make sure you know what you are doing before you run this...",
                    &m[0]
                )
            },
        ),
        (Regex::new(r"(\s*\/dev\/(\w*)(\s|$))").unwrap(), |_m| {
            "⚠️ Warning: this command may override one of your disks, please make sure you know what you are doing before you run this...".into()
        }),
        (
            Regex::new(r":\s*\(\s*\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:").unwrap(),
            |_m| "⚠️ Warning: this command is a fork bomb".into(),
        ),
        (Regex::new(r"\bdd\b").unwrap(), |_m| {
            "⚠️ Warning: dd is a dangerous command, please make sure you know what you are doing before you run it..."
                .into()
        }),
        (Regex::new(r"\|\s*(bash|sh|zsh|fish)/").unwrap(), |m| {
            format!(
                "⚠️ Warning: piping into {} can be dangerous, please make sure you know what you are doing before you run it...",
                &m[0]
            )
        }),
        (Regex::new(r"(\bsudoedit|\bsu|\/etc\/sudoers)\b/").unwrap(), |m| {
            format!(
                "⚠️ Warning: you might be altering root/sudo files with ${}`, please make sure you know what you are doing before you run it...",
                &m[0]
            )
        }),
        (Regex::new(r"(\/dev\/(u?random)|(zero))").unwrap(), |m| {
            format!(
                "⚠️ Warning: {} can be dangerous, please make sure you know what you are doing before you run this...",
                &m[0]
            )
        }),
    ];

    for (re, warning) in warnings {
        if let Some(capture) = re.captures(content) {
            println!("{}\n", warning(&capture).yellow().bold());
        }
    }
}

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\$[A-Za-z0-9\_\-]+)").unwrap());

fn highlighter(s: &str) -> String {
    PARAM_REGEX
        .replace_all(s, |a: &Captures<'_>| {
            let env = a[0].strip_prefix('$').unwrap();
            if std::env::var_os(env).is_some() {
                a[0].into()
            } else {
                (&a[0]).bright_magenta().to_string()
            }
        })
        .into_owned()
}

impl AiArgs {
    pub async fn execute(self) -> eyre::Result<()> {
        let interactive = std::io::stdin().is_terminal();

        // show onboarding if it hasnt been seen
        let seen_onboarding = fig_settings::state::get_bool_or(SEEN_ONBOARDING_KEY, false);
        if !seen_onboarding && interactive {
            eprintln!();
            eprintln!(
                "  Translate {} to {} commands. Run in any shell.",
                "English".bold(),
                "Shell".bold()
            );
            fig_settings::state::set_value(SEEN_ONBOARDING_KEY, true).ok();
        }
        if interactive {
            eprintln!();
        }

        let Self { input, n } = self;
        let mut input = if input.is_empty() { None } else { Some(input.join(" ")) };

        let n = match n {
            Some(n) if n >= 0 || n > 5 => {
                eyre::bail!("n must be 0 < n <= 5");
            },
            Some(n) => n,
            None => 1,
        };

        if !interactive {
            let question = match input {
                Some(_) => {
                    eyre::bail!("only input on stdin is supported when stdin is not a tty")
                },
                None => {
                    let stdin = std::io::stdin();
                    let mut question = String::new();
                    stdin.read_line(&mut question)?;
                    question
                },
            };

            match &generate_response(&question, 1).await?[..] {
                [] => eyre::bail!("no valid completions were generated"),
                [res, ..] => {
                    println!("{res}");
                    return Ok(());
                },
            };
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
                    println!("{}", "Translate Text to Shell".bold());
                    println!();

                    dialoguer::Input::with_theme(&theme())
                        .with_prompt("Text")
                        .interact_text()?
                },
            };

            let question = question.trim().replace('\n', " ");

            'generate_loop: loop {
                let spinner_text = format!("  {} {} ", "Shell".bold(), "·".grey());

                let mut spinner = Spinner::new(vec![
                    SpinnerComponent::Text(spinner_text.clone()),
                    SpinnerComponent::Spinner,
                ]);

                let choices = generate_response(&question, n).await?;

                macro_rules! handle_action {
                    ($action:expr) => {
                        let accepted = matches!(&$action, &Some(DialogActions::Execute { .. }));
                        fig_telemetry::send_translation_actioned(accepted).await;

                        match $action {
                            Some(DialogActions::Execute { command, .. }) => {
                                // let command = PARAM_REGEX
                                //     .replace_all(command, |a: &Captures<'_>| {
                                //         let env = a[0].strip_prefix("$").unwrap();
                                //         if std::env::var_os(env).is_some() {
                                //             a[0].to_string()
                                //         } else {
                                //             dialoguer::Input::with_theme(&theme())
                                //                 .with_prompt(env)
                                //                 .with_prompt(format!("{env}"))
                                //                 .interact_text()
                                //                 .unwrap()
                                //         }
                                //     })
                                //     .to_string();

                                if send_figterm(command.clone(), true).await.is_err() {
                                    let mut child = tokio::process::Command::new("bash")
                                        .arg("-c")
                                        .arg(command)
                                        .spawn()?;
                                    child.wait().await?;
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
                    [choice, ..] => {
                        if let Some(error_reason) = choice.strip_prefix("# UNIMPLEMENTED: ") {
                            spinner.stop_with_message(format!("{spinner_text}❌"));
                            eyre::bail!("{}", error_reason);
                        }

                        spinner.stop_with_message(format!("{spinner_text}{}", highlighter(choice)));
                        println!();
                        warning_message(choice);

                        let actions: Vec<DialogActions> = fig_settings::settings::get("ai.menu-actions")
                            .ok()
                            .flatten()
                            .unwrap_or_else(|| {
                                ["execute", "edit", "regenerate", "ask", "cancel"]
                                    .map(String::from)
                                    .to_vec()
                            })
                            .into_iter()
                            .filter_map(|action| match action.as_str() {
                                "execute" => Some(DialogActions::Execute {
                                    command: choice.to_string(),
                                    display: false,
                                }),
                                "edit" => Some(DialogActions::Edit {
                                    command: choice.to_string(),
                                    display: false,
                                }),
                                "copy" => Some(DialogActions::Copy {
                                    command: choice.to_string(),
                                    display: false,
                                }),
                                "regenerate" => Some(DialogActions::Regenerate),
                                "ask" => Some(DialogActions::Ask),
                                "cancel" => Some(DialogActions::Cancel),
                                _ => None,
                            })
                            .collect();

                        let selected = dialoguer::Select::with_theme(&crate::util::dialoguer_theme())
                            .default(0)
                            .items(&actions)
                            .interact_opt()?;

                        handle_action!(selected.and_then(|i| actions.get(i)));
                    },
                    // choices => {
                    //     spinner.stop_with_message(format!("{spinner_text}{}", "<multiple options>".dark_grey()));
                    //     println!();

                    //     let mut actions: Vec<_> = choices
                    //         .iter()
                    //         .map(|choice| DialogActions::Execute {
                    //             command: choice.to_string(),
                    //             display: true,
                    //         })
                    //         .collect();

                    //     actions.extend_from_slice(&[
                    //         DialogActions::Regenerate,
                    //         DialogActions::Ask,
                    //         DialogActions::Cancel,
                    //     ]);

                    //     let selected = dialoguer::Select::with_theme(&crate::util::dialoguer_theme())
                    //         .default(0)
                    //         .items(&actions)
                    //         .interact_opt()?;

                    //     handle_action!(selected.and_then(|i| actions.get(i)));
                    // },
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lints() {
        warning_message("sudo dd if=/dev/sda");
    }

    #[test]
    fn test_highlighter() {
        println!("{}", highlighter("echo $PATH $ABC $USER $HOME $DEF"));
    }
}
