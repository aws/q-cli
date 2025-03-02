mod conversation_state;
mod input_source;
mod parse;
mod parser;
mod prompt;
mod stdio;
mod tools;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{
    IsTerminal,
    Read,
    Write,
};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use conversation_state::ConversationState;
use crossterm::style::{
    Attribute,
    Color,
    Stylize,
};
use crossterm::{
    cursor,
    execute,
    queue,
    style,
    terminal,
};
use eyre::{
    Result,
    bail,
};
use fig_api_client::StreamingClient;
use fig_api_client::clients::SendMessageOutput;
use fig_api_client::model::{
    ChatResponseStream,
    ToolResult,
    ToolResultContentBlock,
    ToolResultStatus,
};
use fig_os_shim::Context;
use fig_util::CLI_BINARY_NAME;
use input_source::InputSource;
use parser::{
    RecvError,
    ResponseParser,
    ToolUse,
};
use serde_json::Map;
use spinners::{
    Spinner,
    Spinners,
};
use thiserror::Error;
use tokio::signal::unix::{
    SignalKind,
    signal,
};
use tools::{
    Tool,
    ToolSpec,
};
use tracing::{
    debug,
    error,
    trace,
};
use winnow::Partial;
use winnow::stream::Offset;

use crate::cli::chat::parse::{
    ParseState,
    interpret_markdown,
};
use crate::util::region_check;

pub async fn chat(initial_input: Option<String>) -> Result<ExitCode> {
    if !fig_util::system_info::in_cloudshell() && !fig_auth::is_logged_in().await {
        bail!(
            "You are not logged in, please log in with {}",
            format!("{CLI_BINARY_NAME} login",).bold()
        );
    }

    region_check("chat")?;

    let ctx = Context::new();
    let stdin = std::io::stdin();
    let is_interactive = stdin.is_terminal();
    let initial_input = if !is_interactive {
        // append to input string any extra info that was provided.
        let mut input = initial_input.unwrap_or_default();
        stdin.lock().read_to_string(&mut input)?;
        Some(input)
    } else {
        initial_input
    };

    let tool_config = load_tools()?;
    debug!(?tool_config, "Using tools");

    let client = match ctx.env().get("Q_MOCK_CHAT_RESPONSE") {
        Ok(json) => create_stream(serde_json::from_str(std::fs::read_to_string(json)?.as_str())?),
        _ => StreamingClient::new().await?,
    };

    let mut output = stdio::StdioOutput::new(is_interactive);
    let result = ChatContext::new(ChatArgs {
        output: &mut output,
        ctx,
        initial_input,
        input_source: InputSource::new()?,
        is_interactive,
        tool_config,
        client,
        terminal_width_provider: || terminal::window_size().map(|s| s.columns.into()).ok(),
    })
    .try_chat()
    .await;

    if is_interactive {
        queue!(
            output,
            cursor::MoveToColumn(0),
            style::SetAttribute(Attribute::Reset),
            style::ResetColor,
            cursor::Show
        )
        .ok();
    }
    output.flush().ok();

    result.map(|_| ExitCode::SUCCESS)
}

/// The tools that can be used by the model.
#[derive(Debug, Clone)]
pub struct ToolConfiguration {
    tools: HashMap<String, ToolSpec>,
}

/// Returns all tools supported by Q chat.
fn load_tools() -> Result<ToolConfiguration> {
    let tools: Vec<ToolSpec> = serde_json::from_str(include_str!("tools/tool_index.json"))?;
    Ok(ToolConfiguration {
        tools: tools.into_iter().map(|spec| (spec.name.clone(), spec)).collect(),
    })
}

fn print_error<W: Write>(
    output: &mut W,
    prepend_msg: &str,
    report: Option<eyre::Report>,
) -> Result<(), std::io::Error> {
    queue!(
        output,
        style::SetAttribute(Attribute::Bold),
        style::SetForegroundColor(Color::Red),
    )?;
    if let Some(report) = report {
        queue!(output, style::Print(format!("{}: {:?}\n", prepend_msg, report)),)?;
    } else {
        queue!(output, style::Print(prepend_msg), style::Print("\n"))?;
    }
    queue!(
        output,
        style::SetForegroundColor(Color::Reset),
        style::SetAttribute(Attribute::Reset),
    )?;
    output.flush()
}

/// Required fields for initializing a new chat session.
struct ChatArgs<'o, W> {
    /// The [Write] destination for printing conversation text.
    output: &'o mut W,
    ctx: Arc<Context>,
    initial_input: Option<String>,
    input_source: InputSource,
    is_interactive: bool,
    tool_config: ToolConfiguration,
    client: StreamingClient,
    terminal_width_provider: fn() -> Option<usize>,
}

/// Enum used to denote the origin of a tool use event
enum ToolUseStatus {
    /// Variant denotes that the tool use event associated with chat context is a direct result of
    /// a user request
    Idle,
    /// Variant denotes that the tool use event associated with the chat context is a result of a
    /// retry for one or more previously attempted tool use. The tuple is the utterance id
    /// associated with the original user request that necessitated the tool use
    RetryInProgress(String),
}

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("{0}")]
    Client(#[from] fig_api_client::Error),
    #[error("{0}")]
    ResponseStream(#[from] parser::RecvError),
    #[error("{0}")]
    Std(#[from] std::io::Error),
    #[error("{0}")]
    Readline(#[from] rustyline::error::ReadlineError),
    #[error("{0}")]
    Custom(Cow<'static, str>),
    #[error("interrupted")]
    Interrupted { tool_uses: Option<Vec<QueuedTool>> },
}

pub struct ChatContext<'o, W> {
    /// The [Write] destination for printing conversation text.
    output: &'o mut W,
    ctx: Arc<Context>,
    initial_input: Option<String>,
    input_source: InputSource,
    is_interactive: bool,
    /// The client to use to interact with the model.
    client: StreamingClient,
    /// Width of the terminal, required for [ParseState].
    terminal_width_provider: fn() -> Option<usize>,
    spinner: Option<Spinner>,
    /// [ConversationState].
    conversation_state: ConversationState,
    /// Telemetry events to be sent as part of the conversation.
    tool_use_telemetry_events: HashMap<String, ToolUseEventBuilder>,
    /// State used to keep track of tool use relation
    tool_use_status: ToolUseStatus,
}

impl<W> std::fmt::Debug for ChatContext<'_, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatContext")
            .field("initial_input", &self.initial_input)
            .field("input_source", &self.input_source)
            .field("is_interactive", &self.is_interactive)
            .field("client", &self.client)
            .field("terminal_width_provider", &self.terminal_width_provider)
            .field("conversation_state", &self.conversation_state)
            .field("tool_use_telemetry_events", &self.tool_use_telemetry_events)
            .finish()
    }
}

/// An executable `(tool_use_id, Tool)` tuple.
type QueuedTool = (String, Tool);

/// The chat execution state.
///
/// Intended to provide more robust handling around state transitions while dealing with, e.g.,
/// tool validation, execution, response stream handling, etc.
#[derive(Debug)]
enum ChatState {
    /// Prompt the user with `tool_uses`, if available.
    PromptUser { tool_uses: Option<Vec<QueuedTool>> },
    /// Handle the user input, depending on if any tools require execution.
    HandleInput {
        input: String,
        tool_uses: Option<Vec<QueuedTool>>,
    },
    /// Validate the list of tool uses provided by the model.
    ValidateTools(Vec<ToolUse>),
    /// Execute the list of tools.
    ExecuteTools(Vec<QueuedTool>),
    /// Consume the response stream and display to the user.
    HandleResponseStream(SendMessageOutput),
    /// Exit the chat.
    Exit,
}

impl Default for ChatState {
    fn default() -> Self {
        Self::PromptUser { tool_uses: None }
    }
}

impl<'o, W> ChatContext<'o, W>
where
    W: Write,
{
    fn new(args: ChatArgs<'o, W>) -> Self {
        Self {
            output: args.output,
            ctx: args.ctx,
            initial_input: args.initial_input,
            input_source: args.input_source,
            is_interactive: args.is_interactive,
            client: args.client,
            terminal_width_provider: args.terminal_width_provider,
            spinner: None,
            conversation_state: ConversationState::new(args.tool_config),
            tool_use_telemetry_events: HashMap::new(),
            tool_use_status: ToolUseStatus::Idle,
        }
    }

    async fn try_chat(&mut self) -> Result<()> {
        if self.is_interactive {
            execute!(
                self.output,
                style::Print(color_print::cstr! {"
Hi, I'm <g>Amazon Q</g>. Ask me anything.

<em>@history</em> to pass your shell history
<em>@git</em> to pass information about your current git repository
<em>@env</em> to pass your shell environment

"
                })
            )?;
        }

        let mut ctrl_c_stream = signal(SignalKind::interrupt())?;

        let mut next_state = Some(ChatState::PromptUser { tool_uses: None });

        if let Some(user_input) = self.initial_input.take() {
            execute!(
                self.output,
                style::SetForegroundColor(Color::Magenta),
                style::Print("> "),
                style::SetAttribute(Attribute::Reset),
                style::Print(&user_input),
                style::Print("\n")
            )?;
            next_state = Some(ChatState::HandleInput {
                input: user_input,
                tool_uses: None,
            });
        }

        loop {
            debug_assert!(next_state.is_some());
            let chat_state = next_state.take().unwrap_or_default();
            debug!(?chat_state, "changing to state");

            let result = match chat_state {
                ChatState::PromptUser { tool_uses } => self.prompt_user(tool_uses).await,
                ChatState::HandleInput { input, tool_uses } => self.handle_input(input, tool_uses).await,
                ChatState::ExecuteTools(tool_uses) => {
                    let tool_uses_clone = tool_uses.clone();
                    tokio::select! {
                        res = self.tool_use_execute(tool_uses) => res,
                        Some(_) = ctrl_c_stream.recv() => Err(ChatError::Interrupted { tool_uses: Some(tool_uses_clone) })
                    }
                },
                ChatState::ValidateTools(tool_uses) => {
                    tokio::select! {
                        res = self.validate_tools(tool_uses) => res,
                        Some(_) = ctrl_c_stream.recv() => Err(ChatError::Interrupted { tool_uses: None })
                    }
                },
                ChatState::HandleResponseStream(response) => tokio::select! {
                    res = self.handle_response(response) => res,
                    Some(_) = ctrl_c_stream.recv() => Err(ChatError::Interrupted { tool_uses: None })
                },
                ChatState::Exit => return Ok(()),
            };

            match result {
                Ok(state) => next_state = Some(state),
                Err(e) => {
                    error!(?e, "An error occurred processing the current state");
                    if self.is_interactive && self.spinner.is_some() {
                        drop(self.spinner.take());
                        queue!(
                            self.output,
                            terminal::Clear(terminal::ClearType::CurrentLine),
                            cursor::MoveToColumn(0),
                        )?;
                    }
                    let mut tool_uses = None;
                    match e {
                        ChatError::Interrupted { tool_uses: inter } => {
                            execute!(self.output, style::Print("\n"))?;
                            tool_uses = inter;
                        },
                        ChatError::Client(err) => {
                            if let fig_api_client::Error::QuotaBreach(msg) = err {
                                print_error(self.output, msg, None)?;
                            } else {
                                print_error(
                                    self.output,
                                    "Amazon Q is having trouble responding right now",
                                    Some(err.into()),
                                )?;
                            }
                        },
                        _ => {
                            print_error(
                                self.output,
                                "Amazon Q is having trouble responding right now",
                                Some(e.into()),
                            )?;
                        },
                    }
                    self.conversation_state.fix_history();
                    next_state = Some(ChatState::PromptUser { tool_uses });
                },
            }
        }
    }

    /// Read input from the user.
    async fn prompt_user(&mut self, mut tool_uses: Option<Vec<QueuedTool>>) -> Result<ChatState, ChatError> {
        if self.is_interactive {
            execute!(self.output, cursor::Show)?;
        }
        let tool_uses = tool_uses.take().unwrap_or_default();
        if !tool_uses.is_empty() {
            let terminal_width = self.terminal_width();
            for (i, (_, tool)) in tool_uses.iter().enumerate() {
                queue!(
                    self.output,
                    style::SetForegroundColor(Color::Cyan),
                    style::Print(format!("{}. {}\n", i + 1, tool.display_name())),
                    style::SetForegroundColor(Color::Reset),
                    style::SetForegroundColor(Color::DarkGrey),
                    style::Print(format!("{}\n", "▔".repeat(terminal_width))),
                    style::SetForegroundColor(Color::Reset),
                )?;
                tool.queue_description(&self.ctx, self.output)
                    .map_err(|e| ChatError::Custom(format!("failed to print tool: {}", e).into()))?;
                queue!(self.output, style::Print("\n"))?;
            }

            execute!(
                self.output,
                style::SetForegroundColor(Color::DarkGrey),
                style::Print("▁".repeat(terminal_width)),
                style::ResetColor,
                style::Print("\n\nEnter "),
                style::SetForegroundColor(Color::Green),
                style::Print("y"),
                style::ResetColor,
                style::Print(format!(
                    " to run {}, or otherwise continue your conversation.\n\n",
                    match tool_uses.len() == 1 {
                        true => "this tool",
                        false => "these tools",
                    }
                )),
            )?;
        }
        let user_input = match self.input_source.read_line(Some("> "))? {
            Some(line) => line,
            None => return Ok(ChatState::Exit),
        };
        Ok(ChatState::HandleInput {
            input: user_input,
            tool_uses: Some(tool_uses),
        })
    }

    async fn handle_input(
        &mut self,
        user_input: String,
        tool_uses: Option<Vec<QueuedTool>>,
    ) -> Result<ChatState, ChatError> {
        let tool_uses = tool_uses.unwrap_or_default();
        match user_input.as_str() {
            "exit" | "quit" => Ok(ChatState::Exit),
            "/clear" => {
                self.conversation_state.clear();
                execute!(
                    self.output,
                    style::SetForegroundColor(Color::Green),
                    style::Print("\nConversation history cleared\n\n"),
                    style::SetForegroundColor(Color::Reset)
                )?;
                Ok(ChatState::PromptUser { tool_uses: None })
            },
            "y" if !tool_uses.is_empty() => Ok(ChatState::ExecuteTools(tool_uses)),
            _ => {
                self.tool_use_status = ToolUseStatus::Idle;
                if self.is_interactive {
                    queue!(self.output, style::SetForegroundColor(Color::Magenta))?;
                    if user_input.contains("@history") {
                        queue!(self.output, style::Print("Using shell history\n"))?;
                    }
                    if user_input.contains("@git") {
                        queue!(self.output, style::Print("Using git context\n"))?;
                    }
                    if user_input.contains("@env") {
                        queue!(self.output, style::Print("Using environment\n"))?;
                    }
                    queue!(self.output, style::SetForegroundColor(Color::Reset))?;
                    queue!(self.output, cursor::Hide)?;
                    execute!(self.output, style::Print("\n"))?;
                    self.spinner = Some(Spinner::new(Spinners::Dots, "Thinking...".to_owned()));
                }

                if !tool_uses.is_empty() {
                    self.conversation_state.abandon_tool_use(tool_uses, user_input);
                } else {
                    self.conversation_state.append_new_user_message(user_input).await;
                }

                self.send_tool_use_telemetry().await;
                return Ok(ChatState::HandleResponseStream(
                    self.client
                        .send_message(self.conversation_state.as_sendable_conversation_state())
                        .await?,
                ));
            },
        }
    }

    async fn tool_use_execute(&mut self, tool_uses: Vec<QueuedTool>) -> Result<ChatState, ChatError> {
        // Execute the requested tools.
        let terminal_width = self.terminal_width();
        let mut tool_results = vec![];
        for tool in tool_uses {
            let mut tool_telemetry = self.tool_use_telemetry_events.entry(tool.0.clone());
            tool_telemetry = tool_telemetry.and_modify(|ev| ev.is_accepted = true);

            let tool_start = std::time::Instant::now();
            queue!(
                self.output,
                style::Print("\n\nExecuting "),
                style::SetForegroundColor(Color::Cyan),
                style::Print(format!("{}...\n", tool.1.display_name())),
                style::SetForegroundColor(Color::DarkGrey),
                style::Print(format!("{}\n", "▔".repeat(terminal_width))),
                style::SetForegroundColor(Color::Reset),
            )?;
            let invoke_result = tool.1.invoke(&self.ctx, self.output).await;

            if self.is_interactive && self.spinner.is_some() {
                queue!(
                    self.output,
                    terminal::Clear(terminal::ClearType::CurrentLine),
                    cursor::MoveToColumn(0),
                    cursor::Show
                )?;
            }
            execute!(self.output, style::Print("\n"))?;

            let tool_time = std::time::Instant::now().duration_since(tool_start);
            let tool_time = format!("{}.{}", tool_time.as_secs(), tool_time.subsec_millis());

            match invoke_result {
                Ok(result) => {
                    debug!("tool result output: {:#?}", result);
                    execute!(
                        self.output,
                        style::SetForegroundColor(Color::Green),
                        style::Print(format!("🟢 Completed in {}s", tool_time)),
                        style::SetForegroundColor(Color::Reset),
                        style::Print("\n"),
                    )?;

                    tool_telemetry.and_modify(|ev| ev.is_success = Some(true));
                    tool_results.push(ToolResult {
                        tool_use_id: tool.0,
                        content: vec![result.into()],
                        status: ToolResultStatus::Success,
                    });
                },
                Err(err) => {
                    error!(?err, "An error occurred processing the tool");
                    execute!(
                        self.output,
                        style::SetAttribute(Attribute::Bold),
                        style::SetForegroundColor(Color::Red),
                        style::Print(format!("🔴 Execution failed after {}s:\n", tool_time)),
                        style::SetAttribute(Attribute::Reset),
                        style::SetForegroundColor(Color::Red),
                        style::Print(&err),
                        style::SetAttribute(Attribute::Reset),
                        style::Print("\n\n"),
                    )?;

                    tool_telemetry.and_modify(|ev| ev.is_success = Some(false));
                    tool_results.push(ToolResult {
                        tool_use_id: tool.0,
                        content: vec![ToolResultContentBlock::Text(format!(
                            "An error occurred processing the tool: \n{}",
                            &err
                        ))],
                        status: ToolResultStatus::Error,
                    });
                    if let ToolUseStatus::Idle = self.tool_use_status {
                        self.tool_use_status = ToolUseStatus::RetryInProgress(
                            self.conversation_state
                                .message_id()
                                .map_or("No utterance id found".to_string(), |v| v.to_string()),
                        );
                    }
                },
            }
        }

        self.conversation_state.add_tool_results(tool_results);
        self.send_tool_use_telemetry().await;
        return Ok(ChatState::HandleResponseStream(
            self.client
                .send_message(self.conversation_state.as_sendable_conversation_state())
                .await?,
        ));
    }

    async fn handle_response(&mut self, response: SendMessageOutput) -> Result<ChatState, ChatError> {
        let mut buf = String::new();
        let mut offset = 0;
        let mut ended = false;
        let mut parser = ResponseParser::new(response);
        let mut state = ParseState::new(Some(self.terminal_width()));

        let mut tool_uses = Vec::new();
        let mut tool_name_being_recvd: Option<String> = None;
        loop {
            match parser.recv().await {
                Ok(msg_event) => {
                    trace!("Consumed: {:?}", msg_event);
                    match msg_event {
                        parser::ResponseEvent::ToolUseStart { name } => {
                            // We need to flush the buffer here, otherwise text will not be
                            // printed while we are receiving tool use events.
                            buf.push('\n');
                            tool_name_being_recvd = Some(name);
                        },
                        parser::ResponseEvent::AssistantText(text) => {
                            buf.push_str(&text);
                        },
                        parser::ResponseEvent::ToolUse(tool_use) => {
                            if self.is_interactive && self.spinner.is_some() {
                                drop(self.spinner.take());
                                queue!(
                                    self.output,
                                    terminal::Clear(terminal::ClearType::CurrentLine),
                                    cursor::MoveToColumn(0),
                                    cursor::Show
                                )?;
                            }
                            tool_uses.push(tool_use);
                            tool_name_being_recvd = None;
                        },
                        parser::ResponseEvent::EndStream { message } => {
                            self.conversation_state.push_assistant_message(message);
                            ended = true;
                        },
                    }
                },
                Err(RecvError::UnexpectedToolUseEos {
                    tool_use_id,
                    name,
                    message,
                }) => {
                    error!(
                        tool_use_id,
                        name, "The response stream ended before the entire tool use was received"
                    );
                    if self.is_interactive {
                        execute!(self.output, cursor::Hide)?;
                        self.spinner = Some(Spinner::new(
                            Spinners::Dots,
                            "The generated tool use was too large, trying to divide up the work...".to_string(),
                        ));
                    }

                    self.conversation_state.push_assistant_message(*message);
                    let tool_results = vec![ToolResult {
                            tool_use_id,
                            content: vec![ToolResultContentBlock::Text(
                                "The generated tool was too large, try again but this time split up the work between multiple tool uses".to_string(),
                            )],
                            status: ToolResultStatus::Error,
                        }];
                    self.conversation_state.add_tool_results(tool_results);
                    self.send_tool_use_telemetry().await;
                    return Ok(ChatState::HandleResponseStream(
                        self.client
                            .send_message(self.conversation_state.as_sendable_conversation_state())
                            .await?,
                    ));
                },
                Err(err) => return Err(err.into()),
            }

            // Fix for the markdown parser copied over from q chat:
            // this is a hack since otherwise the parser might report Incomplete with useful data
            // still left in the buffer. I'm not sure how this is intended to be handled.
            if ended {
                buf.push('\n');
            }

            if tool_name_being_recvd.is_none() && !buf.is_empty() && self.is_interactive && self.spinner.is_some() {
                drop(self.spinner.take());
                queue!(
                    self.output,
                    terminal::Clear(terminal::ClearType::CurrentLine),
                    cursor::MoveToColumn(0),
                    cursor::Show
                )?;
            }

            // Print the response
            loop {
                let input = Partial::new(&buf[offset..]);
                match interpret_markdown(input, &mut self.output, &mut state) {
                    Ok(parsed) => {
                        offset += parsed.offset_from(&input);
                        self.output.flush()?;
                        state.newline = state.set_newline;
                        state.set_newline = false;
                    },
                    Err(err) => match err.into_inner() {
                        Some(err) => return Err(ChatError::Custom(err.to_string().into())),
                        None => break, // Data was incomplete
                    },
                }

                // TODO: We should buffer output based on how much we have to parse, not as a constant
                // Do not remove unless you are nabochay :)
                std::thread::sleep(Duration::from_millis(8));
            }

            // Set spinner after showing all of the assistant text content so far.
            if let (Some(name), true) = (&tool_name_being_recvd, self.is_interactive) {
                queue!(
                    self.output,
                    style::SetForegroundColor(Color::Blue),
                    style::Print(format!("\n{name}: ")),
                    style::SetForegroundColor(Color::Reset),
                    cursor::Hide,
                )?;
                self.spinner = Some(Spinner::new(Spinners::Dots, "Thinking...".to_string()));
            }

            if ended {
                if let Some(message_id) = self.conversation_state.message_id() {
                    fig_telemetry::send_chat_added_message(
                        self.conversation_state.conversation_id().to_owned(),
                        message_id.to_owned(),
                    )
                    .await;
                }
                if self.is_interactive {
                    queue!(self.output, style::ResetColor, style::SetAttribute(Attribute::Reset))?;
                    execute!(self.output, style::Print("\n"))?;

                    for (i, citation) in &state.citations {
                        queue!(
                            self.output,
                            style::Print("\n"),
                            style::SetForegroundColor(Color::Blue),
                            style::Print(format!("[^{i}]: ")),
                            style::SetForegroundColor(Color::DarkGrey),
                            style::Print(format!("{citation}\n")),
                            style::SetForegroundColor(Color::Reset)
                        )?;
                    }
                }

                break;
            }
        }

        if !tool_uses.is_empty() {
            Ok(ChatState::ValidateTools(tool_uses))
        } else {
            Ok(ChatState::PromptUser { tool_uses: None })
        }
    }

    async fn validate_tools(&mut self, tool_uses: Vec<ToolUse>) -> Result<ChatState, ChatError> {
        let conv_id = self.conversation_state.conversation_id().to_owned();
        debug!(?tool_uses, "Validating tool uses");
        let mut queued_tools: Vec<QueuedTool> = Vec::new();
        let mut tool_results = Vec::new();
        for tool_use in tool_uses {
            let tool_use_id = tool_use.id.clone();
            let mut tool_telemetry = ToolUseEventBuilder::new(conv_id.clone(), tool_use.id.clone())
                .set_tool_use_id(tool_use_id.clone())
                .set_tool_name(tool_use.name.clone())
                .utterance_id(self.conversation_state.message_id().map(|s| s.to_string()));
            match Tool::try_from(tool_use) {
                Ok(mut tool) => {
                    match tool.validate(&self.ctx).await {
                        Ok(()) => {
                            tool_telemetry.is_valid = Some(true);
                            queued_tools.push((tool_use_id.clone(), tool));
                        },
                        Err(err) => {
                            tool_telemetry.is_valid = Some(false);
                            tool_results.push(ToolResult {
                                tool_use_id: tool_use_id.clone(),
                                content: vec![ToolResultContentBlock::Text(format!(
                                    "Failed to validate tool parameters: {err}"
                                ))],
                                status: ToolResultStatus::Error,
                            });
                        },
                    };
                },
                Err(err) => {
                    tool_telemetry.is_valid = Some(false);
                    tool_results.push(err);
                },
            }
            self.tool_use_telemetry_events.insert(tool_use_id, tool_telemetry);
        }

        // If we have any validation errors, then return them immediately to the model.
        if !tool_results.is_empty() {
            debug!(?tool_results, "Error found in the model tools");
            queue!(
                self.output,
                style::SetAttribute(Attribute::Bold),
                style::Print("Tool validation failed: "),
                style::SetAttribute(Attribute::Reset),
            )?;
            for tool_result in &tool_results {
                for block in &tool_result.content {
                    let content = match block {
                        ToolResultContentBlock::Text(t) => Some(t.as_str()),
                        ToolResultContentBlock::Json(d) => d.as_string(),
                    };
                    if let Some(content) = content {
                        queue!(
                            self.output,
                            style::Print("\n"),
                            style::SetForegroundColor(Color::Red),
                            style::Print(format!("{}\n", content)),
                            style::SetForegroundColor(Color::Reset),
                        )?;
                    }
                }
            }
            self.conversation_state.add_tool_results(tool_results);
            self.send_tool_use_telemetry().await;
            if let ToolUseStatus::Idle = self.tool_use_status {
                self.tool_use_status = ToolUseStatus::RetryInProgress(
                    self.conversation_state
                        .message_id()
                        .map_or("No utterance id found".to_string(), |v| v.to_string()),
                );
            }

            let response = self
                .client
                .send_message(self.conversation_state.as_sendable_conversation_state())
                .await?;
            return Ok(ChatState::HandleResponseStream(response));
        }

        let skip_consent = self
            .ctx
            .env()
            .get("Q_CHAT_SKIP_TOOL_CONSENT")
            .is_ok_and(|s| !s.is_empty() && !queued_tools.is_empty())
            || queued_tools.iter().all(|tool| !tool.1.requires_consent(&self.ctx));

        if skip_consent {
            Ok(ChatState::ExecuteTools(queued_tools))
        } else {
            Ok(ChatState::PromptUser {
                tool_uses: Some(queued_tools),
            })
        }
    }

    async fn send_tool_use_telemetry(&mut self) {
        for (_, mut event) in self.tool_use_telemetry_events.drain() {
            event.user_input_id = match self.tool_use_status {
                ToolUseStatus::Idle => self.conversation_state.message_id(),
                ToolUseStatus::RetryInProgress(ref id) => Some(id.as_str()),
            }
            .map(|v| v.to_string());
            let event: fig_telemetry::EventType = event.into();
            let app_event = fig_telemetry::AppTelemetryEvent::new(event).await;
            fig_telemetry::dispatch_or_send_event(app_event).await;
        }
    }

    fn terminal_width(&self) -> usize {
        (self.terminal_width_provider)().unwrap_or(80)
    }
}

#[derive(Debug)]
struct ToolUseEventBuilder {
    pub conversation_id: String,
    pub utterance_id: Option<String>,
    pub user_input_id: Option<String>,
    pub tool_use_id: Option<String>,
    pub tool_name: Option<String>,
    pub is_accepted: bool,
    pub is_success: Option<bool>,
    pub is_valid: Option<bool>,
}

impl ToolUseEventBuilder {
    pub fn new(conv_id: String, tool_use_id: String) -> Self {
        Self {
            conversation_id: conv_id,
            utterance_id: None,
            user_input_id: None,
            tool_use_id: Some(tool_use_id),
            tool_name: None,
            is_accepted: false,
            is_success: None,
            is_valid: None,
        }
    }

    pub fn utterance_id(mut self, id: Option<String>) -> Self {
        self.utterance_id = id;
        self
    }

    pub fn set_tool_use_id(mut self, id: String) -> Self {
        self.tool_use_id.replace(id);
        self
    }

    pub fn set_tool_name(mut self, name: String) -> Self {
        self.tool_name.replace(name);
        self
    }
}

impl From<ToolUseEventBuilder> for fig_telemetry::EventType {
    fn from(val: ToolUseEventBuilder) -> Self {
        fig_telemetry::EventType::ToolUseSuggested {
            conversation_id: val.conversation_id,
            utterance_id: val.utterance_id,
            user_input_id: val.user_input_id,
            tool_use_id: val.tool_use_id,
            tool_name: val.tool_name,
            is_accepted: val.is_accepted,
            is_success: val.is_success,
            is_valid: val.is_valid,
        }
    }
}

/// Testing helper
fn split_tool_use_event(value: &Map<String, serde_json::Value>) -> Vec<ChatResponseStream> {
    let tool_use_id = value.get("tool_use_id").unwrap().as_str().unwrap().to_string();
    let name = value.get("name").unwrap().as_str().unwrap().to_string();
    let args_str = value.get("args").unwrap().to_string();
    let split_point = args_str.len() / 2;
    vec![
        ChatResponseStream::ToolUseEvent {
            tool_use_id: tool_use_id.clone(),
            name: name.clone(),
            input: None,
            stop: None,
        },
        ChatResponseStream::ToolUseEvent {
            tool_use_id: tool_use_id.clone(),
            name: name.clone(),
            input: Some(args_str.split_at(split_point).0.to_string()),
            stop: None,
        },
        ChatResponseStream::ToolUseEvent {
            tool_use_id: tool_use_id.clone(),
            name: name.clone(),
            input: Some(args_str.split_at(split_point).1.to_string()),
            stop: None,
        },
        ChatResponseStream::ToolUseEvent {
            tool_use_id: tool_use_id.clone(),
            name: name.clone(),
            input: None,
            stop: Some(true),
        },
    ]
}

/// Testing helper
fn create_stream(model_responses: serde_json::Value) -> StreamingClient {
    let mut mock = Vec::new();
    for response in model_responses.as_array().unwrap() {
        let mut stream = Vec::new();
        for event in response.as_array().unwrap() {
            match event {
                serde_json::Value::String(assistant_text) => {
                    stream.push(ChatResponseStream::AssistantResponseEvent {
                        content: assistant_text.to_string(),
                    });
                },
                serde_json::Value::Object(tool_use) => {
                    stream.append(&mut split_tool_use_event(tool_use));
                },
                other => panic!("Unexpected value: {:?}", other),
            }
        }
        mock.push(stream);
    }
    StreamingClient::mock(mock)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_flow() {
        let ctx = Context::builder().with_test_home().await.unwrap().build_fake();
        let mut output = std::io::stdout();
        let test_client = create_stream(serde_json::json!([
            [
                "Sure, I'll create a file for you",
                {
                    "tool_use_id": "1",
                    "name": "fs_write",
                    "args": {
                        "command": "create",
                        "file_text": "Hello, world!",
                        "path": "/file.txt",
                    }
                }
            ],
            [
                "Hope that looks good to you!",
            ],
        ]));

        let c = ChatArgs {
            output: &mut output,
            ctx: Arc::clone(&ctx),
            initial_input: None,
            input_source: InputSource::new_mock(vec![
                "create a new file".to_string(),
                "y".to_string(),
                "exit".to_string(),
            ]),
            is_interactive: true,
            tool_config: load_tools().unwrap(),
            client: test_client,
            terminal_width_provider: || Some(80),
        };

        ChatContext::new(c).try_chat().await.unwrap();

        assert_eq!(ctx.fs().read_to_string("/file.txt").await.unwrap(), "Hello, world!");
    }
}
