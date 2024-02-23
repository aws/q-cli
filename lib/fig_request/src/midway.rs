#![allow(dead_code)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use cookie::CookieBuilder;
use once_cell::sync::Lazy;
use reqwest::cookie::Jar;
use reqwest::redirect::Policy;
use reqwest::{
    Client,
    Method,
    Request,
    StatusCode,
};
use tracing::{
    event,
    Level,
};
use url::Url;

/// Error type for [`MidwayAuthRuntimePlugin`] & [`MidwayAuthHttpClient`]
#[derive(Debug)]
pub struct MidwayError {
    kind: ErrorKind,
}

impl std::error::Error for MidwayError {}

impl fmt::Display for MidwayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::HomeNotSet => {
                write!(
                    f,
                    "the HOME environment variable was not set. Can't locate the Midway cookie."
                )
            },
            ErrorKind::NoCookieFound => {
                write!(f, "no Midway cookie found: run `mwinit -s` to refresh the cookie")
            },
            ErrorKind::FailedToRead(err) => {
                write!(f, "failed to read the Midway cookie (at ~/.midway/cookie): {err}")
            },
            ErrorKind::ParseError(err) => {
                write!(f, "parse error (in ~/.midway/cookie): {err}")
            },
            ErrorKind::StreamingBodyUnsupported => {
                write!(f, "Midway HTTP connector does not support streaming request bodies")
            },
            ErrorKind::NoHostSet => {
                write!(f, "no Host header set for request. This is a bug in smithy-rs.")
            },
            ErrorKind::MidwayError(err) => {
                write!(f, "{err}")
            },
        }
    }
}

impl MidwayError {
    fn maybe_midway_error(b: impl AsRef<[u8]>) -> Result<(), Self> {
        #[derive(Debug, serde::Deserialize)]
        enum UnauthorizedStatus {
            #[serde(rename = "error")]
            Keyword,
        }
        #[derive(Debug, serde::Deserialize)]
        enum BadPostureStatus {
            #[serde(rename = "posture_error")]
            Keyword,
        }
        #[derive(Debug, serde::Deserialize)]
        #[serde(untagged)]
        enum MidwayError {
            // https://w.amazon.com/bin/view/GoAnywhere/Development/ClientInterface/
            // https://code.amazon.com/packages/GoAmzn-CoralMidwayClient/blobs/14e126f77150f8b4a9f0ff6e1621496a0fc3150f/--/src/golang.a2z.com/GoAmzn-CoralMidwayClient/midwayclient/handler.go#L63
            #[allow(dead_code)]
            Unauthorized {
                status: UnauthorizedStatus,
                message: String,
                #[serde(rename = "desc")]
                description: String,
                #[serde(rename = "step_up_methods")]
                step_up: Vec<serde_json::Value>,
            },
            #[allow(dead_code)]
            BadPosture {
                status: BadPostureStatus,
                message: String,
                location: String,
                cookie_presented: bool,
                cookie_verified: bool,
                compliance_valid: bool,
            },
        }
        impl fmt::Display for MidwayError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    MidwayError::Unauthorized {
                        message, description, ..
                    } => {
                        write!(f, "Midway refused access: {message}. {description}")
                    },
                    MidwayError::BadPosture { message, .. } => {
                        write!(f, "Midway authentication invalid: {message}")
                    },
                }
            }
        }

        let b = b.as_ref();
        match serde_json::from_slice::<MidwayError>(b) {
            Ok(err) => Err(Self {
                kind: ErrorKind::MidwayError(err.to_string()),
            }),
            Err(reason) => {
                if let Ok(s) = std::str::from_utf8(b) {
                    event!(
                        Level::DEBUG,
                        "Midway error response has unexpected format:\n{s}\nreason: {reason}",
                    );
                }
                Ok(())
            },
        }
    }
}

impl From<ErrorKind> for MidwayError {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
enum ErrorKind {
    FailedToRead(std::io::Error),
    HomeNotSet,
    MidwayError(String),
    NoCookieFound,
    NoHostSet,
    ParseError(Cow<'static, str>),
    StreamingBodyUnsupported,
}

impl ErrorKind {
    fn parse(message: impl Into<Cow<'static, str>>) -> Self {
        ErrorKind::ParseError(message.into())
    }
}

#[derive(Debug)]
struct MidwayCookies {
    cookies: Vec<(url::Url, cookie::Cookie<'static>)>,
}

impl MidwayCookies {
    fn load() -> Result<Self, MidwayError> {
        let midway_cookie =
            PathBuf::from(std::env::var("HOME").map_err(|_err| ErrorKind::HomeNotSet)?).join(".midway/cookie");
        if !midway_cookie.exists() {
            return Err(ErrorKind::NoCookieFound.into());
        }

        let mut cookies = Vec::new();

        // The midway cookie is stored in ~/.midway/cookie, and is stored in the "Netscape
        // cookiejar format" used by cURL: https://curl.se/docs/http-cookies.html. This format is
        // not used by browsers any more, and is mostly now a quirk of cURL. The format is simple
        // enough that we can parse it ourselves and then inject cookies into reqwest's cookie
        // store.
        let cookie_jar = std::fs::read_to_string(midway_cookie).map_err(ErrorKind::FailedToRead)?;
        const HTTP_ONLY_PREFIX: &str = "#HttpOnly_";
        for line in cookie_jar.lines() {
            let line = line.trim_start();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('#') && !line.starts_with(HTTP_ONLY_PREFIX) {
                continue;
            }
            let mut fields = line.split('\t');
            let domain = fields.next().ok_or_else(|| ErrorKind::parse("cookie domain not set"))?;
            let (domain, http_only) = if let Some(domain) = domain.strip_prefix(HTTP_ONLY_PREFIX) {
                (domain, true)
            } else {
                (domain, false)
            };
            let domain = domain.trim_start_matches('.');
            let include_subdomains = fields
                .next()
                .ok_or_else(|| ErrorKind::parse("cookie domain not set"))
                .and_then(|v| match v {
                    "TRUE" => Ok(true),
                    "FALSE" => Ok(false),
                    _ => Err(ErrorKind::parse(
                        "include subdomains field in midway cookie not TRUE or FALSE",
                    )),
                })?;
            let path = fields.next().ok_or_else(|| ErrorKind::parse("https only not set"))?;
            let https_only = fields
                .next()
                .ok_or_else(|| ErrorKind::parse("midway cookie HTTPS only field not set"))
                .and_then(|v| match v {
                    "TRUE" => Ok(true),
                    "FALSE" => Ok(false),
                    _ => Err(ErrorKind::parse("HTTPS only field in midway cookie not TRUE or FALSE")),
                })?;
            let expires = fields
                .next()
                .ok_or_else(|| ErrorKind::parse("expiry was not set"))
                .and_then(|v| {
                    Ok(std::num::NonZeroI64::new(v.parse().map_err(|_err| {
                        ErrorKind::ParseError("expiry was not a number".into())
                    })?))
                })?;
            let name = fields.next().ok_or_else(|| ErrorKind::parse("cookie name not set"))?;
            let value = fields.next().ok_or_else(|| ErrorKind::parse("cookie value not set"))?;

            let mut cookie = cookie::CookieBuilder::new(name, value)
                .path(path)
                .secure(https_only)
                .http_only(http_only);

            // If the cookie domain field is set does it include subdomains.
            if include_subdomains {
                cookie = cookie.domain(domain);
            }

            match expires {
                None => {},
                Some(ts) => {
                    cookie = cookie.expires(
                        time::OffsetDateTime::from_unix_timestamp(ts.get())
                            .map_err(|_err| ErrorKind::parse("expiry was not a valid Unix timestamp"))?,
                    );
                },
            }
            let cookie = cookie.build().into_owned();

            let url = url::Url::parse(&format!("https://{}{}", domain, cookie.path().unwrap()))
                .map_err(|err| ErrorKind::parse(format!("failed to construct URL for cookie domain: {err}")))?;
            if let Some(cookie::Expiration::DateTime(ts)) = cookie.expires() {
                if ts <= time::OffsetDateTime::now_utc() {
                    // skip expired cookie
                    continue;
                }
            }

            cookies.push((url, cookie));
        }

        Ok(Self { cookies })
    }
}

static JAR: Lazy<Arc<Jar>> = Lazy::new(|| Arc::new(Jar::default()));

static CLIENT: Lazy<Client> = Lazy::new(|| {
    reqwest::ClientBuilder::new()
        .redirect(Policy::custom(|attempt| {
            if attempt.url().domain() == Some("midway-auth.amazon.com")
                || attempt.url().domain() == Some("cloudfrontsigner.ninjas.security.a2z.com")
            {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .cookie_provider(JAR.clone())
        .build()
        .unwrap()
});

pub async fn midway_request(dest_url: Url) -> Result<reqwest::Response, crate::Error> {
    let res = CLIENT.execute(Request::new(Method::GET, dest_url.clone())).await?;

    if res.status() != StatusCode::FORBIDDEN {
        return Ok(res);
    }

    // if the request failed, try to get cookies needed
    let mw_cookies = MidwayCookies::load()?;
    for (url, cookie) in mw_cookies.cookies {
        JAR.add_cookie_str(&cookie.to_string(), &url);
    }

    let url = Url::parse_with_params("https://cloudfrontsigner.ninjas.security.a2z.com/sign", &[(
        "encodedTargetUrl",
        dest_url.to_string(),
    )])?;

    let res = CLIENT.execute(Request::new(Method::GET, url)).await?;

    let redirect_url = res
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Url::parse(s).ok())
        .unwrap_or_else(|| dest_url.clone());

    let pairs: HashMap<_, _> = redirect_url.query_pairs().collect();

    let mut redirect_url_no_params = redirect_url.clone();
    redirect_url_no_params.set_query(None);
    redirect_url_no_params.set_fragment(None);
    redirect_url_no_params.set_path("");

    if let Some(policy) = pairs.get("policy") {
        JAR.add_cookie_str(
            &CookieBuilder::new("CloudFront-Policy", policy.to_string())
                .build()
                .to_string(),
            &redirect_url_no_params,
        );
    }

    if let Some(kpid) = pairs.get("kpid") {
        JAR.add_cookie_str(
            &CookieBuilder::new("CloudFront-Key-Pair-Id", kpid.to_string())
                .build()
                .to_string(),
            &redirect_url_no_params,
        );
    }

    if let Some(exp) = pairs.get("exp") {
        JAR.add_cookie_str(
            &CookieBuilder::new("CloudFront-Expiration", exp.to_string())
                .build()
                .to_string(),
            &redirect_url_no_params,
        );
    }

    if let Some(sig) = pairs.get("sig") {
        JAR.add_cookie_str(
            &CookieBuilder::new("CloudFront-Signature", sig.to_string())
                .build()
                .to_string(),
            &redirect_url_no_params,
        );
    }

    Ok(CLIENT.execute(Request::new(Method::GET, dest_url)).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "no auth"]
    async fn test_midway() {
        let res = midway_request(
            "https://prod.us-east-1.shellspecs.jupiter.ai.aws.dev/index.json"
                .try_into()
                .unwrap(),
        )
        .await;
        println!("{:?}", res);
        println!("text: {}", res.unwrap().text().await.unwrap());
    }
}
