use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Connect(#[from] ConnectError),
    #[error(transparent)]
    Send(#[from] SendError),
    #[error(transparent)]
    Recv(#[from] RecvError),
    #[error("timeout")]
    Timeout,
    #[error(transparent)]
    Dir(#[from] fig_util::directories::DirectoryError),
}

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("timeout connecting to socket")]
    Timeout,
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error(transparent)]
    Encode(#[from] fig_proto::FigMessageEncodeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum RecvError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Parse(#[from] fig_proto::FigMessageParseError),
    #[error(transparent)]
    Decode(#[from] fig_proto::FigMessageDecodeError),
}

impl RecvError {
    pub fn is_disconnect(&self) -> bool {
        if let RecvError::Io(io) = self {
            #[cfg(windows)]
            {
                use windows_sys::Win32::Networking::WinSock::WSAECONNRESET;
                if let Some(WSAECONNRESET) = io.raw_os_error() {
                    return true;
                }
            }
            matches!(io.kind(), std::io::ErrorKind::ConnectionAborted)
        } else {
            false
        }
    }
}
