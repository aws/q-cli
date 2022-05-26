use fig_proto::fig::{
    NotificationRequest,
    NotificationType,
};

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::{
    FigId,
    NotificationsState,
};

pub async fn handle_request(
    request: NotificationRequest,
    fig_id: FigId,
    message_id: i64,
    state: &NotificationsState,
) -> RequestResult {
    let notification_type = NotificationType::from_i32(request.r#type.unwrap()).unwrap();

    if request.subscribe.unwrap_or(true) {
        subscribe(fig_id, message_id, notification_type, state)
    } else {
        unsubscribe(&fig_id, notification_type, state)
    }
}

fn subscribe(
    fig_id: FigId,
    channel: i64,
    notification_type: NotificationType,
    state: &NotificationsState,
) -> RequestResult {
    if notification_type == NotificationType::All {
        return RequestResult::error("Cannot subscribe to 'all' notification type");
    }

    let entry = state.subscriptions.entry(fig_id).or_default();
    if entry.contains_key(&notification_type) {
        return RequestResult::error(format!("Already subscribed to notification type {notification_type:?}",));
    }

    entry.insert(notification_type, channel);

    RequestResult::success()
}

fn unsubscribe(fig_id: &FigId, notification_type: NotificationType, state: &NotificationsState) -> RequestResult {
    if notification_type == NotificationType::All {
        return unsubscribe_all(fig_id, state);
    }

    match state.subscriptions.get(fig_id) {
        Some(subscriptions) if !subscriptions.contains_key(&notification_type) => {
            return RequestResult::error(format!("Not subscribed notification type {notification_type:?}",));
        },
        None => {
            return RequestResult::error(format!("Not subscribed notification type {notification_type:?}",));
        },
        Some(subscriptions) => {
            subscriptions.remove(&notification_type);
        },
    }

    RequestResult::success()
}

fn unsubscribe_all(fig_id: &FigId, state: &NotificationsState) -> RequestResult {
    if let Some(subscriptions) = state.subscriptions.get(fig_id) {
        subscriptions.clear();
    }

    RequestResult::success()
}
