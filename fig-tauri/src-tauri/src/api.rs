use std::io::Cursor;

use base64;
use fig_proto::{FigMessage, fig::{ReadFileRequest, ReadFileResponse, ClientOriginatedMessage, ServerOriginatedMessage}, prost::Message, FigProtobufEncodable};
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::client_originated_message::Submessage as ClientOriginatedSubMessage;
use serde::Serialize;

use crate::os::native;

enum ResponseKind {
    Error(String),
    Success,
    Message(ServerOriginatedSubMessage),
}

#[derive(Serialize)]
pub enum ApiRequestError {
    DecodeError,
    EncodeError,
}

#[tauri::command]
pub async fn handle_api_request(mut client_originated_message_b64: String) {
    handle_request(base64::decode(client_originated_message_b64).unwrap()).await;
}

async fn handle_request(mut data: Vec<u8>) -> Result<Vec<u8>, ApiRequestError> {
    let message = ClientOriginatedMessage::decode(data.as_slice()).map_err(|_| ApiRequestError::DecodeError)?;
   
    println!("{:?}", message);

    let response = match message.submessage {
        Some(ClientOriginatedSubMessage::ReadFileRequest(request)) => read_file(request).await,
        _ => Err(ResponseKind::Error("Unknown submessage".to_string())),
    }.unwrap_or_else(|s| s);

    let message = ServerOriginatedMessage {
        id: message.id,
        submessage: Some(match response {
            ResponseKind::Error(msg) => ServerOriginatedSubMessage::Error(msg),
            ResponseKind::Success => ServerOriginatedSubMessage::Success(true),
            ResponseKind::Message(m) => m,
        })
    };

    let encoded = message.encode_fig_protobuf().map_err(|_| ApiRequestError::EncodeError)?;

    Ok(encoded.inner.into_iter().collect::<Vec<u8>>())
}

type ResponseResult = Result<ResponseKind, ResponseKind>;

async fn read_file(request: ReadFileRequest) -> ResponseResult {
    use fig_proto::fig::read_file_response::Type;
    let file_path = native::resolve_path(request.path.unwrap()).map_err(|_| ResponseKind::Error("Invalid path".to_string()))?;
    let file_contents = native::read_file(&file_path).await.map_err(|_| ResponseKind::Error("Failed reading file".to_string()))?;
    let kind = if request.is_binary_file {
        Type::Data(file_contents)
    } else {
        Type::Text(String::from_utf8(file_contents).map_err(|_| ResponseKind::Error("Invalid file encoding".to_string()))?)
    };
    let response = ServerOriginatedSubMessage::ReadFileResponse(ReadFileResponse {
        r#type: Some(kind),
    });

    Ok(ResponseKind::Message(response))
}
