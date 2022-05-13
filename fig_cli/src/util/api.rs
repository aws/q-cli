use std::fmt::Display;

use anyhow::{
    bail,
    Result,
};
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
        let token = get_token().await?;
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
                    Some(err) => bail!(err),
                    None => bail!("Unknown error"),
                }
            }};
        }

        match resp.text().await {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(json) => {
                    // println!("{}", json);
                    bail!(
                        json.get("error")
                            .and_then(|error| error.as_str())
                            .unwrap_or("Unknown error")
                            .to_string()
                    )
                },
                Err(_) => {
                    if !text.is_empty() {
                        bail!(text)
                    } else {
                        print_err!()
                    }
                },
            },
            Err(_) => print_err!(),
        }
    }
}
