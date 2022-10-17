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
    EmitEventName,
    Event,
    WindowEvent,
};
use crate::webview::window::WindowId;
use crate::EventLoopProxy;

#[derive(Debug, Default)]
pub struct WebviewNotificationsState {
    pub subscriptions: DashMap<WindowId, DashMap<NotificationType, i64, FnvBuildHasher>, FnvBuildHasher>,
}

impl WebviewNotificationsState {
    pub async fn broadcast_notification_all(
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
                    event_name: EmitEventName::Notification,
                    payload: base64::encode(encoded),
                },
            })?
        }

        Ok(())
    }
}
