use std::time::Duration;

use fig_ipc::SendMessage;
use fig_proto::daemon::daemon_message::Command;
use fig_proto::daemon::{
    DaemonMessage,
    DispatchHttpRequestCommand,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Ipc(#[from] fig_ipc::Error),
    #[error(transparent)]
    Request(#[from] fig_request::Error),
}

async fn send_daemon_message(message: DaemonMessage) -> Result<(), fig_ipc::Error> {
    let daemon_socket_path = fig_util::directories::daemon_socket_path()?;
    let mut conn = fig_ipc::BufferedUnixStream::connect_timeout(daemon_socket_path, Duration::from_secs(1)).await?;
    conn.send_message(message).await?;
    Ok(())
}

pub async fn send_to_daemon<V>(body: graphql_client::QueryBody<V>, fallback: bool) -> Result<(), Error>
where
    V: serde::Serialize,
{
    let body = serde_json::to_string(&body).unwrap();

    let mut url = fig_settings::api::host();
    url.set_path("/graphql");

    let message = DaemonMessage {
        id: None,
        no_response: Some(true),
        command: Some(Command::DispatchHttpRequest(DispatchHttpRequestCommand {
            method: "POST".to_string(),
            body: body.clone(),
            url: url.to_string(),
            auth: true,
        })),
    };

    match send_daemon_message(message).await {
        Ok(()) => Ok(()),
        Err(err) => {
            if fallback {
                fig_request::Request::post("/graphql")
                    .auth()
                    .body(body)
                    .graphql()
                    .await?;
                Ok(())
            } else {
                Err(err.into())
            }
        },
    }
}
