use std::borrow::Cow;
use std::sync::Arc;

use bytes::Bytes;
use fig_os_shim::Context as OsContext;
use fig_proto::fig::ClientOriginatedMessage;
use fig_proto::prost::Message;
use fig_settings::{
    Settings,
    State,
};
use wry::http::header::CONTENT_TYPE;
use wry::http::{
    HeaderValue,
    Method,
    Request,
    Response,
    StatusCode,
};

use crate::request::{
    Context,
    EventHandler,
};
use crate::webview::{
    DASH_KV_STORE,
    DEBUG_STATE,
    FIGTERM_STATE,
    GLOBAL_PROXY,
    INTERCEPT_STATE,
    NOTIFICATIONS_STATE,
    WindowId,
};

static APPLICATION_FIG_API: HeaderValue = HeaderValue::from_static("application/fig-api");

pub async fn handle(
    _ctx: Arc<OsContext>,
    request: Request<Vec<u8>>,
    window_id: WindowId,
) -> anyhow::Result<Response<Cow<'static, [u8]>>> {
    if request.method() != Method::POST {
        anyhow::bail!("Unsupported method: {}", request.method());
    }

    match request.headers().get(CONTENT_TYPE) {
        Some(val) if val == APPLICATION_FIG_API => {},
        Some(val) => anyhow::bail!("Unsupported content type: {val:?}"),
        None => anyhow::bail!("Missing content type"),
    };

    let client_message = ClientOriginatedMessage::decode(Bytes::from(request.into_body()))?;
    let server_message = fig_desktop_api::handler::api_request(
        EventHandler::default(),
        Context {
            window_id: &window_id,
            debug_state: DEBUG_STATE.get().unwrap().as_ref(),
            figterm_state: FIGTERM_STATE.get().unwrap().as_ref(),
            intercept_state: INTERCEPT_STATE.get().unwrap().as_ref(),
            notifications_state: NOTIFICATIONS_STATE.get().unwrap().as_ref(),
            proxy: GLOBAL_PROXY.get().unwrap(),
            dash_kv_store: DASH_KV_STORE.get().unwrap().as_ref(),
            settings: &Settings::new(),
            state: &State::new(),
            ctx: fig_os_shim::Context::new(),
        },
        client_message,
    )
    .await?;

    let body = server_message.encode_to_vec().into();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, &APPLICATION_FIG_API)
        .body(body)?)
}

// TODO: Add back
//
// #[cfg(test)]
// mod tests {
//     use fig_desktop_api::handler::{
//         ClientOriginatedSubMessage,
//         ServerOriginatedSubMessage,
//     };
//     use fig_proto::fig::{
//         ClientOriginatedMessage,
//         PingRequest,
//         PingResponse,
//         ServerOriginatedMessage,
//     };
//
//     use super::*;
//     use crate::AUTOCOMPLETE_ID;
//
//     #[tokio::test]
//     async fn test_handle() {
//         let body = ClientOriginatedMessage {
//             id: Some(0),
//             submessage: Some(ClientOriginatedSubMessage::PingRequest(PingRequest {})),
//         }
//         .encode_to_vec();
//
//         let request = Request::builder()
//             .method(Method::POST)
//             .header(CONTENT_TYPE, APPLICATION_FIG_API.clone())
//             .body(body)
//             .unwrap();
//
//         let response = handle(request, AUTOCOMPLETE_ID).await.unwrap();
//
//         println!("{:?}", response);
//
//         assert_eq!(response.status(), StatusCode::OK);
//         assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_FIG_API);
//
//         let decoded_response =
// ServerOriginatedMessage::decode(Bytes::from(response.into_body().to_vec())).unwrap();
//         assert_eq!(
//             decoded_response.submessage.unwrap(),
//             ServerOriginatedSubMessage::PingResponse(PingResponse {})
//         );
//     }
// }
