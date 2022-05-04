use anyhow::anyhow;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    AppendToFileRequest, ContentsOfDirectoryRequest, ContentsOfDirectoryResponse,
    DestinationOfSymbolicLinkRequest, DestinationOfSymbolicLinkResponse, WriteFileRequest,
};
use fig_proto::fig::{ReadFileRequest, ReadFileResponse};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::api::{RequestResult, RequestResultImpl};
use crate::utils::{build_filepath, resolve_filepath};

pub async fn read_file(request: ReadFileRequest) -> RequestResult {
    use fig_proto::fig::read_file_response::Type;
    let file_path = resolve_filepath(request.path.unwrap());
    let kind = if request.is_binary_file.unwrap_or(false) {
        Type::Data(
            tokio::fs::read(&file_path)
                .await
                .map_err(|_| anyhow!("Failed reading file: {file_path:?}"))?,
        )
    } else {
        Type::Text(
            tokio::fs::read_to_string(&file_path)
                .await
                .map_err(|_| anyhow!("Failed reading file: {file_path:?}"))?,
        )
    };
    let response =
        ServerOriginatedSubMessage::ReadFileResponse(ReadFileResponse { r#type: Some(kind) });

    Ok(response.into())
}

pub async fn write_file(request: WriteFileRequest) -> RequestResult {
    use fig_proto::fig::write_file_request::Data;
    let file_path = resolve_filepath(request.path.unwrap());
    match request.data.unwrap() {
        Data::Binary(data) => tokio::fs::write(&file_path, data)
            .await
            .map_err(|_| anyhow!("Failed writing to file: {file_path:?}"))?,
        Data::Text(data) => tokio::fs::write(&file_path, data.as_bytes())
            .await
            .map_err(|_| anyhow!("Failed writing to file: {file_path:?}"))?,
    }

    RequestResult::success()
}

pub async fn append_to_file(request: AppendToFileRequest) -> RequestResult {
    use fig_proto::fig::append_to_file_request::Data;
    let file_path = resolve_filepath(request.path.unwrap());
    let mut file = OpenOptions::new()
        .append(true)
        .open(&file_path)
        .await
        .map_err(|_| anyhow!("Failed opening file: {file_path:?}"))?;

    match request.data.unwrap() {
        Data::Binary(data) => file
            .write(&data)
            .await
            .map_err(|_| anyhow!("Failed writing to file: {file_path:?}"))?,
        Data::Text(data) => file
            .write(data.as_bytes())
            .await
            .map_err(|_| anyhow!("Failed writing to file: {file_path:?}"))?,
    };

    RequestResult::success()
}

pub async fn destination_of_symbolic_link(
    request: DestinationOfSymbolicLinkRequest,
) -> RequestResult {
    let file_path = resolve_filepath(request.path.unwrap());
    let real_path = tokio::fs::canonicalize(&file_path)
        .await
        .map_err(|_| anyhow!("Failed resolving symlink: {file_path:?}"))?;

    let response = ServerOriginatedSubMessage::DestinationOfSymbolicLinkResponse(
        DestinationOfSymbolicLinkResponse {
            destination: Some(build_filepath(real_path)),
        },
    );

    Ok(response.into())
}

pub async fn contents_of_directory(request: ContentsOfDirectoryRequest) -> RequestResult {
    let file_path = resolve_filepath(request.directory.unwrap());
    let mut stream = tokio::fs::read_dir(&file_path)
        .await
        .map_err(|_| anyhow!("Failed listing directory: {file_path:?}"))?;

    let mut contents = Vec::new();
    while let Some(item) = stream
        .next_entry()
        .await
        .map_err(|_| anyhow!("Failed listing directory entries {file_path:?}"))?
    {
        contents.push(item.file_name().to_string_lossy().to_string());
    }

    let response =
        ServerOriginatedSubMessage::ContentsOfDirectoryResponse(ContentsOfDirectoryResponse {
            file_names: contents,
        });

    Ok(response.into())
}
