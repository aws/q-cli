// Shared logic between Linux and MacOS (the only relevant UNIX compliant operating systems)

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::net::{UnixListener, UnixStream};

use super::GenericListener;

pub fn bind_socket(path: &Path) -> UnixListener {
    UnixListener::bind(path).expect("Failed to bind to socket")
}

#[async_trait]
impl GenericListener for UnixListener {
    type Stream = UnixStream;

    async fn generic_accept(&self) -> Result<Self::Stream, anyhow::Error> {
        match self.accept().await {
            Ok((stream, _)) => Ok(stream),
            Err(err) => Err(anyhow::Error::new(err)),
        }
    }
}
