//
//  Notifications.swift
//  fig
//
//  Created by Matt Schrage on 9/21/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import FigAPIBindings
import WebKit

class APINotifications {
    
    // todo(mschrage): implement a better system to send notifications to multiple different fig app
    static func post(_ notification: Fig_Notification) {
        
        let allCompanionWindows = Set( WindowManager.shared.windows.map { $0.value }).union(WindowManager.shared.untetheredWindows).union([ WindowManager.shared.autocomplete])

        
        allCompanionWindows.forEach { companion in
            if let webview = WindowManager.shared.autocomplete?.webView {
                API.send(Response.with({
                    $0.notification = notification
                    $0.id = -1
                }), to: webview)
            }
        }
    }
    
    
}

class InternalNotificationsAdapter {
    static let shared = InternalNotificationsAdapter()
    
    init() {
        NotificationCenter.default.addObserver(forName: Settings.settingsUpdatedNotification, object: self, queue: nil) { _ in
            API.notifications.post(Fig_SettingsChangedNotification.with({ notification in
                if let blob = Settings.shared.jsonRepresentation() {
                    notification.jsonBlob = blob
                }
            }))
        }
        
        NotificationCenter.default.addObserver(forName: TTY.processUpdated, object: self, queue: nil) { sender in
            guard let tty = sender.object as? TTY else { return }
            
            guard let pid = tty.pid, let executable = tty.cmd, let directory = tty.cwd else { return }

            API.notifications.post(Fig_ProcessChangedNotification.with({ notification in
                notification.newProcess = Fig_Process.with({ process in
                    process.pid = pid
                    process.executable = executable
                    process.directory = directory
                })
                
                notification.sessionID = tty.descriptor
            }))
        }
    }
    
    deinit {
        NotificationCenter.default.removeObserver(self)
    }
}
