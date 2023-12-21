use auth::builder_id::{
    BuilderIdToken,
    PollCreateToken,
    StartDeviceAuthorizationResponse,
    TokenType,
};
use auth::secret_store::SecretStore;
use fig_proto::fig::auth_builder_id_poll_create_token_response::PollStatus;
use fig_proto::fig::auth_status_response::AuthKind;
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
    let secret_store = SecretStore::new()
        .await
        .map_err(|err| format!("Failed to load secret store: {err}"))?;

    let token = BuilderIdToken::load(&secret_store).await;

    Ok(ServerOriginatedSubMessage::AuthStatusResponse(AuthStatusResponse {
        authed: matches!(token, Ok(Some(_))),
        auth_kind: match token {
            Ok(Some(auth)) => match auth.token_type() {
                TokenType::BuilderId => Some(AuthKind::BuilderId.into()),
                TokenType::IamIdentityCenter => Some(AuthKind::IamIdentityCenter.into()),
            },
            _ => None,
        },
    })
    .into())
}

pub async fn builder_id_start_device_authorization(
    AuthBuilderIdStartDeviceAuthorizationRequest { start_url, region }: AuthBuilderIdStartDeviceAuthorizationRequest,
    ctx: &impl KVStore,
) -> RequestResult {
    if start_url.is_some() != region.is_some() {
        return Err("start_url and region must both be specified or both be omitted".into());
    }

    let secret_store = SecretStore::new()
        .await
        .map_err(|err| format!("Failed to load secret store: {err}"))?;

    let builder_init: StartDeviceAuthorizationResponse =
        auth::builder_id::start_device_authorization(&secret_store, start_url, region)
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
    let secret_store = SecretStore::new()
        .await
        .map_err(|err| format!("Failed to load secret store: {err}"))?;

    let builder_init: StartDeviceAuthorizationResponse =
        ctx.get(&[BUILDER_ID_DATA_KEY, &auth_request_id]).unwrap().unwrap();

    let response = match auth::builder_id::poll_create_token(
        &secret_store,
        builder_init.device_code,
        Some(builder_init.start_url),
        Some(builder_init.region),
    )
    .await
    {
        PollCreateToken::Pending => AuthBuilderIdPollCreateTokenResponse {
            status: PollStatus::Pending.into(),
            error: None,
            error_verbose: None,
        },
        PollCreateToken::Complete(_) => AuthBuilderIdPollCreateTokenResponse {
            status: PollStatus::Complete.into(),
            error: None,
            error_verbose: None,
        },
        PollCreateToken::Error(err) => AuthBuilderIdPollCreateTokenResponse {
            status: PollStatus::Error.into(),
            error: Some(err.to_string()),
            error_verbose: Some(err.to_verbose_string()),
        },
    };

    Ok(ServerOriginatedSubMessage::AuthBuilderIdPollCreateTokenResponse(response).into())
}
