use std::io::Cursor;

use base64;
use fig_proto::fig::client_originated_message::Submessage as ClientOriginatedSubMessage;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::{
    fig::{ClientOriginatedMessage, ServerOriginatedMessage},
    prost::Message,
    FigMessage, FigProtobufEncodable,
};
use serde::Serialize;

mod fs;
mod settings;

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

#[derive(Serialize)]
pub enum ApiRequestError {
    DecodeError,
    EncodeError,
}

#[tauri::command]
pub async fn handle_api_request(client_originated_message_b64: String) {
    let _res = handle_request(base64::decode(client_originated_message_b64).unwrap()).await;
}

async fn handle_request(data: Vec<u8>) -> Result<Vec<u8>, ApiRequestError> {
    let message = ClientOriginatedMessage::decode(data.as_slice())
        .map_err(|_| ApiRequestError::DecodeError)?;

    macro_rules! route {
        ($($struct: ident => $func: path)*) => {
            match message.submessage {
                $(
                    Some(ClientOriginatedSubMessage::$struct(request)) => $func(request).await,
                )*
                _ => Err(ResponseKind::Error("Unknown submessage".to_string()))
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
    }
    .unwrap_or_else(|s| s);

    let message = ServerOriginatedMessage {
        id: message.id,
        submessage: Some(match response {
            ResponseKind::Error(msg) => ServerOriginatedSubMessage::Error(msg),
            ResponseKind::Success => ServerOriginatedSubMessage::Success(true),
            ResponseKind::Message(m) => *m,
        }),
    };

    let encoded = message
        .encode_fig_protobuf()
        .map_err(|_| ApiRequestError::EncodeError)?;

    Ok(encoded.inner.into_iter().collect::<Vec<u8>>())
}

pub type ResponseResult = Result<ResponseKind, ResponseKind>;

#[macro_export]
macro_rules! response_error {
    ($text: expr) => {
        |_| ResponseKind::Error($text.to_string())
    };
}
