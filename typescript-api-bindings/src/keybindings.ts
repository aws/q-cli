import {
  Action,
  KeybindingPressedNotification,
  NotificationType,
} from './fig.pb';
import { sendUpdateApplicationPropertiesRequest } from './requests';
import { _subscribe } from './notifications';

export function pressed(
  handler: (notification: KeybindingPressedNotification) => boolean | undefined
) {
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
}

export function setInterceptKeystrokes(
  actions: Action[],
  intercept: boolean,
  globalIntercept?: boolean
) {
  sendUpdateApplicationPropertiesRequest({
    interceptBoundKeystrokes: intercept,
    interceptGlobalKeystrokes: globalIntercept,
    actionList: { actions },
  });
}
