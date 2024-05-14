use std::io::{
    stdout,
    Write,
};
use std::process::ExitCode;
use std::time::Duration;

use fig_ipc::{
    BufferedUnixStream,
    SendRecvMessage,
};
use fig_proto::figterm::figterm_request_message::Request;
use fig_proto::figterm::figterm_response_message::Response;
use fig_proto::figterm::{
    FigtermRequestMessage,
    FigtermResponseMessage,
    InlineShellCompletionRequest,
    InlineShellCompletionResponse,
};
use fig_telemetry::InlineShellCompletionActionedOptions;
use fig_util::env_var::QTERM_SESSION_ID;
use tracing::error;

const TIMEOUT: Duration = Duration::from_secs(5);

macro_rules! unwrap_or_exit {
    ($expr:expr, $err_msg:expr) => {
        match $expr {
            Ok(value) => value,
            Err(err) => {
                error!(%err, $err_msg);
                return ExitCode::FAILURE;
            }
        }
    };
}

pub(super) async fn inline_shell_completion(buffer: String) -> ExitCode {
    let session_id = unwrap_or_exit!(std::env::var(QTERM_SESSION_ID), "Failed to get session ID");

    let figterm_socket_path = unwrap_or_exit!(
        fig_util::directories::figterm_socket_path(&session_id),
        "Failed to get figterm socket path"
    );

    let mut conn = unwrap_or_exit!(
        BufferedUnixStream::connect(figterm_socket_path).await,
        "Failed to connect to figterm"
    );

    match conn
        .send_recv_message_timeout(
            FigtermRequestMessage {
                request: Some(Request::InlineShellCompletion(InlineShellCompletionRequest {
                    buffer: buffer.clone(),
                })),
            },
            TIMEOUT,
        )
        .await
    {
        Ok(Some(FigtermResponseMessage {
            response:
                Some(Response::InlineShellCompletion(InlineShellCompletionResponse {
                    insert_text: Some(insert_text),
                })),
        })) => {
            let _ = writeln!(stdout(), "{buffer}{insert_text}");
            ExitCode::SUCCESS
        },
        Ok(res) => {
            error!(?res, "Unexpected response from figterm");
            ExitCode::FAILURE
        },
        Err(err) => {
            error!(%err, "Failed to get inline shell completion from figterm");
            ExitCode::FAILURE
        },
    }
}

pub(super) async fn inline_shell_completion_accept(buffer: String, suggestion: String) -> ExitCode {
    fig_telemetry::send_inline_shell_completion_actioned(InlineShellCompletionActionedOptions {
        // TODO: fix fields, these are unused at the moment though
        session_id: "unused".into(),
        request_id: "unused".into(),
        accepted: true,
        edit_buffer_len: buffer.len() as i64,
        suggested_chars_len: suggestion.len() as i64,
        latency: Duration::ZERO,
    })
    .await;
    ExitCode::SUCCESS

    //    let session_id = unwrap_or_exit!(std::env::var(QTERM_SESSION_ID), "Failed to get session
    // ID");
    //
    //    let figterm_socket_path = unwrap_or_exit!(
    //        fig_util::directories::figterm_socket_path(&session_id),
    //        "Failed to get figterm socket path"
    //    );
    //
    //    let mut conn = unwrap_or_exit!(
    //        BufferedUnixStream::connect(figterm_socket_path).await,
    //        "Failed to connect to figterm"
    //    );
    //
    //    match conn
    //        .send_recv_message_timeout(
    //            FigtermRequestMessage {
    //                request: Some(Request::InlineShellCompletionTelemetry(
    //                    InlineShellCompletionTelemetryRequest { buffer, suggestion },
    //                )),
    //            },
    //            TIMEOUT,
    //        )
    //        .await
    //    {
    //        Ok(Some(FigtermResponseMessage {
    //            response:
    // Some(Response::InlineShellCompletionTelemetry(InlineShellCompletionTelemetryResponse {})),
    //        })) => ExitCode::SUCCESS,
    //        Ok(res) => {
    //            error!(?res, "Unexpected response from figterm");
    //            ExitCode::FAILURE
    //        },
    //        Err(err) => {
    //            error!(%err, "Failed to get response from figterm");
    //            ExitCode::FAILURE
    //        },
    //    }
}
