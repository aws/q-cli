use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromptAndSendError {
    FigClientError(fig_api_client::Error),
    IO(std::io::Error),
    Report(eyre::Report),
}

impl std::fmt::Display for PromptAndSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptAndSendError::FigClientError(err) => err.fmt(f),
            PromptAndSendError::IO(err) => err.fmt(f),
            PromptAndSendError::Report(err) => err.fmt(f),
        }
    }
}

impl From<fig_api_client::Error> for PromptAndSendError {
    fn from(value: fig_api_client::Error) -> Self {
        PromptAndSendError::FigClientError(value)
    }
}

impl From<std::io::Error> for PromptAndSendError {
    fn from(value: std::io::Error) -> Self {
        PromptAndSendError::IO(value)
    }
}

impl From<eyre::Report> for PromptAndSendError {
    fn from(value: eyre::Report) -> Self {
        PromptAndSendError::Report(value)
    }
}
