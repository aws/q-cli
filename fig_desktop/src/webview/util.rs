use serde_json::json;
use wry::http::status::StatusCode;
use wry::http::{
    Request as HttpRequest,
    Response as HttpResponse,
    ResponseBuilder as HttpResponseBuilder,
};

pub fn wrap_custom_protocol(
    f: impl Fn(&HttpRequest) -> anyhow::Result<HttpResponse>,
) -> impl Fn(&HttpRequest) -> wry::Result<HttpResponse> {
    move |req: &HttpRequest| -> wry::Result<HttpResponse> {
        Ok(match f(req) {
            Ok(res) => res,
            Err(err) => {
                let response = HttpResponseBuilder::new().status(StatusCode::BAD_REQUEST);
                match req
                    .headers()
                    .get("Accept")
                    .and_then(|accept| accept.to_str().ok())
                    .and_then(|accept| accept.split('/').last())
                {
                    Some("json") => response.mimetype("application/json").body(
                        serde_json::to_vec(&json!({ "error": err.to_string() })).unwrap_or_else(|_| b"{}".to_vec()),
                    ),
                    _ => response.mimetype("text/plain").body(err.to_string().into_bytes()),
                }?
            },
        })
    }
}
