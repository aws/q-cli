use std::io;
use std::path::Path;
use std::pin::Pin;

use pin_project::pin_project;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
#[cfg(windows)]
use tokio::net::windows::named_pipe::{NamedPipeClient, NamedPipeServer};
#[cfg(unix)]
use tokio::net::UnixStream;

#[cfg(unix)]
#[derive(Debug)]
#[pin_project]
pub struct SystemStream(#[pin] UnixStream);
#[cfg(windows)]
#[derive(Debug)]
#[pin_project(project = EnumProj)]
pub enum SystemStream {
    Client(#[pin] NamedPipeClient),
    Server(#[pin] NamedPipeServer),
}

impl SystemStream {
    /// Connects to the socket named by `path`.
    ///
    /// This function will create a new system socket and connect to the path
    /// specified, associating the returned stream with the default event loop's
    #[cfg(unix)]
    pub async fn connect<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let stream = tokio::net::UnixStream::connect(path.as_ref()).await?;
        Ok(Self(stream))
    }

    /// Connects to the socket named by `path`.
    ///
    /// This function will create a new system socket and connect to the path
    /// specified, associating the returned stream with the default event loop's
    #[cfg(windows)]
    pub async fn connect<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        use std::time::Duration;
        use tokio::net::windows::named_pipe::ClientOptions;
        use winapi::shared::winerror;

        loop {
            match ClientOptions::new().open(path.as_ref()) {
                Ok(conn) => return Ok(Self::Client(conn)),
                Err(err) if err.raw_os_error() == Some(winerror::ERROR_PIPE_BUSY as i32) => (),
                Err(err) => return Err(err),
            }

            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }

    /// Waits for the socket to become writable.
    ///
    /// This function is equivalent to `ready(Interest::WRITABLE)` and is usually
    /// paired with `try_write()`.
    pub async fn writable(&self) -> io::Result<()> {
        #[cfg(unix)]
        return self.0.writable().await;
        #[cfg(windows)]
        match self {
            SystemStream::Client(c) => c.writable().await,
            SystemStream::Server(s) => s.writable().await,
        }
    }
}

#[cfg(unix)]
impl From<UnixStream> for SystemStream {
    fn from(from: NamedPipeClient) -> Self {
        Self(from)
    }
}

#[cfg(windows)]
impl From<NamedPipeClient> for SystemStream {
    fn from(from: NamedPipeClient) -> Self {
        Self::Client(from)
    }
}

#[cfg(windows)]
impl From<NamedPipeServer> for SystemStream {
    fn from(from: NamedPipeServer) -> Self {
        Self::Server(from)
    }
}

impl AsyncRead for SystemStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        #[cfg(unix)]
        return self.project().0.poll_read(cx, buf);
        #[cfg(windows)]
        return match self.project() {
            EnumProj::Client(c) => c.poll_read(cx, buf),
            EnumProj::Server(s) => s.poll_read(cx, buf),
        };
    }
}

impl AsyncWrite for SystemStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        #[cfg(unix)]
        return self.project().0.poll_write(cx, buf);
        #[cfg(windows)]
        return match self.project() {
            EnumProj::Client(c) => c.poll_write(cx, buf),
            EnumProj::Server(s) => s.poll_write(cx, buf),
        };
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        #[cfg(unix)]
        return self.project().0.poll_flush(cx);
        #[cfg(windows)]
        return match self.project() {
            EnumProj::Client(c) => c.poll_flush(cx),
            EnumProj::Server(s) => s.poll_flush(cx),
        };
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        #[cfg(unix)]
        return self.project().0.poll_shutdown(cx);
        #[cfg(windows)]
        return match self.project() {
            EnumProj::Client(c) => c.poll_shutdown(cx),
            EnumProj::Server(s) => s.poll_shutdown(cx),
        };
    }
}
