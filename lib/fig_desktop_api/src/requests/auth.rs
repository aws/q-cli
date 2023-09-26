use auth::builder_id::{
    builder_id_token,
    BuilderIdInit,
    BuilderIdPollStatus,
};
use fig_proto::fig::auth_builder_id_poll_response::PollStatus;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    AuthBuilderIdInitRequest,
    AuthBuilderIdInitResponse,
    AuthBuilderIdPollRequest,
    AuthBuilderIdPollResponse,
    AuthStatusRequest,
    AuthStatusResponse,
};

use super::RequestResult;
use crate::kv::KVStore;

const BUILDER_ID_DATA_KEY: &str = "builder_id_data";

pub async fn status(_request: AuthStatusRequest) -> RequestResult {
    Ok(ServerOriginatedSubMessage::AuthStatusResponse(AuthStatusResponse {
        builder_id: builder_id_token().await.is_some(),
    })
    .into())
}

pub async fn builder_id_init(_request: AuthBuilderIdInitRequest, ctx: &impl KVStore) -> RequestResult {
    let builder_init = auth::builder_id::builder_id_init()
        .await
        .map_err(|err| format!("Failed to init auth: {err}"))?;

    let uuid = uuid::Uuid::new_v4().to_string();

    ctx.set(&[BUILDER_ID_DATA_KEY, &uuid], &builder_init).unwrap();

    let response = ServerOriginatedSubMessage::AuthBuilderIdInitResponse(AuthBuilderIdInitResponse {
        auth_request_id: uuid,
        code: builder_init.code,
        url: builder_init.url,
        expires_in: builder_init.expires_in,
        interval: builder_init.interval,
    });

    Ok(response.into())
}

pub async fn builder_id_poll(
    AuthBuilderIdPollRequest { auth_request_id }: AuthBuilderIdPollRequest,
    ctx: &impl KVStore,
) -> RequestResult {
    let builder_init: BuilderIdInit = ctx.get(&[BUILDER_ID_DATA_KEY, &auth_request_id]).unwrap().unwrap();

    let response = match auth::builder_id::builder_id_poll(
        builder_init.device_code,
        builder_init.dynamic_client_id,
        builder_init.dynamic_client_secret,
    )
    .await
    {
        BuilderIdPollStatus::Pending => AuthBuilderIdPollResponse {
            status: PollStatus::Pending.into(),
            error: None,
        },
        BuilderIdPollStatus::Complete => AuthBuilderIdPollResponse {
            status: PollStatus::Complete.into(),
            error: None,
        },
        BuilderIdPollStatus::Error(err) => AuthBuilderIdPollResponse {
            status: PollStatus::Error.into(),
            error: Some(err.to_string()),
        },
    };

    Ok(ServerOriginatedSubMessage::AuthBuilderIdPollResponse(response).into())
}
