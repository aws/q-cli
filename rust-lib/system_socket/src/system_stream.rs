// THIS FILE CONTAINS UNSAFE CODE, EDIT ONLY IF YOU KNOW WHAT YOU'RE DOING.

use std::io;
use std::path::Path;
use std::pin::Pin;

use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;

#[cfg(unix)]
#[derive(Debug)]
pub struct SystemStream(tokio::net::UnixStream);
#[cfg(windows)]
#[derive(Debug)]
pub struct SystemStream(tokio::net::windows::named_pipe::NamedPipeClient);

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
                Ok(conn) => return Ok(Self(conn)),
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
        self.0.writable().await
    }

    /// Retrieve a projection of the inner field which works in pinned contexts
    #[cfg(unix)]
    pub fn pinned_inner(self: Pin<&mut Self>) -> Pin<&mut tokio::net::UnixStream> {
        // SAFETY: This is safe because self is pinned when called
        unsafe { self.map_unchecked_mut(|s| &mut s.0) }
    }

    /// Retrieve a projection of the inner field which works in pinned contexts
    #[cfg(windows)]
    pub fn pinned_inner(
        self: Pin<&mut Self>,
    ) -> Pin<&mut tokio::net::windows::named_pipe::NamedPipeClient> {
        // SAFETY: This is safe because self is pinned when called
        unsafe { self.map_unchecked_mut(|s| &mut s.0) }
    }
}

impl AsyncRead for SystemStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.pinned_inner().poll_read(cx, buf)
    }
}

impl AsyncWrite for SystemStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        self.pinned_inner().poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.pinned_inner().poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.pinned_inner().poll_shutdown(cx)
    }
}
