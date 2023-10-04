use std::borrow::Cow;
use std::path::Path;

use http::header::{
    ACCESS_CONTROL_ALLOW_ORIGIN,
    CONTENT_TYPE,
};
use http::{
    Request,
    Response,
    StatusCode,
};

use super::util::{
    res_400,
    res_404,
    res_500,
};

fn relativize(path: &Path) -> &Path {
    match path.strip_prefix("/") {
        Ok(path) => path,
        Err(_) => path,
    }
}

pub trait Scope {
    const PATH: &'static str;
}

pub struct Dashboard;

impl Scope for Dashboard {
    const PATH: &'static str = "dashboard";
}

// handle `resource://(dir.)?localhost/`
pub async fn handle<S: Scope>(request: Request<Vec<u8>>) -> anyhow::Result<Response<Cow<'static, [u8]>>> {
    let resources_path = fig_util::directories::resources_path()?;

    // If there is a subdomain, prefix the asset path with it
    let mut path = resources_path.clone();
    path.push(S::PATH);
    path.push(relativize(Path::new(request.uri().path())));
    path = path.canonicalize()?;

    // dont allow escaping the resources dir
    if !path.starts_with(resources_path) {
        return Ok(res_400());
    }

    let metadata = match path.metadata() {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(res_404());
        },
        Err(err) => return res_500(err),
    };

    if metadata.is_dir() {
        path.push("index.html");
    }

    let content = match std::fs::read(&path) {
        Ok(content) => content,
        Err(err) => return res_500(err),
    };

    let ext = path.extension().and_then(|ext| ext.to_str());
    let mime = match ext {
        Some("html") => mime::TEXT_HTML_UTF_8.as_ref(),
        Some("css") => mime::TEXT_CSS_UTF_8.as_ref(),
        Some("js") => mime::APPLICATION_JAVASCRIPT_UTF_8.as_ref(),
        Some("json") => mime::APPLICATION_JSON.as_ref(),
        Some("svg") => mime::IMAGE_SVG.as_ref(),
        Some("png") => mime::IMAGE_PNG.as_ref(),
        Some("jpg" | "jpeg") => mime::IMAGE_JPEG.as_ref(),
        Some("woff2") => mime::FONT_WOFF2.as_ref(),
        Some("woff") => mime::FONT_WOFF.as_ref(),
        Some("text") => mime::TEXT_PLAIN.as_ref(),
        _ => match infer::get(&content) {
            Some(mime) => mime.mime_type(),
            // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
            None => mime::APPLICATION_OCTET_STREAM.as_ref(),
        },
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, mime)
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(content.into())?)
}
