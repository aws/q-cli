mod api;
mod parse;

use std::io::{
    stderr,
    Stderr,
    Write,
};
use std::time::Duration;

use color_eyre::owo_colors::OwoColorize;
use crossterm::style::{
    Attribute,
    Color,
    Print,
};
use crossterm::{
    cursor,
    style,
    terminal,
    ExecutableCommand,
    QueueableCommand,
};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use eyre::{
    bail,
    eyre,
    Result,
};
use fig_api_client::ai::{
    cw_endpoint,
    cw_streaming_client,
};
use fig_util::CLI_BINARY_NAME;
use spinners::{
    Spinner,
    Spinners,
};
use winnow::stream::Offset;
use winnow::Partial;

use self::api::send_message;
use self::parse::{
    interpret_markdown,
    ParseState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum ApiResponse {
    Text(String),
    ConversationId(String),
    MessageId(String),
    End,
    Error,
}

pub async fn chat(input: String) -> Result<()> {
    if !auth::is_logged_in().await {
        bail!(
            "You are not logged in, please log in with {}",
            format!("{CLI_BINARY_NAME} login",).bold()
        );
    }

    let mut stderr = stderr();
    let result = try_chat(&mut stderr, input).await;

    stderr
        .queue(style::SetAttribute(Attribute::Reset))?
        .queue(style::ResetColor)?
        .flush()
        .ok();

    result
}

async fn try_chat(stderr: &mut Stderr, mut input: String) -> Result<()> {
    let client = cw_streaming_client(cw_endpoint()).await;
    let mut rx = None;
    let mut conversation_id = None;
    let mut message_id = None;

    loop {
        // Make request with input
        if input.trim() == "exit" {
            return Ok(());
        }

        if !input.is_empty() {
            stderr.queue(style::SetForegroundColor(Color::Magenta))?;
            if input.contains("@history") {
                stderr.queue(style::Print("Using shell history\n"))?;
            }

            if input.contains("@git") {
                stderr.queue(style::Print("Using git context\n"))?;
            }

            if input.contains("@env") {
                stderr.queue(style::Print("Using environment\n"))?;
            }

            rx = Some(send_message(client.clone(), input, &conversation_id).await?);
            stderr
                .queue(style::SetForegroundColor(Color::Reset))?
                .execute(style::Print("\n"))?;
        } else {
            stderr.execute(style::Print(format!(
                "
Hi, I'm Amazon Q. I can answer questions about your shell and CLI tools!
You can include additional context by adding the following to your prompt:

{} to pass your shell history
{} to pass information about your current git repository
{} to pass your shell environment

",
                "@history".bold(),
                "@git".bold(),
                "@env".bold()
            )))?;
        }

        // Print response as we receive it
        if let Some(rx) = &mut rx {
            stderr.queue(cursor::Hide)?;
            let mut spinner = Some(Spinner::new(Spinners::Dots, "Generating your answer...".to_owned()));

            let mut buf = String::new();
            let mut offset = 0;
            let mut ended = false;

            let columns = crossterm::terminal::window_size()?.columns.into();
            let mut state = ParseState::new(columns);

            loop {
                if let Some(response) = rx.recv().await {
                    match response {
                        ApiResponse::Text(content) => match buf.is_empty() {
                            true => buf.push_str(content.trim_start()),
                            false => buf.push_str(&content),
                        },
                        ApiResponse::ConversationId(id) => conversation_id = Some(id),
                        ApiResponse::MessageId(id) => message_id = Some(id),
                        ApiResponse::End => ended = true,
                        ApiResponse::Error => {
                            drop(spinner.take());
                            stderr.execute(style::Print(
                                "Q is having trouble responding right now. Try again later.",
                            ))?;
                            ended = true;
                        },
                    }
                }

                // this is a hack since otherwise the parser might report Incomplete with useful data
                // still left in the buffer. I'm not sure how this is intended to be handled.
                if ended {
                    buf.push('\n');
                }

                if !buf.is_empty() && spinner.take().is_some() {
                    stderr
                        .queue(terminal::Clear(terminal::ClearType::CurrentLine))?
                        .queue(cursor::MoveToColumn(0))?
                        .queue(cursor::Show)?;
                }

                loop {
                    let input = Partial::new(&buf[offset..]);
                    match interpret_markdown(input, stderr as &mut Stderr, &mut state) {
                        Ok(parsed) => {
                            offset += parsed.offset_from(&input);
                            stderr.lock().flush()?;
                            state.newline = state.set_newline;
                            state.set_newline = false;
                        },
                        Err(err) => match err.into_inner() {
                            Some(err) => return Err(eyre!(err.to_string())),
                            None => break, // Data was incomplete
                        },
                    }

                    tokio::time::sleep(Duration::from_millis(5)).await;
                }

                if ended {
                    stderr
                        .queue(style::ResetColor)?
                        .queue(style::SetAttribute(Attribute::Reset))?
                        .queue(Print("\n"))?;

                    for (i, citation) in &state.citations {
                        stderr
                            .queue(style::SetForegroundColor(Color::Blue))?
                            .queue(style::Print(format!("{i} ")))?
                            .queue(style::SetForegroundColor(Color::DarkGrey))?
                            .queue(style::Print(format!("{citation}\n")))?
                            .queue(style::SetForegroundColor(Color::Reset))?;
                    }

                    if !state.citations.is_empty() {
                        stderr.execute(Print("\n"))?;
                    }

                    if let (Some(conversation_id), Some(message_id)) = (&conversation_id, &message_id) {
                        fig_telemetry::send_chat_added_message(conversation_id.to_owned(), message_id.to_owned()).await;
                    }

                    break;
                }
            }
        }

        // Prompt user for input
        const PROMPT: &str = ">";
        input = Input::with_theme(&ColorfulTheme {
            prompt_suffix: dialoguer::console::style(PROMPT.into()).magenta().bright(),
            ..ColorfulTheme::default()
        })
        .report(false)
        .interact_text()?;

        let lines = (input.len() + 3) / usize::from(crossterm::terminal::window_size()?.columns);
        if let Ok(lines) = u16::try_from(lines) {
            if lines > 0 {
                stderr.queue(cursor::MoveToPreviousLine(lines))?;
            }
        }

        stderr
            .queue(style::SetForegroundColor(Color::DarkGrey))?
            .queue(style::Print(format!("{PROMPT} {input}\n")))?
            .queue(style::SetForegroundColor(Color::Reset))?
            .flush()?;
    }
}
