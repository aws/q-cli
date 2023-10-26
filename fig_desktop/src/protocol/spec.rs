use std::borrow::Cow;

use http::header::{
    ACCESS_CONTROL_ALLOW_ORIGIN,
    CONTENT_TYPE,
};
use http::{
    Request,
    Response,
    StatusCode,
};

fn res_404() -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, "text/plain")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(b"Not Found".as_ref().into())
        .unwrap()
}

fn res_ok(bytes: Vec<u8>) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/javascript")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(bytes.into())
        .unwrap()
}

async fn load_spec(spec_path: String) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
    let path = fig_util::directories::autocomplete_specs_dir()?.join(spec_path);
    if path.exists() {
        let content = tokio::fs::read(path).await?;
        Ok(Some(content.into()))
    } else {
        Ok(None)
    }
}

// handle `spec://localhost/spec.js`
pub async fn handle(request: Request<Vec<u8>>) -> anyhow::Result<Response<Cow<'static, [u8]>>> {
    let Some((_, ext)) = request.uri().path().rsplit_once('.') else {
        return Ok(res_404());
    };

    if ext != "js" {
        return Ok(res_404());
    };

    let spec_path = request.uri().path().trim_start_matches('/').to_owned();

    let Ok(Some(spec_content)) = load_spec(spec_path).await else {
        return Ok(res_404());
    };

    Ok(res_ok(spec_content.into()))
}
