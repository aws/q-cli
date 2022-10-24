use fig_proto::fig::{
    CheckForUpdatesRequest,
    CheckForUpdatesResponse,
    UpdateApplicationRequest,
};

use super::{
    RequestResult,
    RequestResultImpl,
    ServerOriginatedSubMessage,
};

pub async fn update_application(_request: UpdateApplicationRequest) -> RequestResult {
    tokio::spawn(fig_install::update(true, Some(Box::new(|_| {})), true));
    RequestResult::success()
}

pub async fn check_for_updates(_request: CheckForUpdatesRequest) -> RequestResult {
    fig_install::check_for_updates(true)
        .await
        .map(|res| {
            Box::new(ServerOriginatedSubMessage::CheckForUpdatesResponse(
                CheckForUpdatesResponse {
                    is_update_available: Some(res.is_some()),
                    version: res.map(|update| update.version),
                },
            ))
        })
        .map_err(|err| format!("Failed to check for updates: {err}").into())
}
