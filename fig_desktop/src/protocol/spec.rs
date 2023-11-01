use std::borrow::Cow;

use http::header::{
    ACCESS_CONTROL_ALLOW_ORIGIN,
    CONTENT_TYPE,
};
use http::{
    HeaderValue,
    Request,
    Response,
    StatusCode,
};

const CDN_URL: &str = "https://d3e7ef0le33nq1.cloudfront.net";

fn res_404() -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, "text/plain")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(b"Not Found".as_ref().into())
        .unwrap()
}

fn res_ok(bytes: Vec<u8>, content_type: HeaderValue) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(bytes.into())
        .unwrap()
}

fn res_reqwest_err(err: fig_request::ReqwestError) -> anyhow::Result<Response<Cow<'static, [u8]>>> {
    let mut builder = Response::builder();
    if let Some(status) = err.status() {
        builder = builder.status(status);
    } else {
        builder = builder.status(StatusCode::INTERNAL_SERVER_ERROR);
    }
    builder = builder
        .header(CONTENT_TYPE, "text/plain")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    Ok(builder.body(err.to_string().as_bytes().to_vec().into())?)
}

// handle `spec://localhost/spec.js`
pub async fn handle(request: Request<Vec<u8>>) -> anyhow::Result<Response<Cow<'static, [u8]>>> {
    let Some(client) = fig_request::client() else {
        return Ok(res_404());
    };

    let path = request.uri().path();

    match client.get(format!("{CDN_URL}{path}")).send().await {
        Ok(response) => {
            if let Err(err) = response.error_for_status_ref() {
                return res_reqwest_err(err);
            }

            let content_type = response
                .headers()
                .get(CONTENT_TYPE)
                .map(|v| v.to_owned())
                .unwrap_or_else(|| "application/javascript".try_into().unwrap());

            Ok(res_ok(response.bytes().await?.to_vec(), content_type))
        },
        Err(err) => res_reqwest_err(err),
    }
}
