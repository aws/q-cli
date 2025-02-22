mod conversation_state;
mod input_source;
mod parse;
mod parser;
mod prompt;
mod stdio;
mod tools;
use std::collections::HashMap;
use std::io::{
    IsTerminal,
    Read,
    Write,
};
use std::process::ExitCode;
use std::sync::Arc;

use color_eyre::owo_colors::OwoColorize;
use conversation_state::ConversationState;
use crossterm::style::{
    Attribute,
    Color,
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
use futures::StreamExt;
use input_source::InputSource;
use parser::{
    ResponseParser,
    ToolUse,
};
use serde_json::Map;
use spinners::{
    Spinner,
    Spinners,
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

const MAX_TOOL_USE_RECURSIONS: u32 = 50;

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
        queue!(output, style::SetAttribute(Attribute::Reset), style::ResetColor).ok();
    }
    output.flush().ok();

    result.map(|_| ExitCode::SUCCESS)
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

/// State required for a chat session.
struct ChatContext<'o, W> {
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
    /// Tool uses requested by the model.
    tool_uses: Vec<ToolUse>,
    /// [ConversationState].
    conversation_state: ConversationState,
    /// The number of times a tool use has been attempted without user intervention.
    tool_use_recursions: u32,
    current_user_input_id: Option<String>,
    tool_use_events: Vec<ToolUseEventBuilder>,
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
            tool_uses: vec![],
            conversation_state: ConversationState::new(args.tool_config),
            tool_use_recursions: 0,
            current_user_input_id: None,
            tool_use_events: vec![],
        }
    }

    async fn try_chat(&mut self) -> Result<()> {
        // todo: what should we set this to?
        if self.is_interactive {
            execute!(
                self.output,
                style::Print(color_print::cstr! {"
Hi, I'm <g>Amazon Q</g>. I can answer questions about your workspace and tooling, and help execute ops related tasks on your behalf!

<em>@history</em> to pass your shell history
<em>@git</em> to pass information about your current git repository
<em>@env</em> to pass your shell environment

"
                })
            )?;
        }

        loop {
            let mut response = self.prompt_and_send_request().await?;
            let response = match response.take() {
                Some(response) => response,
                None => {
                    if let Some(id) = &self.conversation_state.conversation_id {
                        fig_telemetry::send_end_chat(id.clone()).await;
                    }
                    self.send_tool_use_telemetry().await;
                    break;
                },
            };

            // Handle the response
            let mut buf = String::new();
            let mut offset = 0;
            let mut ended = false;
            let mut parser = ResponseParser::new(response);
            let mut state = ParseState::new(Some(self.terminal_width()));
            if let Some(ref msg) = self.conversation_state.next_message {
                if let fig_api_client::model::ChatMessage::AssistantResponseMessage(ref resp) = msg.0 {
                    self.current_user_input_id = resp.message_id.clone();
                }
            }

            loop {
                match parser.recv().await {
                    Ok(msg_event) => {
                        trace!("Consumed: {:?}", msg_event);
                        match msg_event {
                            parser::ResponseEvent::ConversationId(id) => {
                                fig_telemetry::send_start_chat(id.clone()).await;
                                self.conversation_state.conversation_id = Some(id.clone());
                                tokio::task::spawn(async move {
                                    tokio::signal::ctrl_c().await.unwrap();
                                    fig_telemetry::send_end_chat(id.clone()).await;
                                    fig_telemetry::finish_telemetry().await;
                                    #[allow(clippy::exit)]
                                    std::process::exit(0);
                                });
                            },
                            parser::ResponseEvent::AssistantText(text) => {
                                buf.push_str(&text);
                            },
                            parser::ResponseEvent::ToolUse(tool_use) => {
                                self.tool_uses.push(tool_use);
                            },
                            parser::ResponseEvent::EndStream { message } => {
                                self.conversation_state.push_assistant_message(message);
                                ended = true;
                            },
                        };
                    },
                    Err(err) => {
                        bail!("An error occurred reading the model's response: {:?}", err);
                    },
                }

                // Fix for the markdown parser copied over from q chat:
                // this is a hack since otherwise the parser might report Incomplete with useful data
                // still left in the buffer. I'm not sure how this is intended to be handled.
                if ended {
                    buf.push('\n');
                }

                if !buf.is_empty() && self.is_interactive && self.spinner.is_some() {
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
                            Some(err) => bail!(err.to_string()),
                            None => break, // Data was incomplete
                        },
                    }
                }

                if ended {
                    if let (Some(conversation_id), Some(message_id)) = (
                        self.conversation_state.conversation_id(),
                        self.conversation_state.message_id(),
                    ) {
                        fig_telemetry::send_chat_added_message(conversation_id.to_owned(), message_id.to_owned()).await;
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

            if !self.is_interactive {
                break;
            }
        }

        Ok(())
    }

    async fn prompt_and_send_request(&mut self) -> Result<Option<SendMessageOutput>> {
        // Tool uses that need to be executed.
        let mut queued_tools: Vec<(String, Tool)> = Vec::new();

        // Validate the requested tools, updating queued_tools and tool_errors accordingly.
        if !self.tool_uses.is_empty() {
            let conv_id = self
                .conversation_state
                .conversation_id
                .as_ref()
                .unwrap_or(&"No conversation id associated".to_string())
                .to_owned();
            let utterance_id = self
                .conversation_state
                .next_message
                .as_ref()
                .and_then(|msg| {
                    if let fig_api_client::model::ChatMessage::AssistantResponseMessage(resp) = &msg.0 {
                        resp.message_id.clone()
                    } else {
                        None
                    }
                })
                .unwrap_or("No utterance id associated".to_string());
            debug!(?self.tool_uses, "Validating tool uses");
            let mut tool_results = Vec::with_capacity(self.tool_uses.len());
            for tool_use in self.tool_uses.drain(..) {
                let tool_use_id = tool_use.id.clone();
                let mut event_builder = ToolUseEventBuilder::from_conversation_id(conv_id.clone())
                    .set_tool_use_id(&tool_use_id)
                    .set_tool_name(&tool_use.name)
                    .set_utterance_id(&utterance_id);
                if let Some(ref id) = self.current_user_input_id {
                    event_builder.user_input_id = Some(id.clone());
                }
                match Tool::try_from(tool_use) {
                    Ok(mut tool) => {
                        match tool.validate(&self.ctx).await {
                            Ok(()) => {
                                queued_tools.push((tool_use_id, tool));
                            },
                            Err(err) => {
                                tool_results.push(ToolResult {
                                    tool_use_id,
                                    content: vec![ToolResultContentBlock::Text(format!(
                                        "Failed to validate tool parameters: {err}"
                                    ))],
                                    status: ToolResultStatus::Error,
                                });
                                event_builder.is_valid = Some(false);
                            },
                        };
                    },
                    Err(err) => {
                        tool_results.push(err);
                        event_builder.is_valid = Some(false);
                    },
                }
                self.tool_use_events.push(event_builder);
            }

            // If we have any validation errors, then return them immediately to the model.
            if !tool_results.is_empty() {
                debug!(?tool_results, "Error found in the model tools");
                self.conversation_state.add_tool_results(tool_results);
                self.send_tool_use_telemetry().await;
                return Ok(Some(
                    self.client
                        .send_message(self.conversation_state.as_sendable_conversation_state())
                        .await?,
                ));
            }
        }

        // If we have tool uses, display them to the user.
        if !queued_tools.is_empty() {
            self.tool_use_recursions += 1;
            let terminal_width = self.terminal_width();
            if self.tool_use_recursions > MAX_TOOL_USE_RECURSIONS {
                bail!("Exceeded max tool use recursion limit: {}", MAX_TOOL_USE_RECURSIONS);
            }
            for (_, tool) in &queued_tools {
                queue!(self.output, style::Print(format!("{}\n", "─".repeat(terminal_width))))?;
                queue!(self.output, cursor::MoveToPreviousLine(1))?;
                queue!(self.output, cursor::MoveToColumn(2))?;
                queue!(self.output, style::Print(format!(" {} ", tool.display_name())))?;
                queue!(self.output, cursor::MoveToNextLine(1))?;
                queue!(self.output, style::SetAttribute(Attribute::NormalIntensity))?;
                tool.queue_description(self.output)?;
                queue!(self.output, style::Print("\n"))?;
                queue!(self.output, cursor::MoveToNextLine(1))?;
            }
            queue!(self.output, style::Print("─".repeat(terminal_width)))?;
            queue!(self.output, style::Print("\n"))?;
            execute!(
                self.output,
                style::Print("Enter "),
                style::SetForegroundColor(Color::Green),
                style::Print("y"),
                style::ResetColor,
                style::Print(" to consent to running these tools, or anything else to continue your conversation.\n\n")
            )?;
        }

        let user_input = match self.initial_input.take() {
            Some(input) => input,
            None => match self.input_source.read_line(Some("> "))? {
                Some(line) => line,
                None => return Ok(None),
            },
        };

        match user_input.trim() {
            "exit" | "quit" => Ok(None),
            // Tool execution.
            c if c.to_lowercase() == "y" && !queued_tools.is_empty() => {
                // Execute the requested tools.
                let mut tool_results = vec![];
                for tool in queued_tools.drain(..) {
                    let corresponding_builder = self.tool_use_events.iter_mut().find(|v| {
                        if let Some(ref v) = v.tool_use_id {
                            v.eq(&tool.0)
                        } else {
                            false
                        }
                    });
                    match tool.1.invoke(&self.ctx, self.output).await {
                        Ok(result) => {
                            debug!("tool result output: {:#?}", result);
                            tool_results.push(ToolResult {
                                tool_use_id: tool.0,
                                content: vec![result.into()],
                                status: ToolResultStatus::Success,
                            });
                            if let Some(builder) = corresponding_builder {
                                builder.is_success = Some(true);
                            }
                        },
                        Err(err) => {
                            error!(?err, "An error occurred processing the tool");
                            tool_results.push(ToolResult {
                                tool_use_id: tool.0,
                                content: vec![ToolResultContentBlock::Text(format!(
                                    "An error occurred processing the tool: {}",
                                    err
                                ))],
                                status: ToolResultStatus::Error,
                            });
                            if let Some(builder) = corresponding_builder {
                                builder.is_success = Some(false);
                            }
                        },
                    }
                }

                self.conversation_state.add_tool_results(tool_results);
                self.send_tool_use_telemetry().await;
                Ok(Some(
                    self.client
                        .send_message(self.conversation_state.as_sendable_conversation_state())
                        .await?,
                ))
            },
            // New user prompt.
            _ => {
                self.tool_use_recursions = 0;

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
                    tokio::spawn(async {
                        tokio::signal::ctrl_c().await.unwrap();
                        execute!(std::io::stdout(), cursor::Show).unwrap();
                        #[allow(clippy::exit)]
                        std::process::exit(0);
                    });
                    execute!(self.output, style::Print("\n"))?;
                    self.spinner = Some(Spinner::new(Spinners::Dots, "Thinking...".to_owned()));
                }

                self.conversation_state.append_new_user_message(user_input).await;
                self.send_tool_use_telemetry().await;
                Ok(Some(
                    self.client
                        .send_message(self.conversation_state.as_sendable_conversation_state())
                        .await?,
                ))
            },
        }
    }

    fn terminal_width(&self) -> usize {
        (self.terminal_width_provider)().unwrap_or(80)
    }

    async fn send_tool_use_telemetry(&mut self) {
        let futures = self.tool_use_events.drain(..).map(|event| async {
            let event: fig_telemetry::EventType = event.into();
            let app_event = fig_telemetry::AppTelemetryEvent::new(event).await;
            fig_telemetry::send_event(app_event).await;
        });
        let _ = futures::stream::iter(futures)
            .buffer_unordered(10)
            .collect::<Vec<_>>()
            .await;
    }
}

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
    pub fn from_conversation_id(conv_id: String) -> Self {
        Self {
            conversation_id: conv_id,
            utterance_id: None,
            user_input_id: None,
            tool_use_id: None,
            tool_name: None,
            is_accepted: false,
            is_success: None,
            is_valid: None,
        }
    }

    pub fn set_tool_use_id(mut self, id: &String) -> Self {
        self.tool_use_id.replace(id.to_string());
        self
    }

    pub fn set_tool_name(mut self, name: &String) -> Self {
        self.tool_name.replace(name.to_string());
        self
    }

    pub fn set_utterance_id(mut self, id: &String) -> Self {
        self.utterance_id.replace(id.to_string());
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

#[cfg(test)]
mod tests {
    use fig_api_client::model::ChatResponseStream;
    use serde_json::Map;

    use super::*;

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

    fn test_stream() -> serde_json::Value {
        serde_json::json!([
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
        ])
    }

    #[tokio::test]
    async fn test_flow() {
        let ctx = Context::builder().with_test_home().await.unwrap().build_fake();
        let mut output = std::io::stdout();
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
            client: create_stream(test_stream()),
            terminal_width_provider: || Some(80),
        };

        ChatContext::new(c).try_chat().await.unwrap();

        assert_eq!(ctx.fs().read_to_string("/file.txt").await.unwrap(), "Hello, world!");
    }
}
