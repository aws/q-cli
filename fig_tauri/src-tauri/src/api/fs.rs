use anyhow::anyhow;
use camino::Utf8PathBuf;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    AppendToFileRequest,
    ContentsOfDirectoryRequest,
    ContentsOfDirectoryResponse,
    DestinationOfSymbolicLinkRequest,
    DestinationOfSymbolicLinkResponse,
    ReadFileRequest,
    ReadFileResponse,
    WriteFileRequest,
};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::api::{
    RequestResult,
    RequestResultImpl,
};
use crate::utils::{
    build_filepath,
    resolve_filepath,
};

pub async fn read_file(request: ReadFileRequest) -> RequestResult {
    use fig_proto::fig::read_file_response::Type;
    let path = request.path.as_ref().ok_or_else(|| anyhow!("No path provided"))?;
    let resolved_path = resolve_filepath(path);
    let kind = if request.is_binary_file() {
        Type::Data(
            tokio::fs::read(&*resolved_path)
                .await
                .map_err(|_| anyhow!("Failed reading file: {resolved_path}"))?,
        )
    } else {
        Type::Text(
            tokio::fs::read_to_string(&*resolved_path)
                .await
                .map_err(|_| anyhow!("Failed reading file: {resolved_path}"))?,
        )
    };
    let response = ServerOriginatedSubMessage::ReadFileResponse(ReadFileResponse { r#type: Some(kind) });

    Ok(response.into())
}

pub async fn write_file(request: WriteFileRequest) -> RequestResult {
    use fig_proto::fig::write_file_request::Data;
    let path = request.path.as_ref().ok_or_else(|| anyhow!("No path provided"))?;
    let resolved_path = resolve_filepath(path);
    match request.data.unwrap() {
        Data::Binary(data) => tokio::fs::write(&*resolved_path, data)
            .await
            .map_err(|_| anyhow!("Failed writing to file: {resolved_path}"))?,
        Data::Text(data) => tokio::fs::write(&*resolved_path, data.as_bytes())
            .await
            .map_err(|_| anyhow!("Failed writing to file: {resolved_path}"))?,
    }

    RequestResult::success()
}

pub async fn append_to_file(request: AppendToFileRequest) -> RequestResult {
    use fig_proto::fig::append_to_file_request::Data;
    let path = request.path.as_ref().ok_or_else(|| anyhow!("No path provided"))?;
    let resolved_path = resolve_filepath(path);
    let mut file = OpenOptions::new()
        .append(true)
        .open(&*resolved_path)
        .await
        .map_err(|_| anyhow!("Failed opening file: {resolved_path}"))?;

    match request.data.unwrap() {
        Data::Binary(data) => file
            .write(&data)
            .await
            .map_err(|_| anyhow!("Failed writing to file: {resolved_path}"))?,
        Data::Text(data) => file
            .write(data.as_bytes())
            .await
            .map_err(|_| anyhow!("Failed writing to file: {resolved_path}"))?,
    };

    RequestResult::success()
}

pub async fn destination_of_symbolic_link(request: DestinationOfSymbolicLinkRequest) -> RequestResult {
    let path = request.path.as_ref().ok_or_else(|| anyhow!("No path provided"))?;
    let resolved_path = resolve_filepath(path);
    let real_path: Utf8PathBuf = tokio::fs::canonicalize(&*resolved_path)
        .await
        .map_err(|_| anyhow!("Failed resolving symlink: {resolved_path}"))?
        .try_into()?;

    let response = ServerOriginatedSubMessage::DestinationOfSymbolicLinkResponse(DestinationOfSymbolicLinkResponse {
        destination: Some(build_filepath(real_path)),
    });

    Ok(response.into())
}

pub async fn contents_of_directory(request: ContentsOfDirectoryRequest) -> RequestResult {
    let path = request.directory.as_ref().ok_or_else(|| anyhow!("No path provided"))?;
    let resolved_path = resolve_filepath(path);
    let mut stream = tokio::fs::read_dir(&*resolved_path)
        .await
        .map_err(|_| anyhow!("Failed listing directory: {resolved_path}"))?;

    let mut contents = Vec::new();
    while let Some(item) = stream
        .next_entry()
        .await
        .map_err(|_| anyhow!("Failed listing directory entries: {resolved_path}"))?
    {
        contents.push(item.file_name().to_string_lossy().to_string());
    }

    let response =
        ServerOriginatedSubMessage::ContentsOfDirectoryResponse(ContentsOfDirectoryResponse { file_names: contents });

    Ok(response.into())
}

pub async fn create_directory_request(request: fig_proto::fig::CreateDirectoryRequest) -> RequestResult {
    let path = request.path.as_ref().ok_or_else(|| anyhow!("No path provided"))?;
    let resolved_path = resolve_filepath(path);
    if request.recursive() {
        tokio::fs::create_dir_all(&*resolved_path)
            .await
            .map_err(|_| anyhow!("Error"))?;
    } else {
        tokio::fs::create_dir(&*resolved_path)
            .await
            .map_err(|_| anyhow!("Error"))?;
    }

    RequestResult::success()
}
