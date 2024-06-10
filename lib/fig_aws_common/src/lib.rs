use std::borrow::Cow;
use std::fmt;
use std::sync::OnceLock;

use aws_runtime::user_agent::{
    ApiMetadata,
    AwsUserAgent,
};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextMut;
use aws_smithy_runtime_api::client::interceptors::Intercept;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::ConfigBag;
use aws_types::app_name::AppName;
use aws_types::os_shim_internal::Env;
use http::header::{
    InvalidHeaderValue,
    USER_AGENT,
};

const APP_NAME_STR: &str = "AmazonQ-For-CLI";

pub fn app_name() -> AppName {
    static APP_NAME: OnceLock<AppName> = OnceLock::new();
    APP_NAME
        .get_or_init(|| AppName::new(APP_NAME_STR).expect("invalid app name"))
        .clone()
}

#[derive(Debug)]
enum UserAgentOverrideInterceptorError {
    MissingApiMetadata,
    InvalidHeaderValue(InvalidHeaderValue),
}

impl std::error::Error for UserAgentOverrideInterceptorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidHeaderValue(source) => Some(source),
            Self::MissingApiMetadata => None,
        }
    }
}

impl fmt::Display for UserAgentOverrideInterceptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidHeaderValue(_) => "AwsUserAgent generated an invalid HTTP header value. This is a bug. Please file an issue.",
            Self::MissingApiMetadata => "The UserAgentInterceptor requires ApiMetadata to be set before the request is made. This is a bug. Please file an issue.",
        })
    }
}

impl From<InvalidHeaderValue> for UserAgentOverrideInterceptorError {
    fn from(err: InvalidHeaderValue) -> Self {
        UserAgentOverrideInterceptorError::InvalidHeaderValue(err)
    }
}

/// Generates and attaches the AWS SDK's user agent to a HTTP request
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct UserAgentOverrideInterceptor {}

impl UserAgentOverrideInterceptor {
    /// Creates a new `UserAgentInterceptor`
    pub const fn new() -> Self {
        UserAgentOverrideInterceptor {}
    }
}

impl Intercept for UserAgentOverrideInterceptor {
    fn name(&self) -> &'static str {
        "UserAgentOverrideInterceptor"
    }

    fn modify_before_signing(
        &self,
        context: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        // Allow for overriding the user agent by an earlier interceptor (so, for example,
        // tests can use `AwsUserAgent::for_tests()`) by attempting to grab one out of the
        // config bag before creating one.
        let ua: Cow<'_, AwsUserAgent> = cfg.load::<AwsUserAgent>().map(Cow::Borrowed).map_or_else(
            || {
                let api_metadata = cfg
                    .load::<ApiMetadata>()
                    .ok_or(UserAgentOverrideInterceptorError::MissingApiMetadata)?;
                let mut ua = AwsUserAgent::new_from_environment(Env::real(), api_metadata.clone());

                let maybe_app_name = cfg.load::<AppName>();
                if let Some(app_name) = maybe_app_name {
                    ua.set_app_name(app_name.clone());
                }
                Ok(Cow::Owned(ua))
            },
            Result::<_, UserAgentOverrideInterceptorError>::Ok,
        )?;

        let headers = context.request_mut().headers_mut();
        headers.insert(USER_AGENT, ua.aws_ua_header());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_name() {
        println!("{}", app_name());
    }
}
