use anyhow::Result;
use bytes::BytesMut;
use dashmap::DashMap;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    Notification,
    NotificationType,
    ServerOriginatedMessage,
};
use fig_proto::prost::Message;
use fnv::FnvBuildHasher;

use crate::event::{
    Event,
    WindowEvent,
};
use crate::webview::window::WindowId;
use crate::{
    EventLoopProxy,
    FIG_PROTO_MESSAGE_RECEIVED,
};

#[derive(Debug, Default)]
pub struct NotificationsState {
    pub subscriptions: DashMap<WindowId, DashMap<NotificationType, i64, FnvBuildHasher>, FnvBuildHasher>,
}

impl NotificationsState {
    pub async fn send_notification(
        &self,
        notification_type: &NotificationType,
        notification: Notification,
        proxy: &EventLoopProxy,
    ) -> Result<()> {
        for sub in self.subscriptions.iter() {
            let message_id = match sub.get(notification_type) {
                Some(id) => *id,
                None => continue,
            };

            let message = ServerOriginatedMessage {
                id: Some(message_id),
                submessage: Some(ServerOriginatedSubMessage::Notification(notification.clone())),
            };

            let mut encoded = BytesMut::new();
            message.encode(&mut encoded)?;

            proxy.send_event(Event::WindowEvent {
                window_id: sub.key().clone(),
                window_event: WindowEvent::Emit {
                    event: FIG_PROTO_MESSAGE_RECEIVED.into(),
                    payload: base64::encode(encoded),
                },
            })?
        }

        Ok(())
    }
}
