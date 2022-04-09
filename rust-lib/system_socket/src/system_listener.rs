// THIS FILE CONTAINS UNSAFE CODE, EDIT ONLY IF YOU KNOW WHAT YOU'RE DOING.
use std::fmt::Debug;
use std::io;
use std::path::Path;
use std::pin::Pin;

#[cfg(windows)]
use tokio::net::windows::named_pipe::NamedPipeServer;
#[cfg(unix)]
use tokio::net::UnixListener;

use crate::system_connection::SystemConnection;

#[cfg(unix)]
#[derive(Debug)]
pub struct SystemListener(UnixListener);
#[cfg(windows)]
#[derive(Debug)]
pub struct SystemListener(NamedPipeServer);

impl SystemListener {
    /// Creates a new `SystemListener` bound to the specified path.
    ///
    /// # Panics
    ///
    /// This function panics if thread-local runtime is not set.
    pub fn bind<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        #[cfg(unix)]
        let listener = {
            use tokio::net::UnixListener;
            UnixListener::bind(path.as_ref())?
        };
        #[cfg(windows)]
        let listener = {
            use tokio::net::windows::named_pipe::ServerOptions;
            ServerOptions::new()
                .first_pipe_instance(true)
                .create(path.as_ref())?
        };
        Ok(Self(listener))
    }

    /// Accepts a client and spawns a task that runs the given handler for it.
    #[cfg(unix)]
    pub async fn accept<P>(&mut self, path: P) -> io::Result<SystemConnection>
    where
        P: AsRef<Path>,
    {
        use crate::SystemStream;

        let (stream, _addr) = self.0.accept().await?;
        tokio::spawn(handler(stream.into())).await?;

        Ok(SystemStream::from(stream).into())
    }

    /// Accepts a client and spawns a task that runs the given handler for it.
    #[cfg(windows)]
    pub async fn accept<P>(&mut self, path: P) -> io::Result<SystemConnection>
    where
        P: AsRef<Path>,
    {
        use tokio::net::windows::named_pipe::ServerOptions;

        self.0.connect().await?;
        let mut stream = ServerOptions::new().create(path.as_ref())?;
        std::mem::swap(&mut self.0, &mut stream);

        Ok(Self(stream).into())
    }

    /// Waits for the listener to become writable.
    ///
    /// This function is equivalent to `ready(Interest::WRITABLE)` and is usually
    /// paired with `try_write()`.
    #[cfg(windows)]
    pub async fn writable(&self) -> io::Result<()> {
        self.0.writable().await
    }

    /// Retrieve a projection of the inner field which works in pinned contexts
    #[cfg(windows)]
    fn pinned_inner(
        self: Pin<&mut Self>,
    ) -> Pin<&mut tokio::net::windows::named_pipe::NamedPipeServer> {
        // SAFETY: This is safe because self is pinned when called
        unsafe { self.map_unchecked_mut(|s| &mut s.0) }
    }
}

#[cfg(windows)]
impl tokio::io::AsyncRead for SystemListener {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.pinned_inner().poll_read(cx, buf)
    }
}

#[cfg(windows)]
impl tokio::io::AsyncWrite for SystemListener {
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
