use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    AppendToFileRequest, ContentsOfDirectoryRequest, ContentsOfDirectoryResponse,
    DestinationOfSymbolicLinkRequest, DestinationOfSymbolicLinkResponse, WriteFileRequest,
};
use fig_proto::fig::{ReadFileRequest, ReadFileResponse};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::api::ResponseKind;
use crate::os::native;
use crate::response_error;

use super::ResponseResult;

pub async fn read_file(request: ReadFileRequest, _message_id: i64) -> ResponseResult {
    use fig_proto::fig::read_file_response::Type;
    let file_path = native::resolve_filepath(request.path.unwrap());
    let kind = if request.is_binary_file {
        Type::Data(
            tokio::fs::read(&file_path)
                .await
                .map_err(response_error!("Failed reading file"))?,
        )
    } else {
        Type::Text(
            tokio::fs::read_to_string(&file_path)
                .await
                .map_err(response_error!("Failed reading file"))?,
        )
    };
    let response =
        ServerOriginatedSubMessage::ReadFileResponse(ReadFileResponse { r#type: Some(kind) });

    Ok(response.into())
}

pub async fn write_file(request: WriteFileRequest, _message_id: i64) -> ResponseResult {
    use fig_proto::fig::write_file_request::Data;
    let file_path = native::resolve_filepath(request.path.unwrap());
    match request.data.unwrap() {
        Data::Binary(data) => tokio::fs::write(file_path, data)
            .await
            .map_err(response_error!("Failed writing to file"))?,
        Data::Text(data) => tokio::fs::write(file_path, data.as_bytes())
            .await
            .map_err(response_error!("Failed writing to file"))?,
    }

    Ok(ResponseKind::Success)
}

pub async fn append_to_file(request: AppendToFileRequest, _message_id: i64) -> ResponseResult {
    use fig_proto::fig::append_to_file_request::Data;
    let file_path = native::resolve_filepath(request.path.unwrap());
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .await
        .map_err(response_error!("Failed opening file"))?;

    match request.data.unwrap() {
        Data::Binary(data) => file
            .write(&data)
            .await
            .map_err(response_error!("Failed writing to file"))?,
        Data::Text(data) => file
            .write(data.as_bytes())
            .await
            .map_err(response_error!("Failed writing to file"))?,
    };

    Ok(ResponseKind::Success)
}

pub async fn destination_of_symbolic_link(
    request: DestinationOfSymbolicLinkRequest,
    _message_id: i64,
) -> ResponseResult {
    let file_path = native::resolve_filepath(request.path.unwrap());
    let real_path = tokio::fs::canonicalize(file_path)
        .await
        .map_err(response_error!("Failed resolving symlink"))?;

    let response = ServerOriginatedSubMessage::DestinationOfSymbolicLinkResponse(
        DestinationOfSymbolicLinkResponse {
            destination: Some(native::build_filepath(real_path)),
        },
    );

    Ok(response.into())
}

pub async fn contents_of_directory(
    request: ContentsOfDirectoryRequest,
    _message_id: i64,
) -> ResponseResult {
    let file_path = native::resolve_filepath(request.directory.unwrap());
    let mut stream = tokio::fs::read_dir(file_path)
        .await
        .map_err(response_error!("Failed listing directory"))?;

    let mut contents = Vec::new();
    while let Some(item) = stream
        .next_entry()
        .await
        .map_err(response_error!("Failed listing directory entries"))?
    {
        contents.push(item.file_name().to_string_lossy().to_string());
    }

    let response =
        ServerOriginatedSubMessage::ContentsOfDirectoryResponse(ContentsOfDirectoryResponse {
            file_names: contents,
        });

    Ok(response.into())
}
