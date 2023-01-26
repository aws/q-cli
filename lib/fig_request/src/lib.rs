pub mod auth;
pub mod defaults;
mod error;
pub mod reqwest_client;

use std::fmt::Display;
use std::time::Duration;

use auth::get_token;
pub use error::Error;
use fig_settings::api::{
    host,
    release_host,
};
pub use reqwest;
use reqwest::cookie::Cookie;
use reqwest::header::{
    HeaderMap,
    HeaderName,
    HeaderValue,
};
pub use reqwest::Method;
use reqwest::{
    Client,
    RequestBuilder,
    StatusCode,
    Url,
};
use serde::de::DeserializeOwned;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use tracing::debug;

pub fn client() -> Option<&'static Client> {
    reqwest_client::reqwest_client(true)
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait Auth {}

pub struct AddAuth;
impl Auth for AddAuth {}

pub struct MaybeAuth;
impl Auth for MaybeAuth {}

pub struct NoAuth;
impl Auth for NoAuth {}

pub struct Request<A: Auth> {
    builder: Option<RequestBuilder>,
    _auth: A,
}

impl Request<NoAuth> {
    pub fn new(method: Method, endpoint: impl AsRef<str> + Display) -> Self {
        debug!(%method, %endpoint);
        Self::new_with_host(host(), method, endpoint)
    }

    pub fn new_with_url(method: Method, url: reqwest::Url) -> Self {
        Self {
            builder: client()
                .as_ref()
                .map(|client| client.request(method, url).header("Accept", "application/json")),
            _auth: NoAuth,
        }
    }

    pub fn new_with_host(mut host: Url, method: Method, endpoint: impl AsRef<str>) -> Self {
        host.set_path(endpoint.as_ref());

        Self {
            builder: client()
                .as_ref()
                .map(|client| client.request(method, host).header("Accept", "application/json")),
            _auth: NoAuth,
        }
    }

    pub fn new_release(method: Method, endpoint: impl AsRef<str> + Display) -> Self {
        Self::new_with_host(release_host(), method, endpoint)
    }

    pub fn get(endpoint: impl AsRef<str> + Display) -> Self {
        Self::new(Method::GET, endpoint)
    }

    pub fn post(endpoint: impl AsRef<str> + Display) -> Self {
        Self::new(Method::POST, endpoint)
    }

    pub fn delete(endpoint: impl AsRef<str> + Display) -> Self {
        Self::new(Method::DELETE, endpoint)
    }

    pub async fn send(self) -> Result<Response> {
        match self.builder {
            Some(builder) => Ok(Response {
                inner: builder.send().await?,
            }),
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

    /// Body text (parses fig errors)
    pub async fn text(self) -> Result<String> {
        let response = self.send().await?;
        let text = response.handle_fig_response().await?.text().await?;
        Ok(text)
    }

    /// Raw text (does not parse fig errors)
    pub async fn raw_text(self) -> Result<String> {
        let response = self.send().await?;
        let text = response.inner.text().await?;
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
        match response.json::<graphql_client::Response<T>>().await {
            Ok(response) => {
                if let Some(errors) = response.errors {
                    Err(Error::Graphql(errors))
                } else {
                    Ok(response.data.unwrap())
                }
            },
            Err(err) => Err(err.into()),
        }
    }

    pub async fn response(self) -> Result<Response> {
        self.send().await
    }
}

impl Request<AddAuth> {
    pub async fn send(self) -> Result<Response> {
        match self.builder {
            Some(builder) => {
                let token = get_token().await?;
                let builder = builder.bearer_auth(token);
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

    /// Body text (parses fig errors)
    pub async fn text(self) -> Result<String> {
        let response = self.send().await?;
        let text = response.handle_fig_response().await?.text().await?;
        Ok(text)
    }

    /// Raw text (does not parse fig errors)
    pub async fn raw_text(self) -> Result<String> {
        let response = self.send().await?;
        let text = response.inner.text().await?;
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
        match response.json::<graphql_client::Response<T>>().await {
            Ok(response) => match response.errors {
                Some(errors) => Err(Error::Graphql(errors)),
                None => response.data.ok_or(Error::GraphqlNoData),
            },
            Err(err) => Err(err.into()),
        }
    }

    pub async fn response(self) -> Result<Response> {
        self.send().await
    }
}

impl Request<MaybeAuth> {
    pub async fn send(self) -> Result<Response> {
        match self.builder {
            Some(builder) => {
                let token = get_token().await.ok();
                let builder = match token {
                    Some(token) => builder.bearer_auth(token),
                    None => builder,
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

    /// Body text (parses fig errors)
    pub async fn text(self) -> Result<String> {
        let response = self.send().await?;
        let text = response.handle_fig_response().await?.text().await?;
        Ok(text)
    }

    /// Raw text (does not parse fig errors)
    pub async fn raw_text(self) -> Result<String> {
        let response = self.send().await?;
        let text = response.inner.text().await?;
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
        match response.json::<graphql_client::Response<T>>().await {
            Ok(response) => {
                if let Some(errors) = response.errors {
                    Err(Error::Graphql(errors))
                } else {
                    Ok(response.data.unwrap())
                }
            },
            Err(err) => Err(err.into()),
        }
    }

    pub async fn response(self) -> Result<Response> {
        self.send().await
    }
}

impl<A: Auth> Request<A> {
    pub fn body_json(self, body: impl Serialize) -> Self {
        Self {
            builder: self.builder.map(|builder| builder.json(&body)),
            ..self
        }
    }

    pub fn body<T: Into<reqwest::Body>>(self, body: T) -> Self {
        Self {
            builder: self.builder.map(|builder| builder.body(body)),
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
    pub fn auth(self) -> Request<AddAuth> {
        Request {
            builder: self.builder,
            _auth: AddAuth,
        }
    }

    pub fn maybe_auth(self) -> Request<MaybeAuth> {
        Request {
            builder: self.builder,
            _auth: MaybeAuth,
        }
    }

    /// Adds namespace to request. Pass `None` to use the user namespace
    pub fn namespace(self, namespace: Option<impl AsRef<str>>) -> Self {
        if let Some(namespace) = namespace {
            return self.query(&[("namespace", namespace.as_ref())]);
        }
        self
    }

    /// Add a header to the request
    pub fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        Self {
            builder: self.builder.map(|builder| builder.header(key, value)),
            ..self
        }
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        Self {
            builder: self.builder.map(|builder| builder.timeout(timeout)),
            ..self
        }
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

fn parse_fig_error_response(status: StatusCode, text: String) -> Error {
    match serde_json::from_str::<ErrorResponse>(&text) {
        Ok(ErrorResponse { error, sentry_id }) => Error::Fig {
            error,
            status,
            sentry_id,
        },
        Err(_) => {
            if !text.is_empty() {
                Error::Fig {
                    error: text,
                    status,
                    sentry_id: None,
                }
            } else {
                Error::Status(status)
            }
        },
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    fn mock_value(part: &str) -> Value {
        json! {{
            "part": part,
        }}
    }

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

    #[tokio::test]
    async fn api_get() {
        #[derive(Deserialize)]
        struct Response {
            body: Value,
            query: Value,
        }

        let body = mock_value("body");
        let query = mock_value("query");

        let resp: Response = Request::get("/test/fig_request")
            .body_json(body.clone())
            .query(&query)
            .deser_json()
            .await
            .unwrap();

        assert_eq!(resp.body, body);
        assert_eq!(resp.query, query);
    }
}
