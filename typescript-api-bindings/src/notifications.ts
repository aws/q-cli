import {
    Notification,
    ServerOriginatedMessage,
    NotificationRequest,
    NotificationType
  } from "./fig";
  
import { sendMessage } from "./core"

export type NotificationHandler = (notification: Notification) => boolean | undefined;
export interface Subscription {
  unsubscribe() : void
}

const handlers: Partial<Record<NotificationType, NotificationHandler[]>> = {};

export const _unsubscribe = (type: NotificationType, handler?: NotificationHandler) => {
  if (handler && handlers[type] !== undefined) {
    handlers[type] = (handlers[type] ?? []).filter(x => x !== handler);
  }
}

export const _subscribe = (request: NotificationRequest, handler: NotificationHandler): Promise<Subscription> | undefined =>  {
  return new Promise<Subscription>((resolve, reject) => {

  const type = request.type

  if (!type) {
     return reject("NotificationRequest type must be defined.")
  }

  const addHandler = () => {
    handlers[type] = [...(handlers[type] ?? []), handler]
    resolve({ unsubscribe: () => _unsubscribe(type, handler) });
  }

  // primary subscription already exists
  if (handlers[type] !== undefined) {
    return addHandler();
  }

  handlers[type] = [];

  request.subscribe = true

    sendMessage({ $case: "notificationRequest", notificationRequest: request },
      (response: ServerOriginatedMessage["submessage"]) => {
        switch (response?.$case) {
          case "notification":
            if (!handlers[type]) {
              return false
            }

            // call handlers and remove any that have unsubscribed (by returning false)
            const handlersToRemove = handlers[type]?.filter(
              handler => handler(response.notification) === false
            );

            handlers[type] = handlers[type]?.filter(
              handler => !handlersToRemove?.includes(handler)
            );

            return true
          case "success":
            addHandler();
            return true
          case "error":
            reject(response.error);
            break;
          default:
            reject("Not a notification");
            break;
        }

        return false
      })
    })
}

const unsubscribeFromAll = () => {
  sendMessage({ $case: "notificationRequest", notificationRequest: {
    subscribe: false,
    type: NotificationType.ALL
  }})
}

console.log("[fig] unsubscribing any existing notifications...")
unsubscribeFromAll()
