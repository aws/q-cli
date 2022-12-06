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

const CDN_PREFIXS: &[&str] = &[
    "https://cdn.jsdelivr.net/npm/@withfig/autocomplete@2/build",
    "https://unpkg.com/@withfig/autocomplete@^2.0.0/build",
    "https://esm.sh/@withfig/autocomplete@^2.0.0/build",
];

fn res_404() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, "text/plain")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(b"Not Found".to_vec())
        .unwrap()
}

fn res_ok(bytes: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/javascript")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(bytes)
        .unwrap()
}

fn cache_dir() -> PathBuf {
    let cache_folder = fig_util::directories::cache_dir().unwrap().join("autocomplete-specs");
    std::fs::create_dir_all(&cache_folder).unwrap();
    cache_folder
}

fn save_cache(spec: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> anyhow::Result<()> {
    let path = cache_dir().join(spec);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)?;
    Ok(())
}

fn load_cache(spec: impl AsRef<Path>) -> anyhow::Result<Option<Vec<u8>>> {
    let path = cache_dir().join(spec);
    if path.exists() {
        let content = std::fs::read(path)?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

pub fn handle(request: &Request<Vec<u8>>) -> anyhow::Result<Response<Vec<u8>>> {
    let Some((_, ext)) = request.uri().path().rsplit_once('.') else {
        return Ok(res_404());
    };

    if ext != "js" {
        return Ok(res_404());
    }

    let file = request.uri().path().trim_start_matches('/').to_owned();

    let handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        handle.block_on(async {
            for cdn in CDN_PREFIXS {
                let url = format!("{cdn}/{file}");

                let body = match fig_request::client().unwrap().get(url).send().await {
                    Ok(res) => {
                        let bytes = res.bytes().await?.to_vec();
                        save_cache(&file, &bytes)?;
                        bytes
                    },
                    Err(_) => {
                        continue;
                    },
                };

                return Ok(res_ok(body));
            }

            // Try to load from cache
            if let Ok(Some(body)) = load_cache(&file) {
                return Ok(res_ok(body));
            }

            Ok(res_404())
        })
    })
    .join()
    .unwrap()
}
