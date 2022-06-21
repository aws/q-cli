//
//  API+IPC.swift
//  fig
//
//  Created by Matt Schrage on 11/3/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import FigAPIBindings

extension Local_PostExecHook {
  var historyNotification: Fig_HistoryUpdatedNotification {
    return Fig_HistoryUpdatedNotification.with { notification in
      notification.command = self.command
      notification.exitCode = self.exitCode
      notification.hostname = self.context.hostname
      notification.sessionID = self.context.sessionID
      notification.processName = self.context.processName
      notification.currentWorkingDirectory = self.context.currentWorkingDirectory
    }
  }
}

extension Local_EventHook {
  var eventNotification: Fig_EventNotification {
    return Fig_EventNotification.with { notification in
      notification.eventName = self.eventName
      notification.payload = self.payload
    }
  }
}
