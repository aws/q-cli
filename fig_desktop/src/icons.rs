use std::borrow::Cow;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::Cursor;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use image::imageops::FilterType;
use image::{
    ImageOutputFormat,
    Rgba,
};
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

use crate::platform::PlatformState;

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

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
static ASSET_CACHE: Lazy<Cache<PathBuf, ProcessedAsset>> =
    Lazy::new(|| Cache::builder().time_to_live(Duration::from_secs(120)).build());

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(Debug, Clone)]
pub enum AssetKind {
    Png,
    Svg,
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn process_asset(path: PathBuf) -> Result<ProcessedAsset> {
    if let Some(asset) = ASSET_CACHE.get(&path) {
        trace!("icon cache hit");
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
    PlatformState::icon_lookup(name).unwrap_or_else(|| {
        (
            ASSETS
                .get(name)
                .unwrap_or_else(|| ASSETS.get("template").unwrap())
                .clone(),
            AssetKind::Png, // bundled assets are PNGs
        )
    })
}

fn build_asset_response(data: Vec<u8>, asset_kind: AssetKind) -> Response {
    ResponseBuilder::new()
        .status(StatusCode::OK)
        .mimetype(match asset_kind {
            AssetKind::Png => "image/png",
            AssetKind::Svg => "image/svg+xml",
        })
        .header("Access-Control-Allow-Origin", "*")
        .body(data)
        .unwrap()
}

fn cached_asset_response(name: &str) -> Response {
    trace!("building response for asset {name}");
    let (data, asset_kind) = resolve_asset(name);
    build_asset_response(data.to_vec(), asset_kind)
}

fn build_default() -> Response {
    cached_asset_response("template")
}

fn scale(a: u8, b: u8) -> u8 {
    (a as f32 * (b as f32 / 256.0)) as u8
}

pub fn handle(request: &Request) -> anyhow::Result<Response> {
    debug!(uri = request.uri(), "Fig protocol request");
    let url = Url::parse(request.uri())?;
    let domain = url.domain();
    // rust really doesn't like us not specifying RandomState here
    let pairs: HashMap<_, _, RandomState> = HashMap::from_iter(url.query_pairs());

    Ok(match domain {
        Some("template") => {
            let query_pairs: HashMap<Cow<str>, Cow<str>> = url.query_pairs().collect();

            let mut image =
                image::load_from_memory_with_format(ASSETS.get("template").unwrap(), image::ImageFormat::Png).unwrap();

            if let Some(color) = query_pairs.get("color") {
                if color.len() == 6 {
                    if let (Ok(color_r), Ok(color_g), Ok(color_b)) = (
                        u8::from_str_radix(&color[0..2], 16),
                        u8::from_str_radix(&color[2..4], 16),
                        u8::from_str_radix(&color[4..6], 16),
                    ) {
                        imageproc::map::map_colors_mut(&mut image, |Rgba([r, g, b, a])| {
                            Rgba([scale(r, color_r), scale(g, color_g), scale(b, color_b), a])
                        });
                    }
                }
            }

            // todo: add baged
            // if let Some(badge) = query_pairs.get("badge") {}

            let mut png_bytes = std::io::Cursor::new(Vec::new());
            image.write_to(&mut png_bytes, image::ImageFormat::Png).unwrap();
            Some(build_asset_response(png_bytes.into_inner(), AssetKind::Png))
        },
        Some("icon") | Some("asset") => pairs
            .get("asset")
            .or_else(|| pairs.get("type"))
            .map(|name| cached_asset_response(name)),
        None => {
            let decoded_str = &*percent_decode_str(url.path()).decode_utf8()?;
            let path: Cow<Path> = Cow::from(Path::new(decoded_str));

            #[cfg(windows)]
            let path = transform_unix_to_windows_path(path);

            match fs::metadata(&path) {
                Ok(meta) => {
                    if meta.is_dir() {
                        Some(cached_asset_response("folder"))
                    } else if meta.is_file() {
                        Some(cached_asset_response("file"))
                    } else if meta.is_symlink() {
                        Some(cached_asset_response("symlink"))
                    } else {
                        None
                    }
                },
                Err(_) => Some(match path.to_string_lossy().ends_with('/') {
                    true => cached_asset_response("folder"),
                    false => cached_asset_response("file"),
                }),
            }
        },
        _ => None,
    }
    .unwrap_or_else(build_default))
}

/// Translate a unix style path into a windows style path assuming root dir is the drive
#[cfg_attr(not(windows), allow(dead_code))]
fn transform_unix_to_windows_path(path: Cow<'_, Path>) -> Cow<'_, Path> {
    use std::path::Component;

    let string_path = path.as_ref().to_string_lossy();
    let folder = string_path.ends_with('/') || string_path.ends_with('\\');

    let path_components: Vec<_> = path.as_ref().components().collect();
    match &path_components[..] {
        [Component::RootDir, Component::Normal(drive), rest @ ..] => {
            let mut root = std::ffi::OsString::from(drive);
            root.push(":");

            for component in rest {
                root.push("\\");
                root.push(component.as_os_str());
            }

            if folder {
                root.push("/");
            }

            Cow::from(PathBuf::from(root))
        },
        _ => path,
    }
}
