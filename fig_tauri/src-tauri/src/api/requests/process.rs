use std::process::Stdio;

use anyhow::anyhow;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    PseudoterminalExecuteRequest,
    PseudoterminalExecuteResponse,
    RunProcessRequest,
    RunProcessResponse,
};
use tokio::process::Command;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::native::{
    SHELL,
    SHELL_ARGS,
};

// TODO(mia): implement actual pseudoterminal stuff
pub async fn execute(request: PseudoterminalExecuteRequest) -> RequestResult {
    let mut cmd = Command::new(SHELL);
    cmd.args(SHELL_ARGS);
    cmd.stdin(Stdio::inherit());

    cfg_if::cfg_if!(
        if #[cfg(target_os="windows")] {
            // account for weird behavior passing in commands containing && to WSL
            cmd.args(request.command.split(' ').collect::<Vec<&str>>());
        } else {
            cmd.arg(&request.command);
        }
    );

    if let Some(working_directory) = request.working_directory {
        cmd.current_dir(working_directory);
    }

    for var in request.env {
        cmd.env(var.key.clone(), var.value());
    }

    let output = cmd
        .output()
        .await
        .map_err(|_| anyhow!("Failed running command: {:?}", request.command))?;

    RequestResult::Ok(Box::new(ServerOriginatedSubMessage::PseudoterminalExecuteResponse(
        PseudoterminalExecuteResponse {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: if output.stderr.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            exit_code: output.status.code(),
        },
    )))
}

pub async fn run(request: RunProcessRequest) -> RequestResult {
    let mut cmd = Command::new(&request.executable);
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
        .map_err(|_| anyhow!("Failed running command: {:?}", request.executable))?;

    RequestResult::Ok(Box::new(ServerOriginatedSubMessage::RunProcessResponse(
        RunProcessResponse {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(0),
        },
    )))
}

pub async fn write() -> RequestResult {
    RequestResult::error("PseudoterminalWriteRequest is deprecated".to_string())
}
