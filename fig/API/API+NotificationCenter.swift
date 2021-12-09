//
//  FigNotificationCenter.swift
//  fig
//
//  Created by Matt Schrage on 9/23/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//
import WebKit
import FigAPIBindings
typealias NotificationType = Fig_NotificationType
typealias SubscriberList = Set<WKWebView>

class APINotificationCenter {
    init() {
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(settingsDidChange(notification:)),
                                               name: Settings.settingsUpdatedNotification,
                                               object: nil)
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(processDidChange(notification:)),
                                               name: TTY.processUpdated,
                                               object: nil)
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(focusedWindowDidChange(notification:)),
                                               name: WindowManager.focusedWindowChangedNotification,
                                               object: nil)
    }
    
    deinit {
        NotificationCenter.default.removeObserver(self)
    }
    
    var subscribers: Dictionary<NotificationType, SubscriberList> = [:]
    fileprivate var channels: Dictionary<WKWebView, Dictionary<NotificationType, Int64>> = [:]
    func handleRequest(id: Int64, request: NotificationRequest, for webview: WKWebView) throws -> Bool {
        if request.subscribe {
            try self.subscribe(webview: webview, to: request.type, on: id)
        } else {
            try self.unsubscribe(webview: webview, from: request.type)
        }
        
        return true
    }
    
    fileprivate func getChannel(for webview: WKWebView, type: NotificationType) -> Int64? {
        return self.channels[webview]?[type]
    }
    
    fileprivate func setChannel(_ id: Int64?, for webview: WKWebView, type: NotificationType) {
        
        if let id = id {
            var channelsForWebview = self.channels[webview] ?? [:]
            channelsForWebview[type] = id
            
            self.channels[webview] = channelsForWebview
        } else {
            guard var channelsForWebview = self.channels[webview] else { return }
            
            channelsForWebview.removeValue(forKey: type)
            
            guard channelsForWebview.keys.count == 0 else { return }
            
            channels[webview] = nil

        }

    }
    
    
    func subscribe(webview: WKWebView, to type: NotificationType, on channel: Int64) throws {
        guard type != .all else {
            throw APIError.generic(message: "Cannot subscribe to 'all' notification type.")
        }
        
        var subscribersForType = subscribers[type] ?? SubscriberList()
        
        if subscribersForType.contains(webview) {
            throw APIError.generic(message: "Already subscribed to notification type (\(type.rawValue))")
        }
        
        subscribersForType.insert(webview)
        subscribers[type] = subscribersForType
        setChannel(channel, for: webview, type: type)
    }
    
    func unsubscribe(webview: WKWebView, from type: NotificationType) throws {
        guard type != .all else {
            return unsubscribeAll(webview: webview)
        }
        
        guard var subscribersForType = subscribers[type] else {
            throw APIError.generic(message: "Not subscribed notification type (\(type.rawValue))")
        }
        
        if !subscribersForType.contains(webview) {
            throw APIError.generic(message: "Not subscribed notification type (\(type.rawValue))")
        }
        
        subscribersForType.remove(webview)
        
        subscribers[type] = subscribersForType
        setChannel(nil, for: webview, type: type)

    }
    
    func unsubscribeAll(webview: WKWebView) {
        
        for type in Fig_NotificationType.allCases where type != .all {
            try? unsubscribe(webview: webview, from: type)
        }

    }
    
    func post(notification: Fig_Notification) {

        DispatchQueue.main.async {
          guard let type = notification.notificationType else { return }
          
          let subscribers = self.subscribers[type]
          
          subscribers?.forEach({ webview in
             
              API.send(Response.with({
                  $0.notification = notification
                  $0.id = self.channels[webview]?[type] ?? -1
              }), to: webview, using: .binary)
          })
        }
    }
}

extension APINotificationCenter {
    @objc func settingsDidChange(notification: Notification) {
        self.post(Fig_SettingsChangedNotification.with({ notification in
             if let blob = Settings.shared.jsonRepresentation() {
                 notification.jsonBlob = blob
             }
         }))
    }
    
    @objc func processDidChange(notification: Notification) {
        guard let tty = notification.object as? TTY else { return }
        
        guard let pid = tty.pid, let executable = tty.cmd, let directory = tty.cwd else { return }

        self.post(Fig_ProcessChangedNotification.with({ notification in
            notification.newProcess = Fig_Process.with({ process in
                process.pid = pid
                process.executable = executable
                process.directory = directory
            })
            
            notification.sessionID = tty.descriptor
        }))
    }
    
    @objc func focusedWindowDidChange(notification: Notification) {
        guard let window = notification.object as? ExternalWindow else { return }

        self.post(Fig_WindowFocusChangedNotification.with({ notification in
            notification.window = window.fig_window
         }))
    }
}

// Must be updated when new notifications are added!
extension Fig_Notification {
    var notificationType: NotificationType? {
        switch self.type {
        case .editBufferNotification(_):
            return .notifyOnEditbuffferChange
        case .keybindingPressedNotification(_):
            return .notifyOnKeybindingPressed
        case .locationChangedNotification(_):
            return .notifyOnLocationChange
        case .processChangeNotification(_):
            return .notifyOnProcessChanged
        case .shellPromptReturnedNotification(_):
            return .notifyOnPrompt
        case .settingsChangedNotification(_):
            return .notifyOnSettingsChange
        case .windowFocusChangedNotification(_):
            return .notifyOnFocusChanged
        case .historyUpdatedNotification(_):
            return .notifyOnHistoryUpdated
        case .none:
            return nil
        }
    }
}


// Convience methods for posting Notifications
extension APINotificationCenter {
    func post(_ notification: Fig_EditBufferChangedNotification) {
        var wrapper = Fig_Notification()
        wrapper.editBufferNotification = notification
        self.post(notification: wrapper)
    }
    
    func post(_ notification: Fig_ProcessChangedNotification) {
        var wrapper = Fig_Notification()
        wrapper.processChangeNotification = notification
        self.post(notification: wrapper)
    }
    
    func post(_ notification: Fig_ShellPromptReturnedNotification) {
        var wrapper = Fig_Notification()
        wrapper.shellPromptReturnedNotification = notification
        self.post(notification: wrapper)
    }

    func post(_ notification: Fig_SettingsChangedNotification) {
        var wrapper = Fig_Notification()
        wrapper.settingsChangedNotification = notification
        self.post(notification: wrapper)
        
    }
    
    func post(_ notification: Fig_LocationChangedNotification) {
        
    }
    
    func post(_ notification: Fig_KeybindingPressedNotification) {
        var wrapper = Fig_Notification()
        wrapper.keybindingPressedNotification = notification
        self.post(notification: wrapper)
    }
    
    func post(_ notification: Fig_WindowFocusChangedNotification) {
        var wrapper = Fig_Notification()
        wrapper.windowFocusChangedNotification = notification
        self.post(notification: wrapper)
    }
  
    func post(_ notification: Fig_HistoryUpdatedNotification) {
        var wrapper = Fig_Notification()
        wrapper.historyUpdatedNotification = notification
        self.post(notification: wrapper)
    }
}

extension APINotificationCenter {
  func editbufferChanged(buffer: String, cursor: Int, session: SessionId, context: Local_ShellContext? = nil) {
    
    API.notifications.post(
      Fig_EditBufferChangedNotification.with({ notification in
        notification.buffer = buffer
        notification.cursor = Int32(cursor)
        notification.sessionID = session
        
        if let context = context {
          notification.context = context
        }
      })
    )
  }
}
