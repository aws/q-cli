mod api;
mod parse;

use std::io::{
    stderr,
    Stderr,
    Write,
};
use std::time::Duration;

use crossterm::event::{
    Event,
    KeyCode,
    KeyEvent,
    KeyModifiers,
};
use crossterm::style::{
    Attribute,
    Print,
};
use crossterm::terminal::{
    self,
    disable_raw_mode,
    enable_raw_mode,
    is_raw_mode_enabled,
    ClearType,
};
use crossterm::{
    cursor,
    style,
    ExecutableCommand,
    QueueableCommand,
};
use eyre::{
    eyre,
    Result,
};
use fig_api_client::ai::cw_streaming_client;
use futures::{
    FutureExt,
    StreamExt,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{
    UnicodeWidthChar,
    UnicodeWidthStr,
};
use winnow::stream::{
    AsChar,
    Offset,
};
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
}

pub async fn chat() -> Result<()> {
    // check the user is using the amzn IdC instance
    if !auth::is_amzn_user().await? {
        eyre::bail!("chat is not currently implemented");
    }

    let mut stderr = stderr();
    stderr.execute(cursor::SetCursorStyle::BlinkingBar)?;

    let res = try_chat(&mut stderr).await;

    if is_raw_mode_enabled().unwrap_or(false) {
        disable_raw_mode()?;
    }

    // Restore terminal state
    stderr
        .queue(cursor::SetCursorStyle::DefaultUserShape)?
        .queue(style::SetAttribute(Attribute::Reset))?
        .queue(style::ResetColor)?
        .flush()?;

    res
}

async fn try_chat(stderr: &mut Stderr) -> Result<()> {
    let client = cw_streaming_client().await;
    let mut input = String::new();
    let mut rx = None;

    loop {
        // Make request with input
        if !input.is_empty() {
            rx = Some(send_message(client.clone(), input).await?);
            input = String::new();
        } else {
            stderr.execute(Print(
                "\nHi, I'm Amazon Q. I can answer your software development questions!\n\n",
            ))?;
        }

        // Print response as we receive it
        if let Some(rx) = &mut rx {
            let mut buf = String::new();
            let mut offset = 0;
            let mut ended = false;
            let mut state = ParseState::default();
            loop {
                if let Some(response) = rx.recv().await {
                    match response {
                        ApiResponse::Text(content) => {
                            buf.push_str(&content);
                        },
                        ApiResponse::End => ended = true,
                    }
                }

                loop {
                    let input = Partial::new(&buf[offset..]);
                    match interpret_markdown(input, stderr, &mut state) {
                        Ok(parsed) => {
                            offset += parsed.offset_from(&input);
                            stderr.lock().flush()?;
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
        let prompt = "> ";
        stderr.queue(Print(prompt))?.flush()?;
        enable_raw_mode()?;

        let mut cursor: usize = 0; // the byte index of the cursor
        let mut gcursor: usize = 0; // the grapheme index of the cursor
        let mut reader = crossterm::event::EventStream::new();
        while let Some(event) = reader.next().fuse().await {
            match event {
                Ok(event) => {
                    if let Event::Key(KeyEvent { code, modifiers, .. }) = event {
                        match code {
                            KeyCode::Backspace => {
                                if cursor > 0 {
                                    if let Some(grapheme) =
                                        input.graphemes(true).nth(gcursor - 1).map(ToOwned::to_owned)
                                    {
                                        input.replace_range(cursor - grapheme.len()..cursor, "");
                                        cursor -= grapheme.len();
                                        gcursor -= 1;
                                        stderr
                                            .queue(cursor::MoveLeft(grapheme.width().try_into()?))?
                                            .queue(cursor::SavePosition)?
                                            .queue(Print(&input[cursor..]))?
                                            .queue(terminal::Clear(ClearType::FromCursorDown))?
                                            .queue(cursor::RestorePosition)?;
                                    }
                                }
                            },
                            KeyCode::Enter => break,
                            KeyCode::Left => {
                                if cursor > 0 {
                                    if let Some(grapheme) = input.graphemes(true).nth(gcursor - 1) {
                                        cursor -= grapheme.len();
                                        gcursor -= 1;
                                        stderr.queue(cursor::MoveLeft(grapheme.width().try_into()?))?;
                                    }
                                }
                            },
                            KeyCode::Right => {
                                if let Some(grapheme) = input.graphemes(true).nth(gcursor) {
                                    cursor += grapheme.len();
                                    gcursor += 1;
                                    stderr.queue(cursor::MoveRight(grapheme.width().try_into()?))?;
                                }
                            },
                            KeyCode::Tab => {
                                let tab = "    ";
                                input.push_str(tab);
                                cursor += tab.len();
                                gcursor += 4;
                                stderr
                                    .queue(Print(tab))?
                                    .queue(cursor::SavePosition)?
                                    .queue(Print(&input[cursor..]))?
                                    .queue(terminal::Clear(ClearType::FromCursorDown))?
                                    .queue(cursor::RestorePosition)?;
                            },
                            KeyCode::Delete => {
                                if let Some(grapheme) = input.graphemes(true).nth(gcursor) {
                                    input.replace_range(cursor..cursor + grapheme.len(), "");
                                    stderr
                                        .queue(cursor::SavePosition)?
                                        .queue(Print(&input[cursor..]))?
                                        .queue(terminal::Clear(ClearType::FromCursorDown))?
                                        .queue(cursor::RestorePosition)?;
                                }
                            },
                            KeyCode::Char(c) => {
                                if modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                                    return Ok(());
                                }

                                if c.width().is_some() {
                                    input.insert(cursor, c);
                                    cursor += c.len();
                                    gcursor += 1;
                                    stderr
                                        .queue(Print(c))?
                                        .queue(cursor::SavePosition)?
                                        .queue(Print(&input[cursor..]))?
                                        .queue(terminal::Clear(ClearType::FromCursorDown))?
                                        .queue(cursor::RestorePosition)?;
                                }
                            },
                            KeyCode::Esc => return Ok(()),
                            _ => (),
                        }
                    }
                },
                Err(err) => {
                    writeln!(stderr, "Error: {err:?}")?;
                },
            }

            stderr.flush()?;
        }

        disable_raw_mode()?;
        stderr.execute(Print("\n\n"))?;
    }
}
