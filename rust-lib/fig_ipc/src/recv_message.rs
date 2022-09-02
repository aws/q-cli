use std::io;

use async_trait::async_trait;
use bytes::Buf;
use fig_proto::prost::Message;
use fig_proto::{
    FigMessage,
    ReflectMessage,
};
use tokio::io::{
    AsyncRead,
    AsyncReadExt,
};

use crate::error::RecvError;
use crate::BufferedReader;

#[async_trait]
pub trait RecvMessage {
    async fn recv_message<R>(&mut self) -> Result<Option<R>, RecvError>
    where
        R: Message + ReflectMessage + Default;
}

#[async_trait]
impl<T> RecvMessage for BufferedReader<T>
where
    T: AsyncRead + Unpin + Send,
{
    async fn recv_message<M>(&mut self) -> Result<Option<M>, RecvError>
    where
        M: Message + ReflectMessage + Default,
    {
        macro_rules! read_buffer {
            () => {{
                let bytes = self.inner.read_buf(&mut self.buffer).await?;

                // If the buffer is empty, we've reached EOF
                if bytes == 0 {
                    if self.buffer.is_empty() {
                        return Ok(None);
                    } else {
                        return Err(RecvError::Io(io::Error::from(io::ErrorKind::UnexpectedEof)));
                    }
                }
            }};
        }

        // Read into buffer the first time
        read_buffer!();

        loop {
            // Try to parse the message until the buffer is a valid message
            let mut cursor = io::Cursor::new(&self.buffer);
            match FigMessage::parse(&mut cursor) {
                // If the parsed message is valid, return it
                Ok((len, message)) => {
                    self.buffer.advance(len);
                    return Ok(Some(message.decode()?));
                },
                // If the message is incomplete, read more into the buffer
                Err(fig_proto::FigMessageParseError::Incomplete(_)) => {
                    read_buffer!()
                },
                // On any other error, return the error
                Err(err) => {
                    // TODO(grant): add resyncing to message boundary
                    let position = cursor.position() as usize;
                    self.buffer.advance(position);
                    return Err(err.into());
                },
            }
        }
    }
}
