use fig_proto::fig::{
    NotificationRequest,
    NotificationType,
};

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::NotificationsState;

pub async fn handle_request(
    request: NotificationRequest,
    message_id: i64,
    state: &NotificationsState,
) -> RequestResult {
    let notification_type = NotificationType::from_i32(request.r#type.unwrap()).unwrap();

    if request.subscribe.unwrap_or(true) {
        subscribe(message_id, notification_type, state)
    } else {
        unsubscribe(notification_type, state)
    }
}

fn subscribe(channel: i64, notification_type: NotificationType, state: &NotificationsState) -> RequestResult {
    if notification_type == NotificationType::All {
        return RequestResult::error("Cannot subscribe to 'all' notification type");
    }

    if state.subscriptions.contains_key(&notification_type) {
        return RequestResult::error(format!("Already subscribed to notification type {notification_type:?}",));
    }

    state.subscriptions.insert(notification_type, channel);

    RequestResult::success()
}

fn unsubscribe(notification_type: NotificationType, state: &NotificationsState) -> RequestResult {
    if notification_type == NotificationType::All {
        return unsubscribe_all(state);
    }

    if !state.subscriptions.contains_key(&notification_type) {
        return RequestResult::error(format!("Not subscribed notification type {notification_type:?}",));
    }

    state.subscriptions.remove(&notification_type);

    RequestResult::success()
}

fn unsubscribe_all(state: &NotificationsState) -> RequestResult {
    state.subscriptions.clear();
    RequestResult::success()
}
