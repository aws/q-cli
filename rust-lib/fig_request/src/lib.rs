use std::time::Duration;

use fig_auth::get_token;
use fig_settings::api_host;
use once_cell::sync::Lazy;
use reqwest::cookie::Cookie;
use reqwest::header::HeaderMap;
pub use reqwest::Method;
use reqwest::{
    Client,
    RequestBuilder,
    StatusCode,
};
use serde::de::DeserializeOwned;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use thiserror::Error;

static CLIENT: Lazy<Option<Client>> = Lazy::new(|| {
    Client::builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .cookie_store(true)
        .timeout(Duration::from_secs(20))
        .build()
        .ok()
});

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{error}")]
    Fig {
        error: String,
        status: StatusCode,
        sentry_id: Option<String>,
    },
    #[error(transparent)]
    Graphql(#[from] GraphqlError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Auth(#[from] fig_auth::Error),
    #[error("Status {0}")]
    Status(StatusCode),
    #[error("No client")]
    NoClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GraphqlError(Vec<serde_json::Map<String, serde_json::Value>>);

impl std::fmt::Display for GraphqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in &self.0 {
            if let Some(message) = error.get("message") {
                match message.as_str() {
                    Some(message) => write!(f, "{message}")?,
                    None => write!(f, "{message}")?,
                }
            } else {
                write!(f, "Unknown error")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for GraphqlError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GraphqlResponse<T> {
    Data(T),
    Errors(GraphqlError),
}

pub struct Request {
    builder: Option<RequestBuilder>,
    auth: bool,
}

impl Request {
    pub fn new(method: Method, endpoint: impl AsRef<str>) -> Self {
        let mut url = api_host();
        url.set_path(endpoint.as_ref());

        Self {
            builder: CLIENT
                .as_ref()
                .map(|client| client.request(method, url).header("Accept", "application/json")),
            auth: false,
        }
    }

    pub fn get(endpoint: impl AsRef<str>) -> Self {
        Self::new(Method::GET, endpoint)
    }

    pub fn post(endpoint: impl AsRef<str>) -> Self {
        Self::new(Method::POST, endpoint)
    }

    pub fn delete(endpoint: impl AsRef<str>) -> Self {
        Self::new(Method::DELETE, endpoint)
    }

    pub fn body(self, body: impl Serialize) -> Self {
        Self {
            builder: self.builder.map(|builder| builder.json(&body)),
            ..self
        }
    }

    pub fn query<Q: Serialize + ?Sized>(self, query: &Q) -> Self {
        Self {
            builder: self.builder.map(|builder| builder.query(query)),
            ..self
        }
    }

    /// Adds fig auth to the request, this can be expensive if the token needs to be
    /// refreshed so only use when needed.
    pub fn auth(self) -> Self {
        Self { auth: true, ..self }
    }

    /// Adds namespace to request. Pass `None` to use the user namespace
    pub fn namespace(self, namespace: Option<impl AsRef<str>>) -> Self {
        if let Some(namespace) = namespace {
            return self.query(&[("namespace", namespace.as_ref())]);
        }
        self
    }

    pub async fn send(self) -> Result<Response> {
        match self.builder {
            Some(builder) => {
                let builder = match self.auth {
                    true => {
                        let token = match std::env::var("FIG_TOKEN") {
                            Ok(token) => token,
                            Err(_) => get_token().await?,
                        };
                        builder.bearer_auth(token)
                    },
                    false => builder,
                };
                Ok(Response {
                    inner: builder.send().await?,
                })
            },
            None => Err(Error::NoClient),
        }
    }

    /// Deserialize json to `T: [DeserializeOwned]`
    pub async fn deser_json<T: DeserializeOwned + ?Sized>(self) -> Result<T> {
        let response = self.send().await?;
        let json = response.handle_fig_response().await?.json().await?;
        Ok(json)
    }

    /// Deserialize json to a [`serde_json::Value`]
    pub async fn json(self) -> Result<Value> {
        self.deser_json().await
    }

    /// Raw body text
    pub async fn text(self) -> Result<String> {
        let response = self.send().await?;
        let text = response.handle_fig_response().await?.text().await?;
        Ok(text)
    }

    /// Raw body bytes
    pub async fn bytes(self) -> Result<bytes::Bytes> {
        let response = self.send().await?;
        let bytes = response.handle_fig_response().await?.bytes().await?;
        Ok(bytes)
    }

    pub async fn graphql<T: DeserializeOwned + ?Sized>(self) -> Result<T> {
        let response = self.send().await?;
        match response.json::<GraphqlResponse<T>>().await {
            Ok(GraphqlResponse::Data(data)) => Ok(data),
            Ok(GraphqlResponse::Errors(err)) => Err(err.into()),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn response(self) -> Result<Response> {
        self.send().await
    }
}

pub struct Response {
    inner: reqwest::Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    error: String,
    sentry_id: Option<String>,
}

impl Response {
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    pub fn cookies(&self) -> impl Iterator<Item = Cookie> {
        self.inner.cookies()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    pub async fn text(self) -> Result<String, reqwest::Error> {
        self.inner.text().await
    }

    pub async fn json<T: DeserializeOwned>(self) -> Result<T, reqwest::Error> {
        self.inner.json().await
    }

    pub async fn bytes(self) -> Result<bytes::Bytes, reqwest::Error> {
        self.inner.bytes().await
    }

    pub async fn handle_fig_response(self) -> Result<Response> {
        if self.inner.status().is_success() {
            Ok(self)
        } else {
            let err = self.inner.error_for_status_ref().err();
            let status = self.inner.status();

            macro_rules! status_err {
                () => {{
                    match err {
                        Some(err) => return Err(err.into()),
                        None => return Err(Error::Status(status)),
                    }
                }};
            }

            match self.inner.text().await {
                Ok(text) => match serde_json::from_str::<ErrorResponse>(&text) {
                    Ok(ErrorResponse { error, sentry_id }) => Err(Error::Fig {
                        error,
                        status,
                        sentry_id,
                    }),
                    Err(_) => {
                        if !text.is_empty() {
                            Err(Error::Fig {
                                error: text,
                                status,
                                sentry_id: None,
                            })
                        } else {
                            status_err!()
                        }
                    },
                },
                Err(_) => status_err!(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn text() {
        let text = Request::get("/health").text().await.unwrap();
        assert_eq!(&text, "OK");
    }

    #[ignore]
    #[tokio::test]
    async fn auth() {
        let value = Request::get("/user/account").auth().json().await.unwrap();
        assert!(value.get("email").is_some());
        assert!(value.get("username").is_some());
    }
}
