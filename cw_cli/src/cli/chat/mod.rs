mod parse;

use std::io::{
    stderr,
    Stderr,
    Write,
};
use std::time::Duration;

use amzn_codewhisperer_streaming_client::types::{
    ChatMessage,
    ChatResponseStream,
    ChatTriggerType,
    ConversationState,
    EditorState,
    Position,
    ProgrammingLanguage,
    TextDocument,
    UserInputMessage,
    UserInputMessageContext,
};
use amzn_codewhisperer_streaming_client::Client;
use crossterm::event::{
    Event,
    KeyCode,
    KeyEvent,
    KeyModifiers,
};
use crossterm::style::Attribute;
use crossterm::terminal::{
    self,
    disable_raw_mode,
    enable_raw_mode,
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
use tokio::sync::mpsc::{
    Receiver,
    Sender,
};
use tracing::error;
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

use self::parse::{
    interpret_markdown,
    ParseState,
};

enum ApiResponse {
    Text(String),
    End,
}

pub async fn chat() -> Result<()> {
    let mut stderr = stderr();
    stderr.execute(cursor::SetCursorStyle::BlinkingBar)?;

    let res = try_chat(&mut stderr).await;

    // Try to disable even if not enabled
    disable_raw_mode()?;

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
            rx = Some(send_message(client.clone(), input)?);
            input = String::new();
        } else {
            stderr.execute(style::Print(
                "\n\nHi, I'm Amazon Q. I can answer your software development questions!\n\n",
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
                            // dbg!(content.cyan());
                            buf.push_str(&content);
                        },
                        ApiResponse::End => ended = true,
                    }
                }

                loop {
                    let input = Partial::new(&buf[offset..]);
                    match interpret_markdown(input, stderr, &mut state) {
                        Ok((parsed, _)) => {
                            offset += parsed.offset_from(&input);
                            stderr.lock().flush()?;
                        },
                        Err(err) => match err.into_inner() {
                            Some(err) => return Err(eyre!(err.to_string())),
                            None => break, // Data was incomplete
                        },
                    }

                    std::thread::sleep(Duration::from_millis(5));
                }

                if ended {
                    stderr
                        .queue(style::ResetColor)?
                        .queue(style::SetAttribute(Attribute::Reset))?
                        .queue(style::Print("\n\n"))?
                        .flush()?;
                    break;
                }
            }
        }

        // Prompt user for input
        let prompt = "> ";
        stderr.queue(style::Print(prompt))?.flush()?;
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
                                            .queue(style::Print(&input[cursor..]))?
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
                                    .queue(style::Print(tab))?
                                    .queue(cursor::SavePosition)?
                                    .queue(style::Print(&input[cursor..]))?
                                    .queue(terminal::Clear(ClearType::FromCursorDown))?
                                    .queue(cursor::RestorePosition)?;
                            },
                            KeyCode::Delete => {
                                if let Some(grapheme) = input.graphemes(true).nth(gcursor) {
                                    input.replace_range(cursor..cursor + grapheme.len(), "");
                                    stderr
                                        .queue(cursor::SavePosition)?
                                        .queue(style::Print(&input[cursor..]))?
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
                                        .queue(style::Print(c))?
                                        .queue(cursor::SavePosition)?
                                        .queue(style::Print(&input[cursor..]))?
                                        .queue(terminal::Clear(ClearType::FromCursorDown))?
                                        .queue(cursor::RestorePosition)?;
                                }
                            },
                            KeyCode::Esc => return Ok(()),
                            _ => (),
                        }
                    }
                },
                Err(e) => println!("Error: {:?}\r", e),
            }

            stderr.flush()?;
        }

        disable_raw_mode()?;
        stderr.execute(style::Print("\n\n"))?;
    }
}

fn send_message(client: Client, input: String) -> Result<Receiver<ApiResponse>> {
    let (tx, rx) = tokio::sync::mpsc::channel(8);

    let programming_language = ProgrammingLanguage::builder().language_name("shell").build()?;

    let text_document = TextDocument::builder()
        .text("#!/bin/bash\n\n")
        .relative_file_path("test.sh")
        .programming_language(programming_language)
        .build()?;

    let editor_state = EditorState::builder()
        .document(text_document)
        .cursor_state(amzn_codewhisperer_streaming_client::types::CursorState::Position(
            Position::builder().line(2).character(0).build()?,
        ))
        .build();

    let user_input_message_context = UserInputMessageContext::builder().editor_state(editor_state).build();

    let user_input_message = UserInputMessage::builder()
        .content(input)
        .user_input_message_context(user_input_message_context)
        .user_intent(amzn_codewhisperer_streaming_client::types::UserIntent::ImproveCode)
        .build()?;

    let conversation_state = ConversationState::builder()
        .current_message(ChatMessage::UserInputMessage(user_input_message))
        .chat_trigger_type(ChatTriggerType::Manual)
        .build()?;

    tokio::spawn(async move {
        if let Err(err) = try_send_message(client, &tx, conversation_state).await {
            error!(%err);
        }

        // Try to end stream
        tx.send(ApiResponse::End).await.ok();
    });

    Ok(rx)
}

async fn try_send_message(
    client: Client,
    tx: &Sender<ApiResponse>,
    conversation_state: ConversationState,
) -> Result<()> {
    let mut res = client
        .generate_assistant_response()
        .conversation_state(conversation_state)
        .send()
        .await?;

    while let Ok(Some(a)) = res.generate_assistant_response_response.recv().await {
        match a {
            ChatResponseStream::MessageMetadataEvent(_response) => {},
            ChatResponseStream::AssistantResponseEvent(response) => {
                tx.send(ApiResponse::Text(response.content)).await?;
            },
            ChatResponseStream::FollowupPromptEvent(_response) => {
                // let followup = response.followup_prompt()?;
                // println!("content: {}", followup.content());
                // println!("intent: {:?}", followup.user_intent());
            },
            ChatResponseStream::CodeReferenceEvent(_) => {},
            ChatResponseStream::SupplementaryWebLinksEvent(_) => {},
            _ => {},
        }
    }

    Ok(())
}
