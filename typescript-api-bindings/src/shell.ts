import { NotificationType,ProcessChangedNotification, ShellPromptReturnedNotification } from "./fig";
import { sendInsertTextRequest } from './requests'
import { _subscribe } from "./notifications";

const subscribeToProcessChangedNotifications = (handler: (notification: ProcessChangedNotification) => boolean | undefined) => {
    return _subscribe({ type: NotificationType.NOTIFY_ON_PROCESS_CHANGED }, (notification) => {
        switch (notification?.type?.$case) {
            case "processChangeNotification":
                return handler(notification.type.processChangeNotification)
            default:
                break;
        }

        return false
    })
}

const subscribeToPromptReturnedNotifications = (handler: (notification: ShellPromptReturnedNotification) => boolean | undefined) => {
    return _subscribe({ type: NotificationType.NOTIFY_ON_PROMPT }, (notification) => {
        switch (notification?.type?.$case) {
            case "shellPromptReturnedNotification":
                return handler(notification.type.shellPromptReturnedNotification)
            default:
                break;
        }

        return false
    })
}

const insert =  async (text: string) =>
    sendInsertTextRequest({
        type: { $case: "text", text: text}
    });

const promptDidReturn = { subscribe: subscribeToPromptReturnedNotifications}
const processDidChange = { subscribe: subscribeToProcessChangedNotifications };

const Shell = { processDidChange, promptDidReturn, insert };

export default Shell;
