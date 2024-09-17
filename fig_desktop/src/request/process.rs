use std::time::Duration;

use fig_desktop_api::requests::Error;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    EnvironmentVariable,
    PseudoterminalExecuteRequest,
    PseudoterminalExecuteResponse,
    RunProcessRequest,
    RunProcessResponse,
};
use fig_proto::remote::hostbound;
use fig_remote_ipc::figterm::{
    FigtermCommand,
    FigtermSessionId,
    FigtermState,
};
use fig_util::env_var::{
    PROCESS_LAUNCHED_BY_Q,
    Q_TERM,
};
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
use crate::platform::PlatformState;

fn shell_args(shell_path: &str) -> &'static [&'static str] {
    let (_, shell_name) = shell_path.rsplit_once(['/', '\\']).unwrap_or(("", shell_path));
    match shell_name {
        "bash" | "bash.exe" => &["--norc", "--noprofile", "-c"],
        "zsh" | "zsh.exe" => &["--norcs", "-c"],
        "fish" | "fish.exe" => &["--no-config", "-c"],
        _ => {
            warn!(%shell_name, "unknown shell");
            &[]
        },
    }
}

fn set_fig_vars(cmd: &mut Command) {
    cmd.env(Q_TERM, env!("CARGO_PKG_VERSION"));
    cmd.env(PROCESS_LAUNCHED_BY_Q, "1");

    cmd.env("HISTFILE", "");
    cmd.env("HISTCONTROL", "ignoreboth");
    cmd.env("TERM", "xterm-256color");
}

pub async fn execute(request: PseudoterminalExecuteRequest, figterm_state: &FigtermState) -> RequestResult {
    debug!({
        term_session =? request.terminal_session_id,
        command = request.command,
        cwd = request.working_directory(),
        env =? request.env,
        background = request.background_job,
        pipelined = request.is_pipelined
    }, "Executing command");

    let session_sender = figterm_state
        .with_maybe_id(&request.terminal_session_id.map(FigtermSessionId::new), |session| {
            session.sender.clone()
        });

    if let Some(session_sender) = session_sender {
        let (message, rx) = FigtermCommand::pseudoterminal_execute(
            request.command,
            request.working_directory,
            request.background_job,
            request.is_pipelined,
            request.env,
        );
        session_sender
            .send(message)
            .map_err(|err| format!("failed sending command to figterm: {err}"))?;
        drop(session_sender);

        let response = timeout(Duration::from_secs(10), rx)
            .await
            .map_err(|err| Error::from_std(err).wrap_err("Qterm response timed out after 10 sec"))?
            .map_err(|err| Error::from_std(err).wrap_err("Qterm response failed to receive from sender"))?;

        if let hostbound::response::Response::PseudoterminalExecute(response) = response {
            RequestResult::Ok(Box::new(ServerOriginatedSubMessage::PseudoterminalExecuteResponse(
                PseudoterminalExecuteResponse {
                    stdout: response.stdout,
                    stderr: response.stderr,
                    exit_code: response.exit_code,
                },
            )))
        } else {
            Err("invalid response type".to_string().into())
        }
    } else {
        debug!("executing locally");

        let shell = PlatformState::shell();

        // note: we don't know what shell they use because we don't have any figterm sessions to check
        let args = shell_args(&shell);

        let mut cmd = Command::new(&*shell);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(windows::Win32::System::Threading::DETACHED_PROCESS.0);
        // TODO: better SHELL_ARGs handling here based on shell.
        // TODO: handle wsl distro from FigtermState here.
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
            .map_err(|err| format!("Failed running command {:?}: {err}", request.command))?;

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
        term_session =? request.terminal_session_id,
        exe = request.executable,
        args =? request.arguments,
        cwd = request.working_directory(),
        env =? request.env,
        timeout =? request.timeout,
    }, "Running command");

    let session_sender = state.with_maybe_id(&request.terminal_session_id.map(FigtermSessionId::new), |session| {
        session.sender.clone()
    });

    if let Some(session_sender) = session_sender {
        let (message, rx) = FigtermCommand::run_process(
            request.executable,
            request.arguments,
            request.working_directory,
            request.env,
        );
        session_sender
            .send(message)
            .map_err(|err| format!("failed sending command to figterm: {err}"))?;
        drop(session_sender);

        let timeout_duration = request
            .timeout
            .map(Duration::from)
            .unwrap_or_default()
            .max(Duration::from_secs(10));

        let response = timeout(timeout_duration, rx)
            .await
            .map_err(|_err| "Timed out waiting for figterm response")?
            .map_err(|err| format!("Failed to receive figterm response: {err}"))?;

        if let hostbound::response::Response::RunProcess(response) = response {
            RequestResult::Ok(Box::new(ServerOriginatedSubMessage::RunProcessResponse(
                RunProcessResponse { ..response },
            )))
        } else {
            Err("invalid response type".into())
        }
    } else {
        debug!("running locally");

        // TODO: we can infer shell as above for execute if no executable is provided.
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
            match var.value {
                Some(val) => cmd.env(var.key, val),
                None => cmd.env_remove(var.key),
            };
        }

        let output = cmd
            .output()
            .await
            .map_err(|err| format!("Failed running command {:?}: {err}", request.executable))?;

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
