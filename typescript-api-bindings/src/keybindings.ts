import { KeybindingPressedNotification, NotificationType } from './fig';
import { _subscribe } from './notifications';

export const pressed = (
  handler: (notification: KeybindingPressedNotification) => boolean | undefined
) => {
  return _subscribe(
    { type: NotificationType.NOTIFY_ON_KEYBINDING_PRESSED },
    notification => {
      switch (notification?.type?.$case) {
        case 'keybindingPressedNotification':
          return handler(notification.type.keybindingPressedNotification);
        default:
          break;
      }

      return false;
    }
  );
};
