use fig_proto::fig::{NotificationRequest, NotificationType};

use crate::api::ResponseKind;
use crate::state::STATE;

use super::ResponseResult;

pub async fn handle_request(request: NotificationRequest, message_id: i64) -> ResponseResult {
    let notification_type = NotificationType::from_i32(request.r#type.unwrap()).unwrap();

    if request.subscribe.unwrap_or_else(|| true) {
        return subscribe(message_id, notification_type);
    } else {
        return unsubscribe(notification_type);
    }
}

fn subscribe(channel: i64, notification_type: NotificationType) -> ResponseResult {
    if notification_type == NotificationType::All {
        return Err(ResponseKind::Error(
            "Cannot subscribe to 'all' notification type".to_string(),
        ));
    }

    if STATE.lock().subscriptions.contains_key(&notification_type) {
        return Err(ResponseKind::Error(format!(
            "Already subscribed to notification type {:?}",
            notification_type
        )));
    }

    STATE
        .lock()
        .subscriptions
        .insert(notification_type, channel);

    Ok(ResponseKind::Success)
}

fn unsubscribe(notification_type: NotificationType) -> ResponseResult {
    if notification_type == NotificationType::All {
        return unsubscribe_all();
    }

    if !STATE.lock().subscriptions.contains_key(&notification_type) {
        return Err(ResponseKind::Error(format!(
            "Not subscribed notification type {:?}",
            notification_type
        )));
    }

    STATE.lock().subscriptions.remove(&notification_type);

    Ok(ResponseKind::Success)
}

fn unsubscribe_all() -> ResponseResult {
    STATE.lock().subscriptions.clear();
    Ok(ResponseKind::Success)
}
