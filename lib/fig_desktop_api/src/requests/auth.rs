use auth::builder_id::{
    BuilderIdToken,
    PollCreateToken,
    StartDeviceAuthorizationResponse,
};
use auth::secret_store::SecretStore;
use fig_proto::fig::auth_builder_id_poll_create_token_response::PollStatus;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    AuthBuilderIdPollCreateTokenRequest,
    AuthBuilderIdPollCreateTokenResponse,
    AuthBuilderIdStartDeviceAuthorizationRequest,
    AuthBuilderIdStartDeviceAuthorizationResponse,
    AuthStatusRequest,
    AuthStatusResponse,
};

use super::RequestResult;
use crate::kv::KVStore;

const BUILDER_ID_DATA_KEY: &str = "builder-id-data";

pub async fn status(_request: AuthStatusRequest) -> RequestResult {
    let secret_store = SecretStore::load()
        .await
        .map_err(|err| format!("Failed to load secret store: {err}"))?;

    Ok(ServerOriginatedSubMessage::AuthStatusResponse(AuthStatusResponse {
        builder_id: matches!(BuilderIdToken::load(&secret_store).await, Ok(Some(_))),
    })
    .into())
}

pub async fn builder_id_start_device_authorization(
    _request: AuthBuilderIdStartDeviceAuthorizationRequest,
    ctx: &impl KVStore,
) -> RequestResult {
    let secret_store = SecretStore::load()
        .await
        .map_err(|err| format!("Failed to load secret store: {err}"))?;

    let builder_init: StartDeviceAuthorizationResponse = auth::builder_id::start_device_authorization(&secret_store)
        .await
        .map_err(|err| format!("Failed to init auth: {err}"))?;

    let uuid = uuid::Uuid::new_v4().to_string();

    ctx.set(&[BUILDER_ID_DATA_KEY, &uuid], &builder_init).unwrap();

    let response = ServerOriginatedSubMessage::AuthBuilderIdStartDeviceAuthorizationResponse(
        AuthBuilderIdStartDeviceAuthorizationResponse {
            auth_request_id: uuid,
            code: builder_init.user_code,
            url: builder_init.verification_uri_complete,
            expires_in: builder_init.expires_in,
            interval: builder_init.interval,
        },
    );

    Ok(response.into())
}

pub async fn builder_id_poll_create_token(
    AuthBuilderIdPollCreateTokenRequest { auth_request_id }: AuthBuilderIdPollCreateTokenRequest,
    ctx: &impl KVStore,
) -> RequestResult {
    let secret_store = SecretStore::load()
        .await
        .map_err(|err| format!("Failed to load secret store: {err}"))?;

    let builder_init: StartDeviceAuthorizationResponse =
        ctx.get(&[BUILDER_ID_DATA_KEY, &auth_request_id]).unwrap().unwrap();

    let response = match auth::builder_id::poll_create_token(builder_init.device_code, &secret_store).await {
        PollCreateToken::Pending => AuthBuilderIdPollCreateTokenResponse {
            status: PollStatus::Pending.into(),
            error: None,
        },
        PollCreateToken::Complete(_) => AuthBuilderIdPollCreateTokenResponse {
            status: PollStatus::Complete.into(),
            error: None,
        },
        PollCreateToken::Error(err) => AuthBuilderIdPollCreateTokenResponse {
            status: PollStatus::Error.into(),
            error: Some(err.to_string()),
        },
    };

    Ok(ServerOriginatedSubMessage::AuthBuilderIdPollCreateTokenResponse(response).into())
}
