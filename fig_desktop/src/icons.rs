use std::borrow::Cow;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::hash::Hash;
use std::io::Cursor;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use http::header::CONTENT_TYPE;
use http::{
    Request,
    Response,
    StatusCode,
};
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
    warn,
};
use url::Url;

use crate::platform::PlatformState;

const DEFAULT_ICON: &str = "template";

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum AssetSpecifier<'a> {
    Named(Cow<'a, str>),
    PathBased(Cow<'a, Path>),
}

static ASSETS: Lazy<HashMap<AssetSpecifier<'static>, Arc<Vec<u8>>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    macro_rules! load_assets {
        ($($name: expr),*) => {
            $(
                let mut vec = Vec::new();
                vec.extend_from_slice(include_bytes!(concat!(env!("AUTOCOMPLETE_ICONS_PROCESSED"), "/", $name, ".png")));
                map.insert(
                    AssetSpecifier::Named($name.into()),
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

fn resolve_asset(asset: &AssetSpecifier, fallback: Option<&str>) -> (Arc<Vec<u8>>, AssetKind) {
    match &asset {
        AssetSpecifier::Named(_) => ASSETS
            .get(asset)
            .map(|asset| (asset.clone(), AssetKind::Png))
            .or_else(|| PlatformState::icon_lookup(asset)),
        AssetSpecifier::PathBased(_) => PlatformState::icon_lookup(asset),
    }
    .or_else(|| match fallback {
        Some(fallback) => ASSETS
            .get(&AssetSpecifier::Named(fallback.into()))
            .map(|asset| (asset.clone(), AssetKind::Png)),
        None => None,
    })
    .unwrap_or_else(|| {
        ASSETS
            .get(&AssetSpecifier::Named(DEFAULT_ICON.into()))
            .map(|asset| (asset.clone(), AssetKind::Png))
            .unwrap()
    })
}

fn build_asset_response(data: Vec<u8>, asset_kind: AssetKind) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, match asset_kind {
            AssetKind::Png => "image/png",
            AssetKind::Svg => "image/svg+xml",
        })
        .header("Access-Control-Allow-Origin", "*")
        .body(data)
        .unwrap()
}

fn cached_asset_response(asset: &AssetSpecifier, fallback: Option<&str>) -> Response<Vec<u8>> {
    trace!("building response for asset {asset:?}");
    let (data, asset_kind) = resolve_asset(asset, fallback);
    build_asset_response(data.to_vec(), asset_kind)
}

fn build_default() -> Response<Vec<u8>> {
    cached_asset_response(&AssetSpecifier::Named(DEFAULT_ICON.into()), None)
}

fn scale(a: u8, b: u8) -> u8 {
    (a as f32 * (b as f32 / 256.0)) as u8
}

pub fn handle(request: &Request<Vec<u8>>) -> anyhow::Result<Response<Vec<u8>>> {
    debug!(uri =% request.uri(), "Fig protocol request");
    let url = Url::parse(&request.uri().to_string())?;
    let domain = url.domain();
    // rust really doesn't like us not specifying RandomState here
    let pairs: HashMap<_, _, RandomState> = HashMap::from_iter(url.query_pairs());

    Ok(match domain {
        Some("template") => {
            let query_pairs: HashMap<Cow<str>, Cow<str>> = url.query_pairs().collect();

            let asset = ASSETS.get(&AssetSpecifier::Named("template".into())).unwrap();
            let mut image = image::load_from_memory_with_format(asset, image::ImageFormat::Png).unwrap();

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

            let mut png_bytes = std::io::Cursor::new(Vec::new());
            image.write_to(&mut png_bytes, image::ImageFormat::Png).unwrap();
            Some(build_asset_response(png_bytes.into_inner(), AssetKind::Png))
        },
        Some("icon") | Some("asset") => pairs
            .get("asset")
            .or_else(|| pairs.get("type"))
            .map(|name| cached_asset_response(&AssetSpecifier::Named(Cow::Borrowed(name)), None)),
        Some("path") => {
            let decoded_str = &*percent_decode_str(url.path()).decode_utf8().map_err(|err| {
                warn!(%err, "Failed to decode fig url");
                err
            })?;

            cfg_if::cfg_if! {
                if #[cfg(windows)] {
                    let path = transform_unix_to_windows_path(path);
                    // TODO: we might want to shellexpand like we do below, but we need
                    // context on what the home dir is
                    let path_str = path.to_str().unwrap_or("");
                } else {
                    let path_str = shellexpand::tilde(&decoded_str);
                    let path = Path::new(path_str.as_ref());
                }
            }

            let fallback = match fs::metadata(path) {
                Ok(meta) => {
                    if meta.is_dir() {
                        Some("folder")
                    } else if meta.is_file() {
                        Some("file")
                    } else if meta.is_symlink() {
                        Some("symlink")
                    } else {
                        warn!(%path_str, "Unknown file type");
                        None
                    }
                },
                Err(err) => {
                    warn!(%path_str, %err, "Failed to stat path");
                    Some(if path_str.ends_with('/') { "folder" } else { "file" })
                },
            };

            Some(cached_asset_response(
                &AssetSpecifier::PathBased(Cow::Borrowed(path)),
                fallback,
            ))
        },
        _ => None,
    }
    .unwrap_or_else(build_default))
}

/// Translate a unix style path into a windows style path assuming root dir is the drive
#[cfg_attr(not(windows), allow(dead_code))]
fn transform_unix_to_windows_path(path: Cow<'_, str>) -> Cow<'_, Path> {
    use std::path::Component;

    let folder = path.ends_with('/') || path.ends_with('\\');

    let path_components: Vec<_> = Path::new(path.as_ref()).components().collect();
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
        _ => match path {
            Cow::Borrowed(path) => Cow::Borrowed(Path::new(path)),
            Cow::Owned(path) => Cow::Owned(PathBuf::from(path)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_to_windows_path_transform() {
        assert_eq!(
            transform_unix_to_windows_path("/c/User/chay/Downloads".into()),
            Path::new(r"c:\User\chay\Downloads")
        );
        assert_eq!(transform_unix_to_windows_path("/".into()), Path::new("/"));
    }
}
