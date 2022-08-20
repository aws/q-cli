use std::time::Duration;

use anyhow::{
    anyhow,
    bail,
};
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    EnvironmentVariable,
    PseudoterminalExecuteRequest,
    PseudoterminalExecuteResponse,
    RunProcessRequest,
    RunProcessResponse,
};
use fig_proto::secure::hostbound;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{
    debug,
    warn,
};

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::figterm::{
    FigtermCommand,
    FigtermState,
};
use crate::native::SHELL;

fn get_shell_path_from_state(state: &FigtermState) -> Option<String> {
    state.most_recent_session()?.context.as_ref()?.shell_path.clone()
}

fn shell_args(shell_path: &str) -> &'static [&'static str] {
    let (_, shell_name) = shell_path
        .rsplit_once(|c| c == '/' || c == '\\')
        .unwrap_or(("", shell_path));
    match shell_name {
        "bash" | "bash.exe" => &["--norc", "--noprofile", "-c"],
        "zsh" | "zsh.exe" => &["--norcs", "-c"],
        "fish" | "fish.exe" => &["--no-config", "-c"],
        _ => {
            warn!("unknown shell {shell_name}");
            &[]
        },
    }
}

fn set_fig_vars(cmd: &mut Command) {
    cmd.env("FIG_ENV_VAR", "1");
    cmd.env("FIG_SHELL_VAR", "1");
    cmd.env("FIG_TERM", "1");
    cmd.env("FIG_PTY", "1");
    cmd.env("PROCESS_LAUNCHED_BY_FIG", "1");
    cmd.env("HISTFILE", "");
    cmd.env("HISTCONTROL", "ignoreboth");
    cmd.env("TERM", "xterm-256color");
}

// todo(mia): implement actual pseudoterminal stuff
pub async fn execute(request: PseudoterminalExecuteRequest, state: &FigtermState) -> RequestResult {
    if let Some(session) = state.most_recent_session() {
        let (message, rx) = FigtermCommand::pseudoterminal_execute(
            request.command,
            request.working_directory,
            request.background_job,
            request.is_pipelined,
            request.env,
        );
        if let Err(err) = session.sender.send(message) {
            bail!("failed sending command to figterm: {err}");
        }
        drop(session);

        let response = timeout(Duration::from_secs(10), rx).await??;

        if let hostbound::response::Response::PseudoterminalExecute(response) = response {
            RequestResult::Ok(Box::new(ServerOriginatedSubMessage::PseudoterminalExecuteResponse(
                PseudoterminalExecuteResponse {
                    stdout: response.stdout,
                    stderr: response.stderr,
                    exit_code: response.exit_code,
                },
            )))
        } else {
            bail!("invalid response type");
        }
    } else {
        // fall back to executing locally
        // todo(mia): move this code into it's own crate so the logic is shared

        let shell = get_shell_path_from_state(state).unwrap_or_else(|| SHELL.into());
        let args = shell_args(&shell);
        debug!({
            shell,
            args =? args,
            command = request.command,
            cwd = request.working_directory(),
            env =? request.env
        }, "Executing command");

        let mut cmd = Command::new(shell);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(windows::Win32::System::Threading::DETACHED_PROCESS.0);
        // TODO(sean): better SHELL_ARGs handling here based on shell.
        // TODO(sean): handle wsl distro from FigtermState here.
        cmd.args(args);
        cmd.arg(&request.command);

        if let Some(working_directory) = request.working_directory {
            cmd.current_dir(working_directory);
        }

        set_fig_vars(&mut cmd);

        for EnvironmentVariable { key, value } in &request.env {
            match value {
                Some(value) => cmd.env(key, value),
                None => cmd.env_remove(key),
            };
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
}

pub async fn run(request: RunProcessRequest, state: &FigtermState) -> RequestResult {
    debug!({
        exe = request.executable,
        args =? request.arguments,
        cwd = request.working_directory(),
        env =? request.env
    }, "Running command");

    if let Some(session) = state.most_recent_session() {
        let (message, rx) = FigtermCommand::run_process(
            request.executable,
            request.arguments,
            request.working_directory,
            request.env,
        );
        if let Err(err) = session.sender.send(message) {
            bail!("failed sending command to figterm: {err}");
        }
        drop(session);

        let response = timeout(Duration::from_secs(10), rx).await??;

        if let hostbound::response::Response::RunProcess(response) = response {
            RequestResult::Ok(Box::new(ServerOriginatedSubMessage::RunProcessResponse(
                RunProcessResponse { ..response },
            )))
        } else {
            bail!("invalid response type");
        }
    } else {
        // fall back to executing locally
        // todo(mia): move this code into it's own crate so the logic is shared

        // TODO(sean) we can infer shell as above for execute if no executable is provided.
        let mut cmd = Command::new(&request.executable);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(windows::Win32::System::Threading::DETACHED_PROCESS.0);

        if let Some(working_directory) = request.working_directory {
            cmd.current_dir(working_directory);
        } else if let Ok(working_directory) = std::env::current_dir() {
            cmd.current_dir(working_directory);
        }
        for arg in request.arguments {
            cmd.arg(arg);
        }

        set_fig_vars(&mut cmd);

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
}

pub async fn write() -> RequestResult {
    RequestResult::error("PseudoterminalWriteRequest is deprecated".to_string())
}
