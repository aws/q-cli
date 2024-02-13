mod parse;

use std::io::{
    self,
    Stdout,
};
use std::marker::PhantomData;

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
use crossterm::cursor::{
    DisableBlinking,
    EnableBlinking,
    SetCursorStyle,
};
use crossterm::event::{
    Event,
    KeyCode,
    KeyEventKind,
    KeyModifiers,
};
use eyre::Result;
use fig_api_client::ai::cw_streaming_client;
use futures::{
    pending,
    FutureExt,
    StreamExt,
};
use ratatui::backend::{
    Backend,
    CrosstermBackend,
};
use ratatui::layout::{
    Constraint,
    Direction,
    Layout,
    Rect,
};
use ratatui::style::{
    Color,
    Modifier,
    Style,
};
use ratatui::text::{
    Line,
    Span,
    Text,
};
use ratatui::widgets::{
    Block,
    Borders,
    Paragraph,
    Widget as _,
};
use ratatui::{
    Frame,
    Terminal,
    TerminalOptions,
};
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlFlow {
    Continue,
    Exit,
}

enum Message {
    User(String),
    Assistant(String),
}

impl Message {
    fn text(&self) -> &str {
        match self {
            Message::User(text) => text,
            Message::Assistant(text) => text,
        }
    }
}

enum ApiResponse {
    Text { idx: usize, content: String },
    End,
}

/// App holds the state of the application
struct App<B: Backend> {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    cursor_position: usize,
    rx: Option<Receiver<ApiResponse>>,
    client: &'static Client,
    _backend: PhantomData<B>,
}

impl<B> App<B>
where
    B: Backend,
{
    async fn new() -> App<B> {
        let client = cw_streaming_client().await;
        App {
            input: String::new(),
            cursor_position: 0,
            rx: None,
            client,
            _backend: PhantomData::default(),
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn insert_paragraph_before(
        &self,
        terminal: &mut Terminal<B>,
        height: u16,
        paragraph: Paragraph<'_>,
    ) -> io::Result<()> {
        terminal.insert_before(height, |buf| {
            paragraph.render(
                Rect {
                    x: buf.area.x + 1,
                    width: buf.area.width - 1,
                    ..buf.area
                },
                buf,
            );
        })
    }

    fn insert_whole_message_before(&mut self, terminal: &mut Terminal<B>, message: Message) -> io::Result<()> {
        let size = terminal.size()?;

        let wrapped_text = textwrap::wrap(message.text(), size.width as usize - 1);
        let height = 3 + wrapped_text.len() as u16;

        let mut lines = Vec::with_capacity(height as usize);
        lines.extend([
            Line::from(""),
            Line::from(vec![Span::styled(
                match message {
                    Message::User(_) => "❯ You",
                    Message::Assistant(_) => "❯ Q",
                },
                Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            )]),
        ]);
        lines.extend(wrapped_text.iter().map(|text| Line::from(text.to_string())));

        self.insert_paragraph_before(terminal, height, Paragraph::new(lines))?;

        Ok(())
    }

    fn insert_message_text_before(&mut self, terminal: &mut Terminal<B>, message: &str) -> io::Result<()> {
        let size = terminal.size()?;

        let wrapped_text = textwrap::wrap(message, size.width as usize - 1);
        let height = wrapped_text.len() as u16;

        let mut lines = Vec::with_capacity(height as usize);
        lines.extend(wrapped_text.iter().map(|text| Line::from(text.to_string())));

        self.insert_paragraph_before(terminal, height, Paragraph::new(lines))?;

        Ok(())
    }

    fn insert_assistant_message_start(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        let lines = vec![
            Line::from(""),
            Line::styled("❯ Q", Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ];

        self.insert_paragraph_before(terminal, lines.len() as u16, Paragraph::new(lines))?;
        Ok(())
    }

    fn submit_message(&mut self, terminal: &mut Terminal<B>) -> io::Result<String> {
        let input = std::mem::take(&mut self.input);
        self.insert_whole_message_before(terminal, Message::User(input.clone()))?;
        self.insert_assistant_message_start(terminal)?;
        self.reset_cursor();
        Ok(input)
    }

    async fn next_assistant_message(&mut self) -> Option<ApiResponse> {
        let res = match self.rx {
            Some(ref mut rx) => rx.recv().await,
            None => {
                pending!();
                None
            },
        };

        if res.is_none() {
            self.rx = None;
        }

        res
    }

    fn init(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        self.insert_whole_message_before(
            terminal,
            Message::Assistant("Hi, I'm Amazon Q. I can answer your software development questions".into()),
        )?;
        Ok(())
    }

    fn on_event(&mut self, event: Event, terminal: &mut Terminal<B>) -> io::Result<ControlFlow> {
        match event {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            if self.rx.is_none() {
                                let message = self.submit_message(terminal)?;
                                self.rx = Some(send_message(self.client.clone(), message));
                            }
                        },
                        KeyCode::Char('c' | 'd') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(ControlFlow::Exit);
                        },
                        KeyCode::Char(to_insert) => {
                            self.enter_char(to_insert);
                        },
                        KeyCode::Backspace => {
                            self.delete_char();
                        },
                        KeyCode::Left => {
                            self.move_cursor_left();
                        },
                        KeyCode::Right => {
                            self.move_cursor_right();
                        },
                        _ => {},
                    }
                }
            },
            Event::Paste(s) => {
                self.input.push_str(&s);
            },
            // Event::Resize(width, height) => {
            //     terminal.resize(Rect::new(0, 0, width, height))?;
            // },
            _ => {},
        };

        Ok(ControlFlow::Continue)
    }

    fn draw(&self, f: &mut Frame<'_>) {
        use ratatui::style::Stylize;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Length(1)])
            .split(f.size());

        let mut text = Text::from(Line::from(
            ["Press ".into(), "ctrl+c".bold(), " to exit".into()].to_vec(),
        ));
        text.patch_style(Style::default().add_modifier(Modifier::RAPID_BLINK));

        let help_message = Paragraph::new(text);
        f.render_widget(help_message, chunks[2]);

        let mut input_text_style = Style::new();
        if self.input.is_empty() {
            input_text_style = input_text_style.dim();
        }

        let input_text = Text::styled(
            if self.input.is_empty() {
                "Message Q..."
            } else {
                self.input.as_str()
            },
            input_text_style,
        );

        let input = Paragraph::new(input_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(input, chunks[1]);

        // // Make the cursor visible and ask ratatui to put it at the specified coordinates
        // after // rendering
        // f.set_cursor(
        //     // Draw the cursor at the current position in the input field.
        //     // This position is can be controlled via the left and right arrow key
        //     chunks[1].x + app.cursor_position as u16 + 1,
        //     // Move one line down, from the border to the input line
        //     chunks[1].y + 1,
        // );

        // // let messages =
        // // List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"
        // )); // f.render_widget(messages, chunks[2]);

        // let items = app.messages.iter().rev().fold(Text::from(""), |mut acc, message| {
        //     match message {
        //         Message::User(s) => {
        //             acc.extend(["❯ You".bold()]);
        //             acc.extend(Text::from(s.trim().to_owned()));
        //             acc.extend([""]);
        //         },
        //         Message::Assistant(s) => {
        //             acc.extend(["❯ Q".bold().magenta()]);
        //             acc.extend(Text::from(s.trim_start().to_owned()));
        //             acc.extend([""]);
        //         },
        //     };
        //     acc
        // });

        // let mut paragraph = Paragraph::new(items.clone())
        //     .wrap(Wrap { trim: false })
        //     .block(Block::new().borders(Borders::RIGHT));

        // let paragraph_height = paragraph.line_count(chunks[0].width);
        // app.scroll_pos = paragraph_height.saturating_sub(chunks[0].height as usize);

        // paragraph = paragraph.scroll((app.scroll_pos as u16, 0));

        // let scrollbar = Scrollbar::default()
        //     .orientation(ScrollbarOrientation::VerticalRight)
        //     .begin_symbol(Some("↑"))
        //     .end_symbol(Some("↓"));

        // // let mut scrollbar_state =
        // ScrollbarState::new(paragraph_height).position(app.scroll_pos);

        // f.render_widget(paragraph, chunks[0]);
        // // f.render_stateful_widget(
        // //     scrollbar,
        // //     chunks[0].inner(&Margin {
        // //         vertical: 1,
        // //         horizontal: 0,
        // //     }),
        // //     &mut scrollbar_state,
        // // );
    }
}

fn setup_terminal() -> std::io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut backend = CrosstermBackend::new(io::stdout());
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(backend, EnableBlinking, SetCursorStyle::BlinkingBar)?;
    Terminal::with_options(backend, TerminalOptions {
        viewport: ratatui::Viewport::Inline(5),
    })
}

fn teardown_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> std::io::Result<()> {
    let backend = terminal.backend_mut();
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(backend, DisableBlinking, SetCursorStyle::DefaultUserShape)?;
    Ok(())
}

fn send_message(client: Client, input: String) -> Receiver<ApiResponse> {
    let (tx, rx) = tokio::sync::mpsc::channel(8);

    tokio::spawn(async move {
        let mut res = client
            .generate_assistant_response()
            .conversation_state(
                ConversationState::builder()
                    .current_message(ChatMessage::UserInputMessage(
                        UserInputMessage::builder()
                            .content(input)
                            .user_input_message_context(
                                UserInputMessageContext::builder()
                                    .editor_state(
                                        EditorState::builder()
                                            .document(
                                                TextDocument::builder()
                                                    .text("#!/bin/bash\n\n")
                                                    .relative_file_path("test.sh")
                                                    .programming_language(
                                                        ProgrammingLanguage::builder()
                                                            .language_name("shell")
                                                            .build()
                                                            .unwrap(),
                                                    )
                                                    .build()
                                                    .unwrap(),
                                            )
                                            .cursor_state(
                                                amzn_codewhisperer_streaming_client::types::CursorState::Position(
                                                    Position::builder().line(2).character(0).build().unwrap(),
                                                ),
                                            )
                                            .build(),
                                    )
                                    .build(),
                            )
                            .user_intent(amzn_codewhisperer_streaming_client::types::UserIntent::ImproveCode)
                            .build()
                            .unwrap(),
                    ))
                    .chat_trigger_type(ChatTriggerType::Manual)
                    .build()
                    .unwrap(),
            )
            .send()
            .await
            .unwrap();

        let mut idx = 0;

        while let Ok(Some(a)) = res.generate_assistant_response_response.recv().await {
            match a {
                ChatResponseStream::MessageMetadataEvent(_response) => {},
                ChatResponseStream::AssistantResponseEvent(response) => {
                    tx.send(ApiResponse::Text {
                        idx,
                        content: response.content,
                    })
                    .await
                    .unwrap();
                },
                ChatResponseStream::FollowupPromptEvent(_response) => {
                    // let followup = response.followup_prompt().unwrap();
                    // println!("content: {}", followup.content());
                    // println!("intent: {:?}", followup.user_intent());
                },
                ChatResponseStream::CodeReferenceEvent(_) => {},
                ChatResponseStream::SupplementaryWebLinksEvent(_) => {},
                _ => {},
            }

            idx += 1;
        }

        tx.send(ApiResponse::End).await.unwrap();
    });

    rx
}

pub async fn chat() -> Result<()> {
    let mut terminal = setup_terminal()?;

    let app = App::new().await;
    let res = run_app(&mut terminal, app).await;

    teardown_terminal(terminal)?;

    res.map_err(Into::into)
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App<B>) -> io::Result<()> {
    let mut stream = crossterm::event::EventStream::new();

    app.init(terminal)?;

    loop {
        terminal.draw(|f| app.draw(f))?;
        let event = stream.next().fuse();

        tokio::select! {
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if app.on_event(event, terminal)? == ControlFlow::Exit {
                            return Ok(());
                        };
                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => return Ok(()),
                }
            }
            Some(response) = app.next_assistant_message() => {
                // if let Message::Assistant(message) = &mut app.messages[0] {
                //     message.push_str(&response);
                // } else {
                //     app.messages.insert(0, Message::Assistant(response));
                // }

                match response {
                    ApiResponse::Text{content, idx} => {
                        app.insert_message_text_before(terminal,if idx == 0 {
                            content.trim_start()
                        } else {
                             &*content
                        })?;
                    }
                    ApiResponse::End => {
                        app.insert_message_text_before(terminal, "")?;
                    }
                }
            }
        }
    }
}
