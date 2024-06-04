import {
  SettingsChangedNotification,
  NotificationType,
} from "@fig/fig-api-proto/fig";
import { _subscribe, NotificationResponse } from "./notifications.js";

import {
  sendGetSettingsPropertyRequest,
  sendUpdateSettingsPropertyRequest,
} from "./requests.js";

export const didChange = {
  subscribe(
    handler: (
      notification: SettingsChangedNotification,
    ) => NotificationResponse | undefined,
  ) {
    return _subscribe(
      { type: NotificationType.NOTIFY_ON_SETTINGS_CHANGE },
      (notification) => {
        switch (notification?.type?.$case) {
          case "settingsChangedNotification":
            return handler(notification.type.settingsChangedNotification);
          default:
            break;
        }

        return { unsubscribe: false };
      },
    );
  },
};

export async function get(key: string) {
  return sendGetSettingsPropertyRequest({
    key,
  });
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function set(key: string, value: any): Promise<void> {
  return sendUpdateSettingsPropertyRequest({
    key,
    value: JSON.stringify(value),
  });
}

export async function remove(key: string): Promise<void> {
  return sendUpdateSettingsPropertyRequest({
    key,
  });
}

export async function current() {
  const all = await sendGetSettingsPropertyRequest({});
  return JSON.parse(all.jsonBlob ?? "{}");
}
