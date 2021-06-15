//
//  iTermIntegration.swift
//  fig
//
//  Created by Matt Schrage on 6/9/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
//import Starscream
class iTermIntegration {
  static let shared = iTermIntegration()
  static let iTermBundleId = Integrations.iTerm
  
  // Installation
  static let autolaunchAuthenticationAppleScript = "\(NSHomeDirectory())/.fig/tools/fig-iterm-integration.scpt"
  static var iTerm: NSRunningApplication? = nil
  static var kvo: NSKeyValueObservation? = nil
  
  // API
  var isConnectedToAPI = false
  static let apiCredentialsPath = "\(NSHomeDirectory())/.fig/tools/iterm-api-credentials"
  let socket = UnixSocketClient(path: "\(NSHomeDirectory())/Library/Application Support/iTerm2/private/socket", waitForNewline: false)
  let ws = WSFramer(isServer: false)
  
  fileprivate var sessionId: String? {
    didSet {
      guard let sessionId = sessionId else {
        return
      }

      Logger.log(message: "sessionId did changed to \(sessionId)", subsystem: .iterm)

      if let window = AXWindowServer.shared.whitelistedWindow, window.bundleId ?? "" == iTermIntegration.iTermBundleId {
        ShellHookManager.shared.setActiveTab(sessionId, for: window.windowId)

      }

    }
  }
  var currentSessionId: String? {
    get {
      guard appIsInstalled, self.socket.isConnected else  {
        return nil
      }
      
      return self.sessionId
    }
  }
  
  
  init() {
    
    guard self.appIsInstalled else {
      print("iTerm is not installed.")
      return
    }
    
    socket.delegate = self
    ws.register(delegate: self)
    
    NSWorkspace.shared.notificationCenter.addObserver(self,
                                                      selector: #selector(didTerminateApplication),
                                                      name: NSWorkspace.didTerminateApplicationNotification,
                                                      object: nil)
    
    self.attemptToConnect()
    
//    self.timer = Timer.scheduledTimer(withTimeInterval: self.pollingInterval, repeats: true) { _ in
//      print("iTerm: should attempt to connect?")
//      guard self.appIsRunning else {
//        print("iTerm: desktop app is not running, so disconnecting from socket.")
//        self.socket.disconnect()
//        return
//      }
//      guard !self.socket.isConnected else {
//        print("iTerm: Socket is already connected")
//        return
//      }
//
//      self.attemptToConnect()
//    }
    
    
  }
  
  // potentially add file watcher for ~/.fig/tools/iterm-api-credentials
  
  // https://gitlab.com/gnachman/iterm2/-/issues/9058#note_392824614
  // https://github.com/gnachman/iTerm2/blob/c52136b7c0bae545436be8d1441449f19e21faa1/sources/iTermWebSocketConnection.m#L50
  static let iTermLibraryVersion = 0.24
  fileprivate func handshakeRequest(cookie: String, key: String) -> String {
    let CRLF = "\r\n"

    let request = [
      "GET / HTTP/1.1",
      "connection: Upgrade",
      "upgrade: websocket",
      "origin: ws://localhost/",
      "host: localhost",
      "sec-websocket-protocol: api.iterm2.com",
      "sec-websocket-version: 13",
      "sec-websocket-key: \(key)",
      "x-iterm2-library-version: python \(iTermIntegration.iTermLibraryVersion)",
      "x-iterm2-cookie: \(cookie)",
      "x-iterm2-key: \(key)",
      "x-iterm2-disable-auth-ui: true"

    ]
    
    return request.joined(separator: CRLF) + CRLF + CRLF
  }
  
  @objc func didTerminateApplication(notification: Notification) {
    guard let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication else {
      return
    }
    
    guard app.bundleIdentifier == iTermIntegration.iTermBundleId else {
      return
    }
    
    Logger.log(message: "disconnecting socket because application was terminated.", subsystem: .iterm)
//    self.socket.disconnect()
    self.disconnect()
    
  }
  
  func credentials() -> (String, String)? {
    guard FileManager.default.fileExists(atPath: iTermIntegration.apiCredentialsPath) else {
      Logger.log(message: "credentials file does not exist - this is likely because Fig is newly installed and iTerm has not restarted yet!", subsystem: .iterm)
      return nil
    }
    
    guard let contents = try? String(contentsOfFile: iTermIntegration.apiCredentialsPath, encoding: .utf8) else {
      return nil
    }
    
    
    var allCredentials = contents.split(separator: "\n")
    
    guard allCredentials.count > 0 else {
      Logger.log(message: "no credentials availible!", subsystem: .iterm)
      return nil
    }
    
    let currentCredentials = allCredentials.removeFirst()
    let tokens = currentCredentials.split(separator: " ").map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }
    guard tokens.count == 2 else {
      return nil
    }
    
    let updatedCredentialsList = allCredentials.joined(separator: "\n")
    do {
      try updatedCredentialsList.write(to: URL(fileURLWithPath: iTermIntegration.apiCredentialsPath),
                                       atomically: true,
                                       encoding: .utf8)
    } catch {
      Logger.log(message: "error writing updated credential list to \(iTermIntegration.apiCredentialsPath)", subsystem: .iterm)
    }

    return (tokens[0], tokens[1])
    
  }
  
  func disconnect() {
    self.socket.disconnect()
    self.ws.reset()
    self.isConnectedToAPI = false

  }
  
  func attemptToConnect() {
    Logger.log(message: "attempting to connect!", subsystem: .iterm)

//    if socket.isConnected {
//      socket.disconnect()
//    }
    guard appIsRunning else {
      Logger.log(message: "target app is not running...", subsystem: .iterm)
      return
    }
    
    if socket.connect() {
        Logger.log(message: "connected to socket", subsystem: .iterm)

        guard let (cookie, key) =  credentials() else {
          Logger.log(message: "could not find credentials", subsystem: .iterm)

//          self.socket.disconnect()
          self.disconnect()
          return
        }
      
        Logger.log(message: "Sending websocket handshake", subsystem: .iterm)

      
       Timer.delayWithSeconds(1) {
          self.socket.send(message: self.handshakeRequest(cookie: cookie, key: key))
        Logger.log(message: "Sent websocket handshake!", subsystem: .iterm)

       }
       
     } else {
        Logger.log(message: "Already connected...", subsystem: .iterm)
     }
  }

  var appIsInstalled: Bool {
    return NSWorkspace.shared.urlForApplication(withBundleIdentifier: iTermIntegration.iTermBundleId) != nil
  }
  
  var appIsRunning: Bool {
    return NSWorkspace.shared.runningApplications.contains { $0.bundleIdentifier == iTermIntegration.iTermBundleId }
  }
}

extension iTermIntegration: FramerEventClient {
  func frameProcessed(event: FrameEvent) {
//    Logger.log(message: "frame processed", subsystem: .iterm)
    switch event {
    case .frame(let frame):
      let message = try! Iterm2_ServerOriginatedMessage(serializedData: frame.payload)
      
      guard message.error.count == 0 else {
        Logger.log(message: "API error - \(message.error)", subsystem: .iterm)
        return
        
      }
      
      guard !message.notificationResponse.hasStatus else {
        Logger.log(message: "notification response \(message.notificationResponse.status)", subsystem: .iterm)
        return
      }
      
      if message.notification.hasFocusChangedNotification {
        let focusChangedEvent = message.notification.focusChangedNotification
        Logger.log(message: "focus event! - \(focusChangedEvent.session)", subsystem: .iterm)
      
        let session = focusChangedEvent.session
        if session.count > 0 {
          self.sessionId = session
        }
        
        return
      }
      
      
      print("iterm: other API event")

    case .error(let err):
      Logger.log(message: "an error occurred - \(err)", subsystem: .iterm)
    }
  }
  
  
}

class iTermEventStream {
  static func notificationRequest() -> Iterm2_ClientOriginatedMessage {
    var message = Iterm2_ClientOriginatedMessage()
    message.id = 0
    
    var notificationRequest = Iterm2_NotificationRequest()
    notificationRequest.session = "all"
    notificationRequest.subscribe = true
    notificationRequest.notificationType = .notifyOnFocusChange
    message.notificationRequest = notificationRequest
    
    return message
  }

}

extension iTermIntegration: UnixSocketDelegate {

  func socket(_ socket: UnixSocketClient, didReceive message: String) {
    Logger.log(message: "recieved message, '\(message)'", subsystem: .iterm)

    guard !message.contains("HTTP/1.1 401 Unauthorized") else {
      Logger.log(message: "disconnecting because connection refused", subsystem: .iterm)
      
      self.disconnect()
      return
    }
    
    //HTTP/1.1 101 Switching Protocols -> Success
    if (message.contains("HTTP/1.1 101 Switching Protocols")) {
      Logger.log(message: "connection accepted!", subsystem: .iterm)
      self.isConnectedToAPI = true
      let message = iTermEventStream.notificationRequest()
      
      let payload = try! message.serializedData()
      let frame = ws.createWriteFrame(opcode: .binaryFrame,
                                      payload: payload,
                                      isCompressed: false)

      socket.send(data: frame)
    }

  }
  
  func socket(_ socket: UnixSocketClient, didReceive data: Data) {
    Logger.log(message: "recieved data", subsystem: .iterm)
    ws.add(data: data)
  }
  
  func socketDidClose(_ socket: UnixSocketClient) { }
}


extension iTermIntegration: IntegrationProvider {
      
  static func install(withRestart: Bool, inBackground: Bool, completion: (() -> Void)?) {
    
    // Update API preferences
    
    
    guard withRestart else {
      completion?()
      return
    }
    
    if let app = NSRunningApplication.forBundleId(iTermBundleId) {
       self.iTerm = app
       self.iTerm!.terminate()
       self.kvo = self.iTerm!.observe(\.isTerminated, options: .new) { (app, terminated) in
           if terminated.newValue == true {
               print("iTerm terminated! Restarting...")
                NSWorkspace.shared.launchApplication(withBundleIdentifier: iTermBundleId,
                                                     options: [.default],
                                                     additionalEventParamDescriptor: nil,
                                                     launchIdentifier: nil)
               self.kvo!.invalidate()
               self.iTerm = nil
           }
       }
    } else {
       NSWorkspace.shared.launchApplication(withBundleIdentifier: iTermBundleId,
                                            options: [.default],
                                            additionalEventParamDescriptor: nil,
                                            launchIdentifier: nil)
   }
    
  }
  
  static var isInstalled: Bool {
    return FileManager.default.fileExists(atPath: iTermIntegration.autolaunchAuthenticationAppleScript)
  }
  
  static func promptToInstall(completion: (()->Void)?) {
//    Alert.show(title: <#T##String#>, message: <#T##String#>)
  }
}


extension NSRunningApplication {
  
  static func forBundleId(_ bundleId: String?) -> NSRunningApplication? {
    guard let bundleId = bundleId else {
      return nil
    }

    return NSWorkspace.shared.runningApplications.filter ({ return $0.bundleIdentifier == bundleId }).first

  }
}
