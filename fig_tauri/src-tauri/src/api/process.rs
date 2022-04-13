use tokio::process::Command;

use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    PseudoterminalExecuteRequest, PseudoterminalExecuteResponse, PseudoterminalWriteRequest,
    RunProcessRequest, RunProcessResponse,
};

use super::{ResponseKind, ResponseResult};
use crate::response_error;

// TODO(mia): implement actual pseudoterminal stuff
pub async fn execute(request: PseudoterminalExecuteRequest, _: i64) -> ResponseResult {
    let mut cmd = Command::new("/bin/bash");
    cmd.arg("--noprofile")
        .arg("--norc")
        .arg("-c")
        .arg(request.command);
    if let Some(working_directory) = request.working_directory {
        cmd.current_dir(working_directory);
    } else if let Ok(working_directory) = std::env::current_dir() {
        cmd.current_dir(working_directory);
    }
    for var in request.env {
        cmd.env(var.key.clone(), var.value());
    }
    let output = cmd
        .output()
        .await
        .map_err(response_error!("Failed running command"))?;

    Ok(ResponseKind::Message(Box::new(
        ServerOriginatedSubMessage::PseudoterminalExecuteResponse(PseudoterminalExecuteResponse {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: if output.stderr.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            exit_code: output.status.code(),
        }),
    )))
}

pub async fn run(request: RunProcessRequest, _: i64) -> ResponseResult {
    let mut cmd = Command::new(request.executable);
    if let Some(working_directory) = request.working_directory {
        cmd.current_dir(working_directory);
    } else if let Ok(working_directory) = std::env::current_dir() {
        cmd.current_dir(working_directory);
    }
    for arg in request.arguments {
        cmd.arg(arg);
    }
    for var in request.env {
        cmd.env(var.key.clone(), var.value());
    }

    let output = cmd
        .output()
        .await
        .map_err(response_error!("Failed running command"))?;

    Ok(ResponseKind::Message(Box::new(
        ServerOriginatedSubMessage::RunProcessResponse(RunProcessResponse {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(0),
        }),
    )))
}

pub async fn write(_: PseudoterminalWriteRequest, _: i64) -> ResponseResult {
    Err(ResponseKind::Error(
        "PseudoterminalWriteRequest is deprecated".to_string(),
    ))
}
