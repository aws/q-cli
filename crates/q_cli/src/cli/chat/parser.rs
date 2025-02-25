use eyre::Result;
use fig_api_client::clients::SendMessageOutput;
use fig_api_client::model::{
    AssistantResponseMessage,
    ChatMessage,
    ChatResponseStream,
    ToolUse as FigToolUse,
};
use tracing::{
    error,
    trace,
};

use super::tools::serde_value_to_document;
use crate::cli::chat::conversation_state::Message;

/// Represents a tool use requested by the assistant.
#[derive(Debug, Clone)]
pub struct ToolUse {
    /// Corresponds to the `"toolUseId"` returned by the model.
    pub id: String,
    pub name: String,
    /// The tool arguments.
    pub args: serde_json::Value,
}

impl From<ToolUse> for FigToolUse {
    fn from(value: ToolUse) -> Self {
        Self {
            tool_use_id: value.id,
            name: value.name,
            input: serde_value_to_document(value.args),
        }
    }
}

/// State associated with parsing a [ConverseStreamResponse] into a [Message].
///
/// # Usage
///
/// You should repeatedly call [Self::recv] to receive [ResponseEvent]'s until a
/// [ResponseEvent::EndStream] value is returned.
#[derive(Debug)]
pub struct ResponseParser {
    /// The response to consume and parse into a sequence of [Ev].
    response: SendMessageOutput,
    /// Buffer to hold the next event in [SendMessageOutput].
    peek: Option<ChatResponseStream>,
    /// Message identifier for the assistant's response.
    message_id: Option<String>,
    /// Buffer for holding the accumulated assistant response.
    assistant_text: String,
    /// Tool uses requested by the model.
    tool_uses: Vec<ToolUse>,
    /// Buffered line required in case we need to discard a code reference event
    buffered_line: Option<String>,
    /// Short circuit and return early since we simply need to clear our buffered line
    short_circuit: bool,
    /// Whether or not we are currently receiving tool use delta events. Tuple of
    /// `Some((tool_use_id, name))` if true, [None] otherwise.
    parsing_tool_use: Option<(String, String)>,
}

impl ResponseParser {
    pub fn new(response: SendMessageOutput) -> Self {
        Self {
            response,
            peek: None,
            message_id: None,
            assistant_text: String::new(),
            tool_uses: Vec::new(),
            buffered_line: None,
            short_circuit: false,
            parsing_tool_use: None,
        }
    }

    /// Consumes the associated [ConverseStreamResponse] until a valid [ResponseEvent] is parsed.
    pub async fn recv(&mut self) -> Result<ResponseEvent> {
        if self.short_circuit {
            let message = Message(ChatMessage::AssistantResponseMessage(AssistantResponseMessage {
                message_id: self.message_id.take(),
                content: std::mem::take(&mut self.assistant_text),
                tool_uses: if self.tool_uses.is_empty() {
                    None
                } else {
                    Some(self.tool_uses.clone().into_iter().map(Into::into).collect())
                },
            }));
            return Ok(ResponseEvent::EndStream { message });
        }

        if let Some((id, name)) = self.parsing_tool_use.take() {
            let tool_use = self.parse_tool_use(id, name).await?;
            self.tool_uses.push(tool_use.clone());
            return Ok(ResponseEvent::ToolUse(tool_use));
        }

        loop {
            match self.next().await {
                Ok(Some(output)) => match output {
                    ChatResponseStream::AssistantResponseEvent { content } => {
                        self.assistant_text.push_str(&content);
                        let text = self.buffered_line.take();
                        self.buffered_line = Some(content);
                        if let Some(text) = text {
                            return Ok(ResponseEvent::AssistantText(text));
                        }
                    },
                    ChatResponseStream::CodeReferenceEvent(_) => {
                        self.buffered_line = None;
                    },
                    ChatResponseStream::InvalidStateEvent { reason, message } => {
                        error!(%reason, %message, "invalid state event");
                    },
                    ChatResponseStream::MessageMetadataEvent {
                        conversation_id,
                        utterance_id,
                    } => {
                        if let Some(id) = utterance_id {
                            self.message_id = Some(id);
                        }
                        if let Some(id) = conversation_id {
                            return Ok(ResponseEvent::ConversationId(id));
                        }
                    },
                    ChatResponseStream::ToolUseEvent {
                        tool_use_id,
                        name,
                        input,
                        stop,
                    } => {
                        debug_assert!(input.is_none(), "Unexpected initial content in first tool use event");
                        debug_assert!(
                            stop.is_none_or(|v| !v),
                            "Unexpected immediate stop in first tool use event"
                        );
                        self.parsing_tool_use = Some((tool_use_id.clone(), name.clone()));
                        return Ok(ResponseEvent::ToolUseStart { name });
                    },
                    _ => {},
                },
                Ok(None) => {
                    if let Some(text) = self.buffered_line.take() {
                        self.short_circuit = true;
                        return Ok(ResponseEvent::AssistantText(text));
                    }

                    let message = Message(ChatMessage::AssistantResponseMessage(AssistantResponseMessage {
                        message_id: self.message_id.take(),
                        content: std::mem::take(&mut self.assistant_text),
                        tool_uses: if self.tool_uses.is_empty() {
                            None
                        } else {
                            Some(self.tool_uses.clone().into_iter().map(Into::into).collect())
                        },
                    }));
                    return Ok(ResponseEvent::EndStream { message });
                },
                Err(err) => return Err(err.into()),
            }
        }
    }

    /// Consumes the response stream until a valid [ToolUse] is parsed.
    ///
    /// The arguments are the fields from the first [ChatResponseStream::ToolUseEvent] consumed.
    async fn parse_tool_use(&mut self, id: String, name: String) -> Result<ToolUse> {
        let mut tool_string = String::new();
        while let Some(ChatResponseStream::ToolUseEvent { .. }) = self.peek().await? {
            if let Some(ChatResponseStream::ToolUseEvent { input, stop, .. }) = self.next().await? {
                if let Some(i) = input {
                    tool_string.push_str(&i);
                }
                if let Some(true) = stop {
                    break;
                }
            }
        }
        let args = serde_json::from_str(&tool_string)?;
        Ok(ToolUse { id, name, args })
    }

    /// Returns the next event in the [SendMessageOutput] without consuming it.
    async fn peek(&mut self) -> Result<Option<&ChatResponseStream>, fig_api_client::Error> {
        if self.peek.is_some() {
            return Ok(self.peek.as_ref());
        }
        match self.next().await? {
            Some(v) => {
                self.peek = Some(v);
                Ok(self.peek.as_ref())
            },
            None => Ok(None),
        }
    }

    /// Consumes the next [SendMessageOutput] event.
    async fn next(&mut self) -> Result<Option<ChatResponseStream>, fig_api_client::Error> {
        if let Some(ev) = self.peek.take() {
            return Ok(Some(ev));
        }
        trace!("Attempting to recv next event");
        let r = self.response.recv().await?;
        trace!(?r, "Received new event");
        Ok(r)
    }
}

#[derive(Debug)]
pub enum ResponseEvent {
    /// Conversation identifier returned by the backend.
    ConversationId(String),
    /// Text returned by the assistant. This should be displayed to the user as it is received.
    AssistantText(String),
    /// Notification that a tool use is being received.
    ToolUseStart { name: String },
    /// A tool use requested by the assistant. This should be displayed to the user as it is
    /// received.
    ToolUse(ToolUse),
    /// Represents the end of the response. No more events will be returned.
    EndStream {
        /// The completed message containing all of the assistant text and tool use events
        /// previously emitted. This should be stored in the conversation history and sent in
        /// subsequent requests.
        message: Message,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse() {
        let _ = tracing_subscriber::fmt::try_init();
        let tool_use_id = "TEST_ID".to_string();
        let tool_name = "execute_bash".to_string();
        let tool_args = serde_json::json!({
            "command": "echo hello"
        })
        .to_string();
        let tool_use_split_at = 5;
        let mut events = vec![
            ChatResponseStream::AssistantResponseEvent {
                content: "hi".to_string(),
            },
            ChatResponseStream::AssistantResponseEvent {
                content: " there".to_string(),
            },
            ChatResponseStream::ToolUseEvent {
                tool_use_id: tool_use_id.clone(),
                name: tool_name.clone(),
                input: None,
                stop: None,
            },
            ChatResponseStream::ToolUseEvent {
                tool_use_id: tool_use_id.clone(),
                name: tool_name.clone(),
                input: Some(tool_args.as_str().split_at(tool_use_split_at).0.to_string()),
                stop: None,
            },
            ChatResponseStream::ToolUseEvent {
                tool_use_id: tool_use_id.clone(),
                name: tool_name.clone(),
                input: Some(tool_args.as_str().split_at(tool_use_split_at).1.to_string()),
                stop: None,
            },
            ChatResponseStream::ToolUseEvent {
                tool_use_id: tool_use_id.clone(),
                name: tool_name.clone(),
                input: None,
                stop: Some(true),
            },
        ];
        events.reverse();
        let mock = SendMessageOutput::Mock(events);
        let mut parser = ResponseParser::new(mock);

        for _ in 0..5 {
            println!("{:?}", parser.recv().await.unwrap());
        }
    }
}
