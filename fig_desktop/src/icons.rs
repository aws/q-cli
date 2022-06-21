use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use image::imageops::FilterType;
use image::ImageOutputFormat;
use moka::sync::Cache;
use once_cell::sync::Lazy;
use percent_encoding::percent_decode_str;
use tracing::{
    debug,
    trace,
};
use url::Url;
use wry::http::status::StatusCode;
use wry::http::{
    Request,
    Response,
    ResponseBuilder,
};

use crate::native;

static ASSETS: Lazy<HashMap<&str, Arc<Vec<u8>>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    macro_rules! load_assets {
        ($($name: expr),*) => {
            $(
                let mut vec = Vec::new();
                vec.extend_from_slice(include_bytes!(concat!(env!("AUTOCOMPLETE_ICONS_PROCESSED"), "/", $name, ".png")));
                map.insert(
                    $name,
                    Arc::new(vec),
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

pub type ProcessedAsset = (Arc<Vec<u8>>, AssetKind);

static ASSET_CACHE: Lazy<Cache<PathBuf, ProcessedAsset>> =
    Lazy::new(|| Cache::builder().time_to_live(Duration::from_secs(120)).build());

#[derive(Clone)]
pub enum AssetKind {
    Png,
    Svg,
}

pub fn process_asset(path: PathBuf) -> Result<ProcessedAsset> {
    if let Some(asset) = ASSET_CACHE.get(&path) {
        println!("cache hit");
        return Ok(asset);
    }
    trace!("cache miss processing asset for {path:?}");

    let is_svg = path
        .extension()
        .and_then(OsStr::to_str)
        .map(|ext| ext.to_lowercase() == "svg")
        .unwrap_or(true);

    let built = if is_svg {
        (Arc::new(std::fs::read(&path)?), AssetKind::Svg)
    } else {
        let icon = image::open(&path)?;
        let icon = icon.resize(32, 32, FilterType::CatmullRom);
        let mut cursor = Cursor::new(Vec::new());
        icon.write_to(&mut cursor, ImageOutputFormat::Png)?;
        let buffer = cursor.into_inner();
        (Arc::new(buffer), AssetKind::Png)
    };

    ASSET_CACHE.insert(path, built.clone());

    Ok(built)
}

fn resolve_asset(name: &str) -> (Arc<Vec<u8>>, AssetKind) {
    native::icons::lookup(name).unwrap_or_else(|| {
        (
            ASSETS
                .get(name)
                .unwrap_or_else(|| ASSETS.get("template").unwrap())
                .clone(),
            AssetKind::Png, // bundled assets are PNGs
        )
    })
}

fn build_asset(name: &str) -> Response {
    trace!("building response for asset {}", name);

    let resolved = resolve_asset(name);

    ResponseBuilder::new()
        .status(StatusCode::OK)
        .mimetype(match resolved.1 {
            AssetKind::Png => "image/png",
            AssetKind::Svg => "image/svg+xml",
        })
        .header("Access-Control-Allow-Origin", "*")
        .body(resolved.0.to_vec())
        .unwrap()
}

fn build_default() -> Response {
    build_asset("template")
}

pub fn handle(request: &Request) -> wry::Result<Response> {
    debug!("request for '{}' over fig protocol", request.uri());
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
