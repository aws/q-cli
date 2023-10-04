use std::borrow::Cow;
use std::path::Path;

use http::header::{
    ACCESS_CONTROL_ALLOW_ORIGIN,
    CONTENT_TYPE,
};
use http::uri::Scheme;
use http::{
    Request,
    Response,
    StatusCode,
};
use serde::{
    Deserialize,
    Serialize,
};
use tracing::error;

use crate::protocol::util::res_404;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AssetMetadata {
    status: u16,
    headers: Vec<(String, String)>,
}

fn transform_path(path: &Path) -> Cow<Path> {
    // strip the leading slash
    let path = match path.strip_prefix("/") {
        Ok(path) => path,
        Err(_) => path,
    };

    // if the path is empty or ends with a slash, it's a directory and we append index.html
    if path.as_os_str().is_empty() || path.ends_with("/") {
        path.join("index.html").into()
    } else {
        path.into()
    }
}

pub async fn handle(request: Request<Vec<u8>>) -> anyhow::Result<Response<Cow<'static, [u8]>>> {
    let mut url_parts = request.uri().clone().into_parts();
    url_parts.scheme = Some(Scheme::HTTPS);

    // The authority schema for figapp:// is <subdomain>.localhost
    let app = match url_parts.authority {
        Some(authority) => match authority.as_str().rsplit_once('.') {
            Some((subdomain, "localhost")) => {
                url_parts.authority = Some(format!("{subdomain}.fig.io").parse().unwrap());
                subdomain.to_owned()
            },
            _ => {
                error!("Invalid authority: {authority}");
                return Ok(res_404());
            },
        },
        None => {
            error!("No authority in request");
            return Ok(res_404());
        },
    };

    let url = http::Uri::from_parts(url_parts).unwrap();

    let mut resp_builder = Response::builder().status(StatusCode::OK);
    resp_builder = resp_builder.header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");

    let path = Path::new(url.path());
    let path = transform_path(path);
    let path = match app.as_str() {
        _ if app.ends_with("autocomplete") => Path::new("../autocomplete-engine/app/dist").join(path),
        _ => {
            return Ok(res_404());
        },
    };

    let data = tokio::fs::read(&path).await?;

    let mime = match infer::get(&data) {
        Some(mime) => mime.to_string(),
        None => match path.extension().and_then(|ext| ext.to_str()) {
            Some("css") => mime::TEXT_CSS,
            Some("html") => mime::TEXT_HTML,
            Some("js") => mime::APPLICATION_JAVASCRIPT,
            Some("json") => mime::APPLICATION_JSON,
            Some("svg") => mime::IMAGE_SVG,
            Some("png") => mime::IMAGE_PNG,
            Some("jpg" | "jpeg") => mime::IMAGE_JPEG,
            Some("gif") => mime::IMAGE_GIF,
            Some("woff") => mime::FONT_WOFF,
            Some("woff2") => mime::FONT_WOFF2,
            Some(_) => mime::TEXT_PLAIN,
            // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
            None => mime::APPLICATION_OCTET_STREAM,
        }
        .to_string(),
    };
    resp_builder = resp_builder.header(CONTENT_TYPE, mime);

    // Finish building the response with the contents
    Ok(resp_builder.body(data.into())?)
}
