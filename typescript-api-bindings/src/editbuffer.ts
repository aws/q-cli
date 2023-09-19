import { EditBufferChangedNotification, NotificationType } from './fig.pb';
import { _subscribe, NotificationResponse } from './notifications';

export function subscribe(
  handler: (notification: EditBufferChangedNotification) => NotificationResponse | undefined
) {
  return _subscribe(
    { type: NotificationType.NOTIFY_ON_EDITBUFFFER_CHANGE },
    notification => {
      switch (notification?.type?.$case) {
        case 'editBufferNotification':
          return handler(notification.type.editBufferNotification);
        default:
          break;
      }

      return { unsubscribe: false };
    }
  );
}
