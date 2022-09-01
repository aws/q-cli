use std::time::Duration;

use async_trait::async_trait;
use fig_proto::prost::Message;
use fig_proto::{
    FigProtobufEncodable,
    ReflectMessage,
};
use tokio::io::{
    AsyncRead,
    AsyncWrite,
};
use tracing::error;

use crate::{
    BufferedReader,
    Error,
    RecvMessage,
    SendMessage,
};

#[async_trait]
pub trait SendRecvMessage: SendMessage + RecvMessage {
    async fn send_recv_message<M, R>(&mut self, message: M) -> Result<Option<R>, Error>
    where
        M: FigProtobufEncodable,
        R: Message + ReflectMessage + Default;

    async fn send_recv_message_timeout<M, R>(&mut self, message: M, timeout: Duration) -> Result<Option<R>, Error>
    where
        M: FigProtobufEncodable,
        R: Message + ReflectMessage + Default;
}

#[async_trait]
impl<T> SendRecvMessage for BufferedReader<T>
where
    T: AsyncWrite + AsyncRead + Unpin + Send,
{
    async fn send_recv_message<M, R>(&mut self, message: M) -> Result<Option<R>, Error>
    where
        M: FigProtobufEncodable,
        R: Message + ReflectMessage + Default,
    {
        self.send_message(message).await?;
        Ok(self.recv_message().await?)
    }

    async fn send_recv_message_timeout<M, R>(&mut self, message: M, timeout: Duration) -> Result<Option<R>, Error>
    where
        M: FigProtobufEncodable,
        R: Message + ReflectMessage + Default,
    {
        self.send_message(message).await?;
        Ok(match tokio::time::timeout(timeout, self.recv_message()).await {
            Ok(result) => result?,
            Err(_) => {
                error!("Timeout while receiving response from message");
                return Err(Error::Timeout);
            },
        })
    }
}
