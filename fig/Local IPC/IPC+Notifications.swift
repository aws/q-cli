//
//  IPC+Notifications.swift
//  fig
//
//  Created by Matt Schrage on 11/29/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

extension IPC {
//  static let promptHookNotification = Notification.Name(rawValue: "promptHookNotification")
//  static let preExecHookNotification = Notification.Name(rawValue: "preExecHookNotification")
//  static let postExecHookNotification = Notification.Name(rawValue: "postExecHookNotification")
//  static let initializeHookNotification = Notification.Name(rawValue: "initializeHookNotification")
//  static let editbufferHookNotification = Notification.Name(rawValue: "editbufferHookNotification")

  enum Notifications: String {
    case prompt = "promptHookNotification"
    case preExec = "preExecHookNotification"
    case postExec = "postExecHookNotification"
    case initialize = "initializeHookNotification"
    case editBuffer = "editbufferHookNotification"
    case keyboardFocusChanged = "keyboardFocusChangedHookNotification"

    var notification: Notification.Name {
      return Notification.Name(rawValue: self.rawValue)
    }
  }
  static func post(notification: IPC.Notifications, object: Any?) {
    NotificationCenter.default.post(name: notification.notification, object: object)
  }

}
