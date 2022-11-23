import { SettingsChangedNotification, NotificationType } from './fig.pb';
import { _subscribe } from './notifications';

import {
  sendGetSettingsPropertyRequest,
  sendUpdateSettingsPropertyRequest
} from './requests';

export const didChange = {
  subscribe(
    handler: (notification: SettingsChangedNotification) => boolean | undefined
  ) {
    return _subscribe(
      { type: NotificationType.NOTIFICATION_TYPE_NOTIFY_ON_SETTINGS_CHANGE },
      notification => {
        switch (notification?.type?.$case) {
          case 'settingsChangedNotification':
            return handler(notification.type.settingsChangedNotification);
          default:
            break;
        }

        return false;
      }
    );
  }
};

export async function get(key: string) {
  return sendGetSettingsPropertyRequest({
    key
  });
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function set(key: string, value: any): Promise<void> {
  return sendUpdateSettingsPropertyRequest({
    key,
    value: JSON.stringify(value)
  });
}

export async function remove(key: string): Promise<void> {
  return sendUpdateSettingsPropertyRequest({
    key
  });
}

export async function current() {
  const all = await sendGetSettingsPropertyRequest({});
  return JSON.parse(all.jsonBlob ?? '{}');
}
