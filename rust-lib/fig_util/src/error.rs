use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io operation error")]
    IoError(#[from] std::io::Error),
    #[error("unsupported platform")]
    UnsupportedPlatform,
    #[error("process has no parent")]
    NoParentProcess,
    #[error("could not find the os hwid")]
    HwidNotFound,
    #[error("the shell, `{0}`, isn't supported yet")]
    UnknownShell(String),
}
