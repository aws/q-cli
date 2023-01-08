use reqwest::StatusCode;

use crate::{
    auth,
    defaults,
};

#[derive(Debug)]
pub enum Error {
    Fig {
        error: String,
        status: StatusCode,
        sentry_id: Option<String>,
    },
    Graphql(Vec<graphql_client::Error>),
    GraphqlNoData,
    Reqwest(reqwest::Error),
    Status(StatusCode),
    Serde(serde_json::Error),
    Defaults(defaults::DefaultsError),
    Io(std::io::Error),
    Dir(fig_util::directories::DirectoryError),
    RefreshError(auth::RefreshError),
    Settings(fig_settings::Error),
    NoClient,
    NoToken,
}

impl Error {
    pub fn is_status(&self, status: StatusCode) -> bool {
        match self {
            Error::Fig { status: s, .. } | Error::Status(s) => *s == status,
            _ => false,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Fig {
                error,
                status,
                sentry_id,
            } => match sentry_id {
                Some(sentry_id) => {
                    write!(f, "{error} (status: {status}, error_id: {sentry_id})",)
                },
                None => write!(f, "{error} (status: {status})"),
            },
            Error::Graphql(err) => match err.len() {
                0 => write!(f, "Empty graphql error"),
                1 => write!(f, "Graphql error: {}", err[0]),
                _ => write!(f, "Graphql errors: {err:?}"),
            },
            Error::GraphqlNoData => write!(f, "Graphql error: No data"),
            Error::Reqwest(err) => write!(f, "Reqwest error: {err}"),
            Error::Status(err) => write!(f, "Status error: {err}"),
            Error::Serde(err) => write!(f, "Serde error: {err}"),
            Error::Defaults(err) => write!(f, "Defaults error: {err}"),
            Error::Io(err) => write!(f, "Io error: {err}"),
            Error::Dir(err) => write!(f, "Dir error: {err}"),
            Error::RefreshError(err) => write!(f, "Refresh error: {err}"),
            Error::Settings(err) => write!(f, "Settings error: {err}"),
            Error::NoClient => write!(f, "No client"),
            Error::NoToken => write!(f, "No token"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Vec<graphql_client::Error>> for Error {
    fn from(e: Vec<graphql_client::Error>) -> Self {
        Error::Graphql(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl From<StatusCode> for Error {
    fn from(e: StatusCode) -> Self {
        Error::Status(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

impl From<defaults::DefaultsError> for Error {
    fn from(e: defaults::DefaultsError) -> Self {
        Error::Defaults(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<fig_util::directories::DirectoryError> for Error {
    fn from(e: fig_util::directories::DirectoryError) -> Self {
        Error::Dir(e)
    }
}

impl From<fig_settings::Error> for Error {
    fn from(e: fig_settings::Error) -> Self {
        Error::Settings(e)
    }
}

impl From<auth::RefreshError> for Error {
    fn from(e: auth::RefreshError) -> Self {
        Error::RefreshError(e)
    }
}
