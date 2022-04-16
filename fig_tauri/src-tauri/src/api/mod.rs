use crate::utils::truncate_string;
use bytes::BytesMut;
use fig_proto::fig::client_originated_message::Submessage as ClientOriginatedSubMessage;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::{
    fig::{ClientOriginatedMessage, ServerOriginatedMessage},
    prost::Message,
};
use serde::Serialize;
use tauri::Window;
use tracing::warn;

pub mod debugger;
pub mod figterm;
mod fs;
mod notifications;
mod process;
pub mod properties;
mod settings;
pub mod window;

const FIG_GLOBAL_ERROR_OCCURRED: &str = "FigGlobalErrorOccurred";
pub const FIG_PROTO_MESSAGE_RECIEVED: &str = "FigProtoMessageRecieved";

#[derive(Debug)]
pub enum ResponseKind {
    Error(String),
    Success,
    Message(Box<ServerOriginatedSubMessage>),
}

impl From<ServerOriginatedSubMessage> for ResponseKind {
    fn from(from: ServerOriginatedSubMessage) -> Self {
        ResponseKind::Message(Box::new(from))
    }
}

#[derive(Serialize, Debug)]
pub enum ApiRequestError {
    DecodeError,
    EncodeError,
}

#[tauri::command]
pub async fn handle_api_request(window: Window, client_originated_message_b64: String) {
    let res = handle_request(base64::decode(client_originated_message_b64).unwrap()).await;
    match res {
        Ok(data) => window.emit(FIG_PROTO_MESSAGE_RECIEVED, base64::encode(data)),
        Err(ApiRequestError::DecodeError) => window.emit(FIG_GLOBAL_ERROR_OCCURRED, "Decode error"),
        Err(ApiRequestError::EncodeError) => window.emit(FIG_GLOBAL_ERROR_OCCURRED, "Encode error"),
    }
    .unwrap();
}

async fn handle_request(data: Vec<u8>) -> Result<BytesMut, ApiRequestError> {
    let message = ClientOriginatedMessage::decode(data.as_slice())
        .map_err(|_| ApiRequestError::DecodeError)?;

    // TODO: return error
    let message_id = message.id.unwrap();

    macro_rules! route {
        ($($struct: ident => $func: path)*) => {
            match message.submessage {
                $(
                    Some(ClientOriginatedSubMessage::$struct(request)) => $func(request, message_id).await,
                )*
                _ => {
                    let truncated = truncate_string(format!("{:?}", message), 150);
                    warn!("Missing handler: {}", truncated);
                    Err(ResponseKind::Error(format!("Unknown submessage {}", truncated)))
                }
            }
        }
    }

    let response = route! {
        /* fs */
        ReadFileRequest => fs::read_file
        WriteFileRequest => fs::write_file
        AppendToFileRequest => fs::append_to_file
        DestinationOfSymbolicLinkRequest => fs::destination_of_symbolic_link
        ContentsOfDirectoryRequest => fs::contents_of_directory
        /* settings */
        GetSettingsPropertyRequest => settings::get
        UpdateSettingsPropertyRequest => settings::update
        /* notifications */
        NotificationRequest => notifications::handle_request
        /* processes */
        RunProcessRequest => process::run
        PseudoterminalExecuteRequest => process::execute
        PseudoterminalWriteRequest => process::write
        /* window */
        PositionWindowRequest => window::position_window
        /* debugger */
        DebuggerUpdateRequest => debugger::update
        /* properties */
        UpdateApplicationPropertiesRequest => properties::update
        /* figterm */
        InsertTextRequest => figterm::insert_text
    }
    .unwrap_or_else(|s| s);

    let message = ServerOriginatedMessage {
        id: message.id,
        submessage: Some(match response {
            ResponseKind::Error(msg) => {
                warn!("Send error response: {}", msg);
                ServerOriginatedSubMessage::Error(msg)
            }
            ResponseKind::Success => ServerOriginatedSubMessage::Success(true),
            ResponseKind::Message(m) => *m,
        }),
    };

    let mut encoded = BytesMut::new();
    message
        .encode(&mut encoded)
        .map_err(|_| ApiRequestError::EncodeError)?;

    Ok(encoded)
}

pub type ResponseResult = Result<ResponseKind, ResponseKind>;

#[macro_export]
macro_rules! response_error {
    ($($arg:tt)*) => {{
        |_| ResponseKind::Error(format!($($arg)*))
    }};
}
