// Shared logic between Linux and MacOS (the only relevant UNIX compliant operating systems)

use std::path::Path;

use tokio::net::{UnixListener, UnixStream};

pub struct Listener(UnixListener);

impl Listener {
    pub fn bind(path: &Path) -> Self {
        Self(UnixListener::bind(path).expect("Failed to bind to socket"))
    }

    pub async fn accept(&self) -> Result<UnixStream, anyhow::Error> {
        match self.0.accept().await {
            Ok((stream, _)) => Ok(stream),
            Err(err) => Err(anyhow::Error::new(err)),
        }
    }
}
