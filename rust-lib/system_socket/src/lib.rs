use std::fmt::Debug;
use std::io;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;
use std::pin::Pin;
use std::task::{
    Context,
    Poll,
};

use pin_project::pin_project;
use tokio::io::{
    AsyncRead,
    AsyncWrite,
};
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use tokio::net::UnixStream;
use tokio::sync::oneshot;
#[cfg(windows)]
use uds_windows::UnixListener;
#[cfg(windows)]
use uds_windows::UnixStream;

#[derive(Debug)]
#[pin_project]
pub struct SystemListener(#[pin] UnixListener);

impl SystemListener {
    /// Creates a new `SystemListener` bound to the specified path.
    ///
    /// # Panics
    ///
    /// This function panics if thread-local runtime is not set.
    #[allow(unused_variables)]
    pub fn bind<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(Self(UnixListener::bind(path.as_ref())?))
    }

    /// Accepts a client and spawns a task that runs the given handler for it.
    pub async fn accept(&self) -> io::Result<SystemStream> {
        #[cfg(unix)]
        let (stream, _) = self.0.accept().await?;
        #[cfg(windows)]
        let (stream, _) = tokio::task::block_in_place(|| self.0.accept())?;
        Ok(SystemStream::from(stream))
    }
}

#[derive(Debug)]
#[pin_project]
pub struct SystemStream(#[pin] UnixStream);

impl SystemStream {
    /// Connects to the socket named by `path`.
    ///
    /// This function will create a new system socket and connect to the path
    /// specified, associating the returned stream with the default event loop's
    pub async fn connect<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        #[cfg(unix)]
        let stream = UnixStream::connect(path.as_ref()).await?;
        #[cfg(windows)]
        let stream = tokio::task::block_in_place(|| UnixStream::connect(path.as_ref()))?;
        Ok(Self(stream))
    }

    /// Waits for the socket to become writable.
    ///
    /// This function is equivalent to `ready(Interest::WRITABLE)` and is usually
    /// paired with `try_write()`.
    pub async fn writable(&self) -> io::Result<()> {
        #[cfg(unix)]
        return self.0.writable().await;
        #[cfg(windows)]
        return Ok(());
    }
}

impl From<UnixStream> for SystemStream {
    fn from(from: UnixStream) -> Self {
        Self(from)
    }
}

impl AsyncRead for SystemStream {
    #[cfg(unix)]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.project().0.poll_read(cx, buf)
    }

    #[cfg(windows)]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        use std::io::Read;

        let mut read = vec![];
        let len = tokio::task::block_in_place(|| self.project().0.read(&mut read))?;
        buf.set_filled(len);

        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for SystemStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        #[cfg(unix)]
        return self.project().0.poll_write(cx, buf);
        #[cfg(windows)]
        return Poll::Ready(tokio::task::block_in_place(|| self.project().0.write(buf)));
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        #[cfg(unix)]
        return self.project().0.poll_flush(cx);
        #[cfg(windows)]
        return Poll::Ready(tokio::task::block_in_place(|| self.project().0.flush()));
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        #[cfg(unix)]
        return self.project().0.poll_shutdown(cx);
        #[cfg(windows)]
        return Poll::Ready(tokio::task::block_in_place(|| self.0.shutdown(std::net::Shutdown::Both)));
    }
}
