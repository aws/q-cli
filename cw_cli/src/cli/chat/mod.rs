mod parse;

use std::io::{
    self,
    Stdout,
    Write as _,
};

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
    self,
    DisableMouseCapture,
    EnableMouseCapture,
    Event,
    KeyCode,
    KeyEvent,
    KeyEventKind,
    KeyModifiers,
    MouseEvent,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode,
    enable_raw_mode,
    EnterAlternateScreen,
    LeaveAlternateScreen,
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
    Margin,
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
    List,
    ListDirection,
    ListItem,
    Paragraph,
    Scrollbar,
    ScrollbarOrientation,
    ScrollbarState,
    Wrap,
};
use ratatui::{
    Frame,
    Terminal,
    TerminalOptions,
};
use tokio::sync::mpsc::Receiver;
use winnow::Parser;

enum Message {
    User(String),
    Assistant(String),
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    cursor_position: usize,
    /// History of recorded messages
    messages: Vec<Message>,
    rx: Option<Receiver<String>>,
    scroll_pos: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            messages: vec![Message::Assistant(
                "Hi, I'm Amazon Q. I can answer your software development questions".into(),
            )],
            cursor_position: 0,
            rx: None,
            scroll_pos: 0,
        }
    }
}

impl App {
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

    fn submit_message(&mut self) -> String {
        self.messages.insert(0, Message::User(self.input.clone()));
        let input = std::mem::replace(&mut self.input, String::new());
        self.reset_cursor();
        input
    }

    async fn next_assistant_message(&mut self) -> Option<String> {
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
}

fn setup_terminal() -> std::io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBlinking,
        SetCursorStyle::BlinkingBar
    )?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn teardown_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> std::io::Result<()> {
    let mut stdout = io::stdout();
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBlinking,
        SetCursorStyle::DefaultUserShape
    )?;
    Ok(())
}

fn send_message(client: Client, input: String) -> Receiver<String> {
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

        while let Ok(Some(a)) = res.generate_assistant_response_response.recv().await {
            match a {
                ChatResponseStream::MessageMetadataEvent(response) => {
                    // println!("{:?}", response.conversation_id());
                    // print!("{} ", "Q >".magenta().bold());
                    // std::io::stdout().flush().unwrap();
                },
                ChatResponseStream::AssistantResponseEvent(response) => {
                    // print!("{}", response.content());
                    tx.send(response.content).await.unwrap();
                },
                ChatResponseStream::FollowupPromptEvent(response) => {
                    // let followup = response.followup_prompt().unwrap();
                    // println!("content: {}", followup.content());
                    // println!("intent: {:?}", followup.user_intent());
                },
                ChatResponseStream::CodeReferenceEvent(_) => {},
                ChatResponseStream::SupplementaryWebLinksEvent(_) => {},
                _ => {},
            }
        }
    });

    rx
}

pub async fn chat() -> Result<()> {
    use crossterm::style::Stylize;

    let mut terminal = setup_terminal()?;

    let app = App::default();
    let res = run_app(&mut terminal, app).await;

    teardown_terminal(&mut terminal)?;

    res.map_err(Into::into)
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let client = cw_streaming_client().await;
    let mut stream = crossterm::event::EventStream::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let mut event = stream.next().fuse();

        tokio::select! {
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Enter => {
                                    if app.rx.is_none() {
                                        let message = app.submit_message();
                                        app.rx = Some(send_message(client.clone(), message));
                                    }
                                },
                                KeyCode::Char('c' | 'd') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    return Ok(());
                                },
                                KeyCode::Char(to_insert) => {
                                    app.enter_char(to_insert);
                                },
                                KeyCode::Backspace => {
                                    app.delete_char();
                                },
                                KeyCode::Left => {
                                    app.move_cursor_left();
                                },
                                KeyCode::Right => {
                                    app.move_cursor_right();
                                },
                                _ => {},
                            }
                        }
                    }
                    Some(Ok(Event::Paste(s))) => {
                        app.input.push_str(&s);
                    }
                    Some(Ok(Event::Mouse(MouseEvent {
                        kind: MouseEventKind::ScrollDown,
                        column,
                        ..
                    }))) => {
                        app.scroll_pos = app.scroll_pos.saturating_sub(1);
                    }
                    Some(Ok(Event::Mouse(MouseEvent {
                        kind: MouseEventKind::ScrollUp,
                        column,
                        ..
                    }))) => {
                        app.scroll_pos += 1;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => return Ok(()),
                }
            }
            Some(response) = app.next_assistant_message() => {
                if let Message::Assistant(message) = &mut app.messages[0] {
                    message.push_str(&response);
                } else {
                    app.messages.insert(0, Message::Assistant(response));
                }
            }
        }
    }
}

fn ui(f: &mut Frame<'_>, app: &mut App) {
    use ratatui::style::Stylize;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3), Constraint::Length(1)])
        .split(f.size());

    let mut text = Text::from(Line::from(
        ["Press ".into(), "ctrl+c".bold(), " to exit".into()].to_vec(),
    ));
    text.patch_style(Style::default().add_modifier(Modifier::RAPID_BLINK));

    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[2]);

    let mut input_text_style = Style::new();
    if app.input.is_empty() {
        input_text_style = input_text_style.dim();
    }

    let mut input_text = Text::styled(
        if app.input.is_empty() {
            "Message Q..."
        } else {
            app.input.as_str()
        },
        input_text_style,
    );

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(input, chunks[1]);
    // Make the cursor visible and ask ratatui to put it at the specified coordinates after
    // rendering
    f.set_cursor(
        // Draw the cursor at the current position in the input field.
        // This position is can be controlled via the left and right arrow key
        chunks[1].x + app.cursor_position as u16 + 1,
        // Move one line down, from the border to the input line
        chunks[1].y + 1,
    );

    // let messages =
    // List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    // f.render_widget(messages, chunks[2]);

    let items = app.messages.iter().rev().fold(Text::from(""), |mut acc, message| {
        match message {
            Message::User(s) => {
                acc.extend(["❯ You".bold()]);
                acc.extend(Text::from(s.trim().to_owned()));
                acc.extend([""]);
            },
            Message::Assistant(s) => {
                acc.extend(["❯ Q".bold().magenta()]);
                acc.extend(Text::from(s.trim_start().to_owned()));
                acc.extend([""]);
            },
        };
        acc
    });

    let mut paragraph = Paragraph::new(items.clone())
        .wrap(Wrap { trim: false })
        .block(Block::new().borders(Borders::RIGHT));

    let paragraph_height = paragraph.line_count(chunks[0].width);
    app.scroll_pos = paragraph_height.saturating_sub(chunks[0].height as usize);

    paragraph = paragraph.scroll((app.scroll_pos as u16, 0));

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    // let mut scrollbar_state = ScrollbarState::new(paragraph_height).position(app.scroll_pos);

    f.render_widget(paragraph, chunks[0]);
    // f.render_stateful_widget(
    //     scrollbar,
    //     chunks[0].inner(&Margin {
    //         vertical: 1,
    //         horizontal: 0,
    //     }),
    //     &mut scrollbar_state,
    // );
}
