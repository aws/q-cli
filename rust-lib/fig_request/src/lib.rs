use fig_auth::get_token;
use fig_settings::api_host;
use once_cell::sync::Lazy;
pub use reqwest::Method;
use reqwest::{
    Client,
    RequestBuilder,
    Response,
};
use serde::de::DeserializeOwned;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use thiserror::Error;

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .https_only(true)
        .build()
        .unwrap()
});

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Fig(String),
    #[error(transparent)]
    Graphql(#[from] GraphqlError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Auth(#[from] fig_auth::Error),
    #[error("Unknown")]
    Unknown,
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
    builder: RequestBuilder,
    auth: bool,
}

impl Request {
    pub fn new(method: Method, endpoint: impl AsRef<str>) -> Self {
        let mut url = api_host();
        url.set_path(endpoint.as_ref());

        Self {
            builder: CLIENT.request(method, url).header("Accept", "application/json"),
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
            builder: self.builder.json(&body),
            ..self
        }
    }

    pub fn query<Q: Serialize + ?Sized>(self, query: &Q) -> Self {
        Self {
            builder: self.builder.query(query),
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
        let builder = match self.auth {
            true => {
                let token = match std::env::var("FIG_TOKEN") {
                    Ok(token) => token,
                    Err(_) => get_token().await?,
                };
                self.builder.bearer_auth(token)
            },
            false => self.builder,
        };
        Ok(builder.send().await?)
    }

    /// Deserialize json to `T: [DeserializeOwned]`
    pub async fn deser_json<T: DeserializeOwned + ?Sized>(self) -> Result<T> {
        let response = self.send().await?;
        let json = handle_fig_response(response).await?.json().await?;
        Ok(json)
    }

    /// Deserialize json to a [`serde_json::Value`]
    pub async fn json(self) -> Result<Value> {
        self.deser_json().await
    }

    /// Raw body text
    pub async fn text(self) -> Result<String> {
        let response = self.send().await?;
        let text = handle_fig_response(response).await?.text().await?;
        Ok(text)
    }

    /// Raw body bytes
    pub async fn bytes(self) -> Result<bytes::Bytes> {
        let response = self.send().await?;
        let bytes = handle_fig_response(response).await?.bytes().await?;
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
}

pub async fn handle_fig_response(resp: Response) -> Result<Response> {
    if resp.status().is_success() {
        Ok(resp)
    } else {
        let err = resp.error_for_status_ref().err();
        macro_rules! status_err {
            () => {{
                match err {
                    Some(err) => return Err(err.into()),
                    None => return Err(Error::Unknown),
                }
            }};
        }

        match resp.text().await {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(json) => Err(match json.get("error").and_then(|error| error.as_str()) {
                    Some(error) => Error::Fig(error.into()),
                    None => status_err!(),
                }),
                Err(_) => {
                    if !text.is_empty() {
                        Err(Error::Fig(text))
                    } else {
                        status_err!()
                    }
                },
            },
            Err(_) => status_err!(),
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
