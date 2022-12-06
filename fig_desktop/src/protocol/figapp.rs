use std::path::{
    Path,
    PathBuf,
};

use http::header::{
    ACCESS_CONTROL_ALLOW_ORIGIN,
    CONTENT_TYPE,
};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AssetMetadata {
    status: u16,
    headers: Vec<(String, String)>,
}

fn res_404() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, "text/plain")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(b"Not Found".to_vec())
        .unwrap()
}

fn transform_path(path: impl AsRef<Path>) -> PathBuf {
    // strip the leading slash
    let path = match path.as_ref().strip_prefix("/") {
        Ok(path) => path,
        Err(_) => path.as_ref(),
    };

    // if the path is empty or ends with a slash, it's a directory and we append index.html
    if path.as_os_str().is_empty() || path.ends_with("/") {
        path.join("index.html")
    } else {
        path.to_path_buf()
    }
}

fn cache_dir(app: impl AsRef<Path>) -> PathBuf {
    let cache_folder = fig_util::directories::cache_dir().unwrap().join("app").join(app);
    std::fs::create_dir_all(&cache_folder).unwrap();
    cache_folder
}

fn save_cache(
    app: impl AsRef<Path>,
    path: impl AsRef<Path>,
    meta: &AssetMetadata,
    contents: impl AsRef<[u8]>,
) -> anyhow::Result<()> {
    let path = cache_dir(app).join(transform_path(path));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, contents)?;

    let meta_path = match path.extension() {
        Some(ext) => {
            let mut ext = ext.to_owned();
            ext.push(".meta");
            path.with_extension(ext)
        },
        None => path.with_extension("meta"),
    };

    let meta = serde_json::to_string(meta)?;
    std::fs::write(meta_path, meta)?;

    Ok(())
}

fn load_cache(app: impl AsRef<Path>, path: impl AsRef<Path>) -> anyhow::Result<(AssetMetadata, Vec<u8>)> {
    let path = cache_dir(app).join(transform_path(path));
    let content = std::fs::read(&path)?;

    let meta_path = match path.extension() {
        Some(ext) => {
            let mut ext = ext.to_owned();
            ext.push(".meta");
            path.with_extension(ext)
        },
        None => path.with_extension("meta"),
    };

    let meta = std::fs::read_to_string(meta_path)?;
    let meta = serde_json::from_str(&meta)?;

    Ok((meta, content))
}

fn res_cache(app: impl AsRef<Path>, path: impl AsRef<Path>) -> Response<Vec<u8>> {
    match load_cache(&app, path) {
        Ok((meta, content)) => {
            let mut resp_builder = Response::builder().status(meta.status);
            for (key, value) in meta.headers {
                resp_builder = resp_builder.header(key, value);
            }
            resp_builder = resp_builder.header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");
            resp_builder.body(content).unwrap()
        },
        Err(err) => {
            error!(%err, "Error loading cache");
            res_404()
        },
    }
}

pub fn handle(request: &Request<Vec<u8>>) -> anyhow::Result<Response<Vec<u8>>> {
    let mut url_parts = request.uri().clone().into_parts();
    url_parts.scheme = Some("https".parse().unwrap());

    // The authority schema for figapp:// is <subdomain>.localhost
    let app = match url_parts.authority {
        Some(authority) => match authority.as_str().rsplit_once('.') {
            Some((subdomain, "localhost")) => {
                // Some(format!("{subdomain}.fig.io").parse().unwrap())
                // TODO: remove this when branch is merged
                url_parts.authority = Some("autocomplete-6eickbqei-withfig.vercel.app".parse().unwrap());
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

    // check if reachable
    #[cfg(target_os = "macos")]
    if !crate::webview::reachable(url.authority().unwrap().as_str().to_owned()) {
        return Ok(res_cache(&app, url.path()));
    }

    let handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        handle.block_on(async {
            let res = match fig_request::client().unwrap().get(&url.to_string()).send().await {
                Ok(res) => res,
                Err(err) => {
                    error!(%err, "Error fetching figapp");

                    // Try to load from cache
                    let path = url.path();
                    return Ok(res_cache(&app, path));
                },
            };

            // Start building the response
            let mut resp_builder = Response::builder().status(res.status());
            for (key, value) in res.headers().iter() {
                resp_builder = resp_builder.header(key.clone(), value.clone());
            }
            resp_builder = resp_builder.header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");

            // Save meta and contents to cache
            let meta = AssetMetadata {
                status: res.status().as_u16(),
                headers: res
                    .headers()
                    .iter()
                    .map(|(key, value)| (key.to_string(), value.to_str().unwrap().to_string()))
                    .collect(),
            };

            let contents = res.bytes().await.unwrap().to_vec();

            let path = url.path();
            if let Err(err) = save_cache(&app, path, &meta, contents.clone()) {
                error!(%err, %app, %path, "Error saving cache");
            }

            // Finish building the response with the contents
            Ok(resp_builder.body(contents)?)
        })
    })
    .join()
    .unwrap()
}
