use std::borrow::Cow;
use std::str::FromStr;

use anyhow::Result;
use bytes::Bytes;
use fig_request::reqwest::{
    Client,
    Request as ReqwestRequest,
};
use fig_request::utils::reqwest_response_to_http_response;
use fnv::FnvHashSet;
use futures::prelude::*;
use http::header::{
    ACCESS_CONTROL_ALLOW_ORIGIN,
    CONTENT_TYPE,
};
use http::{
    HeaderValue,
    Method,
    Request,
    Response,
    StatusCode,
};
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize,
};
use url::Url;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthType {
    None,
    Midway,
}

impl AuthType {
    pub async fn get(&self, default_client: &Client, request: ReqwestRequest) -> Result<http::Response<Bytes>> {
        match self {
            AuthType::Midway => fig_request::midway::midway_request(request)
                .await
                .map_err(anyhow::Error::from),
            _ => Ok(
                reqwest_response_to_http_response(default_client.execute(request).await?.error_for_status()?).await?,
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CdnSource {
    url: String,
    auth_type: AuthType,
}

static CDNS: Lazy<Vec<CdnSource>> = Lazy::new(|| {
    vec![
        // Public cdn
        CdnSource {
            url: option_env!("CW_BUILD_SPECS_URL")
                .unwrap_or("https://specs.codewhisperer.us-east-1.amazonaws.com")
                .into(),
            auth_type: AuthType::None,
        },
        // TODO: enable this only for internal users
        // Internal Amazon spec cdn
        // CdnSource {
        //     url: "https://gamma.us-east-1.shellspecs.jupiter.ai.aws.dev".into(),
        //     auth_type: AuthType::Midway,
        // },
    ]
});

fn res_404() -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, "text/plain")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(b"Not Found".as_ref().into())
        .unwrap()
}

fn res_ok(bytes: Vec<u8>, content_type: HeaderValue) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(bytes.into())
        .unwrap()
}

#[derive(Debug, Clone)]
struct SpecIndexMeta {
    cdn_source: CdnSource,
    spec_index: SpecIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecIndex {
    completions: Vec<String>,
    diff_versioned_completions: Vec<String>,
}

async fn remote_index_json(client: &Client) -> &Vec<Result<SpecIndexMeta>> {
    static INDEX_CACHE: tokio::sync::OnceCell<Vec<Result<SpecIndexMeta>>> = tokio::sync::OnceCell::const_new();
    INDEX_CACHE
        .get_or_init(|| async {
            future::join_all(CDNS.iter().map(|cdn_source| async move {
                let url = Url::from_str(&format!("{}/index.json", cdn_source.url)).unwrap();
                let request = ReqwestRequest::new(Method::GET, url);
                let response = cdn_source.auth_type.get(client, request).await?;

                Ok(SpecIndexMeta {
                    cdn_source: cdn_source.clone(),
                    spec_index: serde_json::from_slice(response.body())?,
                })
            }))
            .await
        })
        .await
}

async fn merged_index_json(client: &Client) -> Result<SpecIndex> {
    let mut completions = FnvHashSet::default();
    let mut diff_versioned_completions = FnvHashSet::default();

    for res in remote_index_json(client).await {
        match res {
            Ok(meta) => {
                completions.extend(meta.spec_index.completions.clone());
                diff_versioned_completions.extend(meta.spec_index.diff_versioned_completions.clone());
            },
            Err(err) => {
                tracing::error!(%err, "failed to fetch spec index");
            },
        }
    }

    let mut completions: Vec<_> = completions.into_iter().collect();
    completions.sort();

    let mut diff_versioned_completions: Vec<_> = diff_versioned_completions.into_iter().collect();
    diff_versioned_completions.sort();

    Ok(SpecIndex {
        completions,
        diff_versioned_completions,
    })
}

// handle `spec://localhost/spec.js`
pub async fn handle(request: Request<Vec<u8>>) -> anyhow::Result<Response<Cow<'static, [u8]>>> {
    let Some(client) = fig_request::client() else {
        return Ok(res_404());
    };

    let path = request.uri().path();

    if path == "/index.json" {
        let index = merged_index_json(client).await?;
        Ok(res_ok(
            serde_json::to_vec(&index)?,
            "application/json".try_into().unwrap(),
        ))
    } else {
        // default to trying the first cdn
        let mut cdn_source = &CDNS[0];

        let spec_name = path.strip_prefix('/').unwrap_or(path);
        let spec_name = spec_name.strip_suffix(".js").unwrap_or(spec_name);

        for meta in remote_index_json(client).await.iter().skip(1).flatten() {
            if meta
                .spec_index
                .completions
                .binary_search_by(|name| name.as_str().cmp(spec_name))
                .is_ok()
            {
                cdn_source = &meta.cdn_source;
                break;
            }
        }

        let url = Url::from_str(&format!("{}{path}", cdn_source.url))?;
        let request = ReqwestRequest::new(Method::GET, url);
        let response = cdn_source.auth_type.get(client, request).await?;

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .map_or_else(|| "application/javascript".try_into().unwrap(), |v| v.to_owned());

        Ok(res_ok(response.body().to_vec(), content_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_index_json() {
        let client = Client::new();
        let index = remote_index_json(&client).await;
        println!("{index:?}");
    }
}
