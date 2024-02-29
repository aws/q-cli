mod api;
mod parse;

use std::io::{
    stderr,
    Stderr,
    Write,
};
use std::time::Duration;

use crossterm::style::{
    Attribute,
    Color,
    Print,
};
use crossterm::{
    style,
    ExecutableCommand,
    QueueableCommand,
};
use dialoguer::Input;
use eyre::{
    eyre,
    Result,
};
use fig_api_client::ai::{
    cw_endpoint,
    cw_streaming_client,
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
    End,
    Error,
}

pub async fn chat(input: String) -> Result<()> {
    // check the user is using the amzn IdC instance
    if !auth::is_amzn_user().await? {
        eyre::bail!("chat is not currently implemented");
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

    loop {
        // Make request with input
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

            rx = Some(send_message(client.clone(), input).await?);
            stderr
                .queue(style::SetForegroundColor(Color::Reset))?
                .execute(style::Print("\n"))?;
        } else {
            stderr.execute(style::Print(
                "
Hi, I'm Amazon Q. I can answer your software development questions!
You can include additional context from your shell by typing @history, @git or @env in your prompt.

",
            ))?;
        }

        // Print response as we receive it
        if let Some(rx) = &mut rx {
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
                        ApiResponse::End => ended = true,
                        ApiResponse::Error => {
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
                    buf.push(' ');
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
                        .queue(Print("\n\n"))?
                        .flush()?;
                    break;
                }
            }
        }

        // Prompt user for input
        input = Input::new().report(true).with_prompt("?").interact_text()?;
    }
}
