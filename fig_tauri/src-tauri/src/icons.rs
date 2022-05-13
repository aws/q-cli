use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::{
    Path,
    PathBuf,
};

use bytes::Bytes;
use once_cell::sync::Lazy;
use percent_encoding::percent_decode_str;
use tauri::http::status::StatusCode;
use tauri::http::{
    Request as HttpRequest,
    Response as HttpResponse,
};
use tauri::{
    AppHandle,
    Runtime,
};
use tracing::{
    debug,
    trace,
};
use url::Url;

static ASSETS: Lazy<HashMap<&str, Bytes>> = Lazy::new(|| {
    let mut map = HashMap::new();

    macro_rules! load_assets {
        ($($name: expr),*) => {
            $(
                map.insert(
                    $name,
                    Bytes::from_static(include_bytes!(concat!(env!("AUTOCOMPLETE_ICONS_PROCESSED"), "/", $name, ".png"))),
                );
            )*
        };
    }

    load_assets! {
        "alert", "android", "apple", "asterisk", "aws", "azure", "box", "carrot", "characters", "command", "commandkey", "commit", "cpu", "database",
        "discord", "docker", "file", "firebase", "folder", "flag", "gcloud", "gear", "git", "github", "gitlab", "gradle", "heroku", "invite", "kubernetes",
        "netlify", "node", "npm", "okteto", "option", "package", "slack", "statusbar", "string", "symlink", "template", "twitter", "vercel", "yarn"
    }

    map
});

trait ResponseWith {
    fn with_status(self, status: StatusCode) -> Self;
    fn with_mimetype(self, mimetype: &'static str) -> Self;
}

impl ResponseWith for HttpResponse {
    fn with_status(mut self, status: StatusCode) -> Self {
        self.set_status(status);
        self
    }

    fn with_mimetype(mut self, mimetype: &'static str) -> Self {
        self.set_mimetype(Some(mimetype.to_string()));
        self
    }
}

fn build_asset(name: &str) -> HttpResponse {
    trace!("building response for asset {}", name);
    HttpResponse::new(
        ASSETS
            .get(name)
            .unwrap_or_else(|| ASSETS.get("template").unwrap())
            .to_vec(),
    )
    .with_mimetype("image/png")
}

fn build_default() -> HttpResponse {
    build_asset("template")
}

pub fn handle<R: Runtime>(_: &AppHandle<R>, request: &HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    debug!("request for fig://{} over fig protocol", request.uri());
    let url = Url::parse(request.uri())?;
    let domain = url.domain();
    // rust really doesn't like us not specifying RandomState here
    let pairs: HashMap<_, _, RandomState> = HashMap::from_iter(url.query_pairs());

    let mut response = None;

    if domain == Some("template") {
        // TODO(mia): implement
    } else if domain == Some("icon") || domain == Some("asset") {
        if let Some(name) = pairs.get("asset").or_else(|| pairs.get("type")) {
            response.replace(build_asset(name));
        }
    } else if domain == None {
        let meta = fs::metadata(&*percent_decode_str(url.path()).decode_utf8_lossy())?;
        if meta.is_dir() {
            response.replace(build_asset("folder"));
        } else if meta.is_file() {
            response.replace(build_asset("file"));
        } else if meta.is_symlink() {
            response.replace(build_asset("symlink"));
        }
    }

    Ok(response.unwrap_or_else(build_default))
}
