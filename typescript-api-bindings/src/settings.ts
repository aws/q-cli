import { SettingsChangedNotification, NotificationType } from './fig';
import { _subscribe } from './notifications';

import {
  sendGetSettingsPropertyRequest,
  sendUpdateSettingsPropertyRequest,
} from './requests';

export const didChange = {
  subscribe(
    handler: (notification: SettingsChangedNotification) => boolean | undefined
  ) {
    return _subscribe(
      { type: NotificationType.NOTIFY_ON_SETTINGS_CHANGE },
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
  },
};

export async function get(key: string) {
  return sendGetSettingsPropertyRequest({
    key: key,
  });
}

export async function set(key: string, value: any): Promise<void> {
  return sendUpdateSettingsPropertyRequest({
    key: key,
    value: JSON.stringify(value),
  });
}

export async function remove(key: string): Promise<void> {
  return sendUpdateSettingsPropertyRequest({
    key: key,
  });
}

export async function current() {
  let all = await sendGetSettingsPropertyRequest({});
  return JSON.parse(all.jsonBlob ?? '{}');
}
