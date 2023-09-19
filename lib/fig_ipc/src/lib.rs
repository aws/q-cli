pub mod daemon;
pub mod local;
#[cfg(unix)]
pub mod util;

mod error;

mod buffered_reader;
mod recv_message;
mod send_message;
mod send_recv_message;
mod unix_socket;

pub use buffered_reader::BufferedReader;
pub use error::{
    ConnectError,
    Error,
    RecvError,
    SendError,
};
pub use recv_message::RecvMessage;
pub use send_message::SendMessage;
pub use send_recv_message::SendRecvMessage;
pub use unix_socket::{
    socket_connect,
    socket_connect_timeout,
    BufferedUnixStream,
};
