import { EditBufferChangedNotification, NotificationType } from "./fig";
import { _subscribe } from "./notifications";

const subscribe = (handler: (notification: EditBufferChangedNotification) => boolean | undefined) => {
    return _subscribe({ type: NotificationType.NOTIFY_ON_EDITBUFFFER_CHANGE }, (notification) => {
        switch (notification?.type?.$case) {
            case "editBufferNotification":
                return handler(notification.type.editBufferNotification)
            default:
                break;
        }

        return false
    })
}
const EditBufferNotifications = { subscribe };

export default EditBufferNotifications;
