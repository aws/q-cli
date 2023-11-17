use aws_sdk_ssooidc::error::{
    DisplayErrorContext,
    SdkError,
};
use aws_sdk_ssooidc::operation::create_token::CreateTokenError;
use aws_sdk_ssooidc::operation::register_client::RegisterClientError;
use aws_sdk_ssooidc::operation::start_device_authorization::StartDeviceAuthorizationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{}", DisplayErrorContext(&.0))]
    Ssooidc(#[from] Box<aws_sdk_ssooidc::Error>),
    #[error("{}", DisplayErrorContext(&.0))]
    SdkRegisterClient(#[from] SdkError<RegisterClientError>),
    #[error("{}", DisplayErrorContext(&.0))]
    SdkCreateToken(#[from] SdkError<CreateTokenError>),
    #[error("{}", DisplayErrorContext(&.0))]
    SdkStartDeviceAuthorization(#[from] SdkError<StartDeviceAuthorizationError>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TimeComponentRange(#[from] time::error::ComponentRange),
    #[error(transparent)]
    Directories(#[from] fig_util::directories::DirectoryError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("Security error: {}", .0)]
    Security(String),
    #[error(transparent)]
    StringFromUtf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    StrFromUtf8(#[from] std::str::Utf8Error),
    #[error("No token")]
    NoToken,
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;
