// THIS FILE CONTAINS UNSAFE CODE, EDIT ONLY IF YOU KNOW WHAT YOU'RE DOING.
use std::io;
use std::pin::Pin;

use crate::SystemListener;
use crate::SystemStream;

/// Platform abstracting type for socket communication.
///
/// In regards to IPC, only listening processes will interact with this type.
pub enum SystemConnection {
    Listener(SystemListener),
    Stream(SystemStream),
}

impl SystemConnection {
    /// Waits for the connection to become writable.
    ///
    /// This function is equivalent to `ready(Interest::WRITABLE)` and is usually
    /// paired with `try_write()`.
    pub async fn writable(&self) -> io::Result<()> {
        match self {
            Self::Listener(l) => l.writable().await,
            Self::Stream(s) => s.writable().await,
        }
    }

    /// Retrieve a projection of the inner field which works in pinned contexts
    #[cfg(unix)]
    fn pinned_inner(self: Pin<&mut Self>) -> Pin<&mut SystemStream> {
        // SAFETY: This is safe because self is pinned when called
        unsafe {
            self.map_unchecked_mut(|c| match c {
                SystemConnection::Listener(_) => unreachable!("Unix always returns a stream"),
                SystemConnection::Stream(s) => return s,
            })
        }
    }

    /// Retrieve a projection of the inner field which works in pinned contexts
    #[cfg(windows)]
    fn pinned_inner(self: Pin<&mut Self>) -> Pin<&mut SystemListener> {
        // SAFETY: This is safe because self is pinned when called
        unsafe {
            self.map_unchecked_mut(|c| match c {
                SystemConnection::Listener(l) => return l,
                SystemConnection::Stream(_) => unreachable!("Windows always returns a listener"),
            })
        }
    }
}

impl From<SystemListener> for SystemConnection {
    fn from(from: SystemListener) -> Self {
        Self::Listener(from)
    }
}

impl From<SystemStream> for SystemConnection {
    fn from(from: SystemStream) -> Self {
        Self::Stream(from)
    }
}

#[cfg(windows)]
impl tokio::io::AsyncRead for SystemConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.pinned_inner().poll_read(cx, buf)
    }
}

#[cfg(windows)]
impl tokio::io::AsyncWrite for SystemConnection {
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
