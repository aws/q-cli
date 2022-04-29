use std::fmt::Debug;
use std::io;
use std::path::Path;

use pin_project::pin_project;
#[cfg(windows)]
use tokio::net::windows::named_pipe::NamedPipeServer;
#[cfg(unix)]
use tokio::net::UnixListener;

use crate::SystemStream;

#[cfg(unix)]
#[derive(Debug)]
#[pin_project]
pub struct SystemListener(#[pin] UnixListener);
#[cfg(windows)]
#[derive(Debug)]
#[pin_project]
pub struct SystemListener(#[pin] NamedPipeServer, std::path::PathBuf);

impl SystemListener {
    /// Creates a new `SystemListener` bound to the specified path.
    ///
    /// # Panics
    ///
    /// This function panics if thread-local runtime is not set.
    #[allow(unused_variables)]
    pub fn bind<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        #[cfg(unix)]
        return Ok(Self(UnixListener::bind(path.as_ref())?));
        #[cfg(windows)]
        Ok(Self(
            {
                use tokio::net::windows::named_pipe::ServerOptions;
                ServerOptions::new()
                    .first_pipe_instance(true)
                    .create(path.as_ref())?
            },
            path.as_ref().to_path_buf(),
        ))
    }

    /// Accepts a client and spawns a task that runs the given handler for it.
    #[cfg(unix)]
    pub async fn accept(&mut self) -> io::Result<SystemStream> {
        let (stream, _) = self.0.accept().await?;
        Ok(stream.into())
    }

    /// Accepts a client and spawns a task that runs the given handler for it.
    #[cfg(windows)]
    pub async fn accept(&mut self) -> io::Result<SystemStream> {
        use tokio::net::windows::named_pipe::ServerOptions;

        self.0.connect().await?;
        let mut stream = ServerOptions::new().create(&self.1)?;
        std::mem::swap(&mut self.0, &mut stream);

        Ok(stream.into())
    }

    /// Waits for the listener to become writable.
    ///
    /// This function is equivalent to `ready(Interest::WRITABLE)` and is usually
    /// paired with `try_write()`.
    #[cfg(windows)]
    pub async fn writable(&self) -> io::Result<()> {
        self.0.writable().await
    }
}

#[cfg(windows)]
impl tokio::io::AsyncRead for SystemListener {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.project().0.poll_read(cx, buf)
    }
}

#[cfg(windows)]
impl tokio::io::AsyncWrite for SystemListener {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        self.project().0.poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.project().0.poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.project().0.poll_shutdown(cx)
    }
}
