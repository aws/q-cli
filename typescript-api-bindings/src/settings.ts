import { SettingsChangedNotification, NotificationType } from "./fig";
import { _subscribe } from "./notifications";

import { sendGetSettingsPropertyRequest, sendUpdateSettingsPropertyRequest } from "./requests"
const subscribe = (handler: (notification: SettingsChangedNotification) => boolean | undefined) => {
    _subscribe({ type: NotificationType.NOTIFY_ON_SETTINGS_CHANGE }, (notification) => {
        switch (notification?.type?.$case) {
            case "settingsChangedNotification":
                return handler(notification.type.settingsChangedNotification)
            default:
                break;
        }

        return false
    })
}

const get = async (
  key: string,
) =>
  sendGetSettingsPropertyRequest({
    key: key,
  });

const set = async (
  key: string,
  value: any
): Promise<void> =>
    sendUpdateSettingsPropertyRequest({
    key: key,
    value: JSON.stringify(value)
  });

  const remove = async (
    key: string,
  ): Promise<void> =>
      sendUpdateSettingsPropertyRequest({
      key: key,
    });

const didChange = { subscribe }
const Settings = { didChange, get, set, remove };

export default Settings;
