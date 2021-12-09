import { SettingsChangedNotification, NotificationType } from './fig';
import { _subscribe } from './notifications';

import {
  sendGetSettingsPropertyRequest,
  sendUpdateSettingsPropertyRequest,
} from './requests';

export const didChange = {
  subscribe: (
    handler: (notification: SettingsChangedNotification) => boolean | undefined
  ) => {
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
  }
};

export const get = async (key: string) =>
  sendGetSettingsPropertyRequest({
    key: key,
  });

export const set = async (key: string, value: any): Promise<void> =>
  sendUpdateSettingsPropertyRequest({
    key: key,
    value: JSON.stringify(value),
  });

export const remove = async (key: string): Promise<void> =>
  sendUpdateSettingsPropertyRequest({
    key: key,
  });

export const current = async () => {
  let all = await sendGetSettingsPropertyRequest({});
  return JSON.parse(all.jsonBlob ?? '{}');
};
