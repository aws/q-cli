use std::fmt::Display;

use fig_auth::get_token;
use fig_settings::api_host;
use reqwest::{
    Client,
    Method,
    Response,
    Url,
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Fig(String),
    #[error("Unknown")]
    UnknownFig,
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    Auth(#[from] fig_auth::Error),
}

pub async fn request_json<'a, D, B>(method: Method, endpoint: D, body: B, auth: bool) -> Result<Value>
where
    D: Display,
    B: Into<Option<&'a Value>>,
{
    request(method, endpoint, body, auth).await
}

pub async fn request<'a, T, D, B>(method: Method, endpoint: D, body: B, auth: bool) -> Result<T>
where
    T: DeserializeOwned,
    D: Display,
    B: Into<Option<&'a Value>>,
{
    let api_host = api_host();
    let url = Url::parse(&format!("{api_host}{endpoint}"))?;

    let mut request = Client::new().request(method, url).header("Accept", "application/json");

    if auth {
        let token = match std::env::var("FIG_TOKEN") {
            Ok(token) => token,
            Err(_) => get_token().await?,
        };
        request = request.bearer_auth(token);
    }

    if let Some(body) = body.into() {
        request = request.json(body);
    }

    let response = request.send().await?;
    let json = handle_fig_response(response).await?.json().await?;
    Ok(json)
}

pub async fn handle_fig_response(resp: Response) -> Result<Response> {
    if resp.status().is_success() {
        Ok(resp)
    } else {
        let err = resp.error_for_status_ref().err();
        macro_rules! print_err {
            () => {{
                match err {
                    Some(err) => return Err(err.into()),
                    None => return Err(Error::UnknownFig),
                }
            }};
        }

        match resp.text().await {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(json) => Err(match json.get("error").and_then(|error| error.as_str()) {
                    Some(error) => Error::Fig(error.into()),
                    None => Error::UnknownFig,
                }),
                Err(_) => {
                    if !text.is_empty() {
                        Err(Error::Fig(text))
                    } else {
                        print_err!()
                    }
                },
            },
            Err(_) => print_err!(),
        }
    }
}
