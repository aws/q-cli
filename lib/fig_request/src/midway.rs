use std::borrow::Cow;
use std::fmt;
use std::path::{
    Path,
    PathBuf,
};

use bytes::Bytes;
use http::Response;
use once_cell::sync::Lazy;
use reqwest::header::HeaderValue;
use reqwest::StatusCode;
use tracing::{
    event,
    Level,
};

use crate::utils::reqwest_response_to_http_response;
use crate::Error;

static MIDWAY_ORIGIN: Lazy<url::Origin> =
    Lazy::new(|| url::Url::parse("https://midway-auth.amazon.com").unwrap().origin());

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

pub async fn midway_request(request: reqwest::Request) -> Result<Response<Bytes>, Error> {
    let client = crate::client_no_redirect().expect("midway client_no_redirect is None");

    let mut cookies = MidwayCookies::load()?;

    // Stash the URL origin for later so we can accept redirects for it.
    let original_origin = request.url().origin();
    let mut original_accept = None;
    let mut next_url = request.url().clone();
    if next_url.host().is_none() {
        return Err(MidwayError::from(ErrorKind::NoHostSet).into());
    }

    loop {
        // Leave the original request intact in case we need to retry it.
        let mut req = request
            .try_clone()
            .ok_or_else(|| MidwayError::from(ErrorKind::StreamingBodyUnsupported))?;
        {
            event!(Level::TRACE, "adding cookies");
            let mut cookie_str = req
                .headers_mut()
                .remove(reqwest::header::COOKIE.as_str())
                .map_or_else(Vec::new, |cookie| cookie.to_str().unwrap().as_bytes().to_vec());
            for (cookie_url, cookie) in cookies.cookies.iter() {
                if let Some(cookie::Expiration::DateTime(ts)) = cookie.expires() {
                    if ts <= time::OffsetDateTime::now_utc() {
                        // skip expired cookie
                        event!(
                            Level::TRACE,
                            name = cookie.name(),
                            expires = %ts,
                            "skip expired cookie"
                        );
                        continue;
                    }
                }

                // Does the given cookie apply to the current URL?
                //
                // 1. Does the scheme work out? Secure cookies are only sent over HTTPS.
                if cookie.secure() == Some(true) && next_url.scheme() != "https" {
                    // No. Secure is set, but schema is not https.
                    event!(
                        Level::DEBUG,
                        name = cookie.name(),
                        host = %next_url,
                        "skip secure cookie on insecure domain"
                    );
                    continue;
                }
                // 2. Does the host match?
                if cookie_url.host() == next_url.host() {
                    // Exact host match -- great!
                } else if let Some(domain) = cookie.domain() {
                    // The cookie domain key is set, so (potentially) allow subdomains.
                    let req_host = next_url.host_str().expect("All relevant URLs have hostnames");
                    if let Some(subdomain) = req_host.strip_suffix(domain) {
                        assert!(!subdomain.is_empty());
                        // Request URL ends in cookie domain.
                        // As long as it's actually a subdomain, allow it.
                        if subdomain.ends_with('.') {
                        } else {
                            // Cookie domain is, say, bar.com, and request host is
                            // something like foobar.com. Not a match.
                            event!(
                                Level::TRACE,
                                name = cookie.name(),
                                %domain,
                                host = %req_host,
                                "skip cookie for URL that doesn't quite match domain specifier"
                            );
                            continue;
                        }
                    } else {
                        event!(
                            Level::TRACE,
                            name = cookie.name(),
                            %domain,
                            host = %req_host,
                            "skip cookie for URL that doesn't match domain specifier"
                        );
                        continue;
                    }
                } else {
                    // Only exact host matches are allowed.
                    // And it didn't match.
                    event!(
                        Level::TRACE,
                        name = cookie.name(),
                        domain = ?cookie_url.host_str(),
                        host = %next_url.host().expect("All relevant URLs have hostnames"),
                        "skip cookie for unrelated domain"
                    );
                    continue;
                }

                if let Some(cookie_path) = cookie.path().map(Path::new) {
                    // The cookie only applies to a particular sub-path.
                    // Check that the request is for that sub-path.
                    let url_path = Path::new(next_url.path());
                    if !url_path.starts_with(cookie_path) {
                        event!(
                            Level::DEBUG,
                            name = cookie.name(),
                            url_path = %url_path.display(),
                            cookie_path = %cookie_path.display(),
                            "skip cookie for non-matching path"
                        );
                        continue;
                    }
                }

                event!(Level::DEBUG, name = cookie.name(), "including cookie");

                // We want to include the cookie.
                use std::io::Write;
                if cookie_str.is_empty() {
                    write!(&mut cookie_str, "{}={}", cookie.name(), cookie.value()).unwrap();
                } else {
                    // NOTE: Technically it's possible here that we end up emitting the
                    // same cookie name multiple times, either because it was already set
                    // in the COOKIE header, or because it was set for multiple paths for
                    // the same domain and so doesn't get "merged" earlier. But for Midway,
                    // this is almost certainly good enough.
                    write!(&mut cookie_str, "; {}={}", cookie.name(), cookie.value()).unwrap();
                }
            }

            if !cookie_str.is_empty() {
                let cookie = bytes::Bytes::from(cookie_str);
                let cookie = reqwest::header::HeaderValue::from_maybe_shared(cookie)
                    .expect("cookie is not valid for use in an HTTP header");
                req.headers_mut().insert(reqwest::header::COOKIE, cookie);
            }
        }

        let url = reqwest::Url::parse(next_url.as_str()).expect("A valid http::Uri is a valid url::Url");
        *req.url_mut() = next_url;

        // Midway, for some reason, gets very confused if the Accept header isn't set in a request.
        // Rather than give 401/403 status codes and JSON responses, every response becomes a
        // redirect to /login.html with a 404 status code.
        //
        // So, set that whenever we're talking to midway, and restore the original when we're not.
        if url.domain() == Some("midway-auth.amazon.com") {
            let headers = req.headers_mut();
            let original_accept_header = headers
                .get_all(reqwest::header::ACCEPT)
                .iter()
                .map(|h| h.to_owned())
                .collect::<Vec<_>>();
            if !original_accept_header.is_empty() {
                original_accept = Some(original_accept_header);
            }
            headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("*/*"));
        } else {
            let headers = req.headers_mut();
            headers.remove(reqwest::header::ACCEPT);
            if let Some(hvs) = original_accept.take() {
                for header_value in hvs {
                    headers.append(reqwest::header::ACCEPT, header_value);
                }
            }
        }

        println!("req: {:?}", req);

        let mut res = client.execute(req).await?;

        println!("{url}");
        println!("{:?}", res.headers());
        println!("{:?}", res.status());

        // Pick up any cookies set by the request.
        {
            event!(Level::TRACE, "picking up set-cookie headers");
            for header in res.headers().get_all(reqwest::header::SET_COOKIE) {
                if let Ok(cookie_str) = header.to_str() {
                    if let Ok(cookie) = cookie::Cookie::parse(cookie_str) {
                        // Make sure the cookie is allowed to set the domain it does.
                        // https://datatracker.ietf.org/doc/html/rfc6265#section-5.3
                        if let Some(domain) = cookie.domain() {
                            // Don't allow cookie domain specifiers from IP addresses.
                            if let Some(url::Host::Domain(_)) = url.host() {
                            } else {
                                event!(
                                    Level::WARN,
                                    name = cookie.name(),
                                    host = ?url.host(),
                                    "ignoring set-cookie from ip host"
                                );
                                continue;
                            }

                            let url_host = url.host_str().expect("All relevant URLs have a host");

                            if !domain.contains('.') {
                                // Trying to set for a TLD, which we won't allow.
                                // This is not complete (e.g., .co.uk), but likely good
                                // enough here since we're just hitting internal services
                                // and Midway.
                                event!(
                                    Level::WARN,
                                    name = cookie.name(),
                                    %domain,
                                    "ignoring set-cookie for TLD"
                                );
                                continue;
                            }

                            // Only allowed to set for superdomains, not subdomains.
                            if let Some(remainder) = url_host.strip_suffix(domain) {
                                if remainder.is_empty() || remainder.ends_with('.') {
                                    // Either domain == URL host or domain holds some
                                    // suffix of the host components of the URL, in which
                                    // case it's a superdomain.
                                } else {
                                    event!(
                                        Level::WARN,
                                        name = cookie.name(),
                                        host = %url_host,
                                        %domain,
                                        "ignoring set-cookie for subdomain"
                                    );
                                    continue;
                                }
                            }
                        }

                        // Make sure to remove any cookie that should be replaced.
                        event!(
                            Level::TRACE,
                            name = cookie.name(),
                            path = ?cookie.path(),
                            domain = ?cookie.domain(),
                            host = ?url.host_str(),
                            "looking for overwritten cookies"
                        );
                        cookies.cookies.retain(|(old_cookie_url, old_cookie)| {
                            // Specifically, keep ones with
                            //
                            //  - a different name;
                            //  - a different path; or
                            //  - a different target domain
                            let unrelated = old_cookie.name() != cookie.name()
                                || old_cookie.path() != cookie.path()
                                || old_cookie.domain().or_else(|| old_cookie_url.host_str())
                                    != cookie.domain().or_else(|| url.host_str());
                            if !unrelated {
                                event!(Level::DEBUG, name = old_cookie.name(), "replacing cookie");
                            } else {
                                event!(
                                    Level::TRACE,
                                    name = old_cookie.name(),
                                    path = ?old_cookie.path(),
                                    domain = ?old_cookie.domain(),
                                    host = ?old_cookie_url.host_str(),
                                    "not replacing cookie"
                                );
                            }

                            unrelated
                        });

                        event!(Level::DEBUG, name = cookie.name(), "eating cookie");
                        cookies.cookies.push((url.clone(), cookie.into_owned()));
                    }
                }
            }
        }

        let status = res.status();

        // There are still Sentry SSO endpoints that redirect to Midway with a Location header.
        // However the returned HTTP status is 400.
        let is_sentry_redirect =
            (status == StatusCode::BAD_REQUEST) && res.headers().contains_key(reqwest::header::LOCATION);

        if !status.is_redirection() && !is_sentry_redirect {
            // If Midway returns a 4xx response, chances are the body is actually a Midway error,
            // not an error from the underlying service. We want to catch that and propagate it as
            // a connector error so that the user doesn't get an error seemingly related to the
            // response not matching the service model!
            //
            // Part of what makes this tricky is dealing with streaming bodies, and ensuring that
            // we leave everything in a usable state if the error _isn't_ a Midway error!
            //
            // See also https://amzn-aws.slack.com/archives/C01PSV1LSBC/p1651015546322179
            if (status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN)
                && url.domain() == Some("midway-auth.amazon.com")
            {
                event!(
                    Level::DEBUG,
                    %url,
                    "got 401 or 403 response from midway"
                );

                let err = res.error_for_status_ref().err();
                let res = reqwest_response_to_http_response(res).await?;

                MidwayError::maybe_midway_error(res.body())?;

                if let Some(err) = err {
                    return Err(err.into());
                }

                return Ok(res);
            }

            event!(
                Level::TRACE,
                %url,
                "passing through non-redirect response"
            );

            break reqwest_response_to_http_response(res.error_for_status()?).await;
        }
        if let Some(url) = res.headers_mut().remove(reqwest::header::LOCATION) {
            // NOTE: We parse this as a url::Url so that we get the nice auto-handling
            // of default ports, which need to be stripped so that Midway allowlisting
            // doesn't block https:// URLs with :443 for example.
            if let Some(url) = url.to_str().ok().and_then(|s| url::Url::parse(s).ok()) {
                // This middleware is only equipped to handle Midway redirects.
                // Anything else we let other layers deal with.
                let origin = url.origin();
                if origin == original_origin || origin == *MIDWAY_ORIGIN {
                } else {
                    event!(
                        Level::INFO,
                        %url,
                        "passing through redirect for unknown origin"
                    );
                    break reqwest_response_to_http_response(res.error_for_status()?).await;
                }

                next_url = url;
            } else {
                // This, too, should be considered an error, but we leave that to the
                // higher layers.
                event!(Level::ERROR, "passing through redirect with invalid location uri");
                break reqwest_response_to_http_response(res.error_for_status()?).await;
            }
        } else {
            // This should arguably be an error. But we can't easily represent it with
            // SendOperationError since it's not a _send_ error, so instead we just return
            // the response as-is.
            event!(Level::ERROR, "passing through redirect without location header");
            break reqwest_response_to_http_response(res.error_for_status()?).await;
        }

        // we don't have poll_ready anymore
        // We're going to have to follow the redirect.
        // Which means we have to call again.
        // Which means we have to wait for the inner service to be ready.
        // futures_util::future::poll_fn(|cx| svc.poll_ready(cx)).await?;

        event!(
            Level::DEBUG,
            url = %next_url,
            "following redirect"
        );
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::*;

    #[tokio::test]
    async fn test_midway() {
        let request = reqwest::Request::new(
            Method::GET,
            "https://gamma.us-east-1.shellspecs.jupiter.ai.aws.dev/index.json"
                .try_into()
                .unwrap(),
        );

        let res = midway_request(request).await;
        println!("{:?}", res);
    }
}
