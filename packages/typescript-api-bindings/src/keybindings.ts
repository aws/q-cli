import {
  Action,
  KeybindingPressedNotification,
  NotificationType,
} from "@fig/fig-api-proto/dist/fig.pb";
import { sendUpdateApplicationPropertiesRequest } from "./requests";
import { _subscribe, NotificationResponse } from "./notifications";

export function pressed(
  handler: (
    notification: KeybindingPressedNotification,
  ) => NotificationResponse | undefined,
) {
  return _subscribe(
    { type: NotificationType.NOTIFY_ON_KEYBINDING_PRESSED },
    (notification) => {
      switch (notification?.type?.$case) {
        case "keybindingPressedNotification":
          return handler(notification.type.keybindingPressedNotification);
        default:
          break;
      }

      return { unsubscribe: false };
    },
  );
}

export function setInterceptKeystrokes(
  actions: Action[],
  intercept: boolean,
  globalIntercept?: boolean,
  currentTerminalSessionId?: string,
) {
  sendUpdateApplicationPropertiesRequest({
    interceptBoundKeystrokes: intercept,
    interceptGlobalKeystrokes: globalIntercept,
    actionList: { actions },
    currentTerminalSessionId,
  });
}
