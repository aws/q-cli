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
  static let apiCredentialsPath = "\(NSHomeDirectory())/.fig/tools/iterm-api-credentials"
  static let iTermBundleId = Integrations.iTerm
  
  fileprivate let pollingInterval: TimeInterval = 20
  fileprivate var timer: Timer? = nil
  
  let socket = UnixSocketClient(path: "\(NSHomeDirectory())/Library/Application Support/iTerm2/private/socket", waitForNewline: false)
  let ws = WSFramer(isServer: false)
  fileprivate var sessionId: String? {
    didSet {
      guard let sessionId = sessionId else {
        return
      }
      /// Update current sessionId
//      ShellHooksManager.shared.setTab()
      print("iTerm: sessionId did changed to \(sessionId)")

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
      "x-iterm2-library-version: python 0.24",
      "x-iterm2-cookie: \(cookie)",
      "x-iterm2-key: \(key)",
      "x-iterm2-disable-auth-ui: true"

    ]
    
    return request.joined(separator: CRLF) + CRLF + CRLF
  }
  
  func credentials() -> (String, String)? {
    guard FileManager.default.fileExists(atPath: iTermIntegration.apiCredentialsPath) else {
      return nil
    }
    
    guard let contents = try? String(contentsOfFile: iTermIntegration.apiCredentialsPath, encoding: .utf8) else {
      return nil
    }
    
    
    var allCredentials = contents.split(separator: "\n")
    
    guard allCredentials.count > 0 else {
      print("iTerm: no credentials availible")
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
      print("iTerm: error writing updated credential list to \(iTermIntegration.apiCredentialsPath)")
    }

    return (tokens[0], tokens[1])
    
  }
  
  func attemptToConnect() {
    print("iTerm: attempting to connect! Is iTerm running: \(appIsRunning)")
    if socket.isConnected {
      socket.disconnect()
    }
    
    if appIsRunning, socket.connect() {
        print("iTerm: connected to socket")
        guard let (cookie, key) =  credentials() else {
          print("iTerm: could not find credentials")
          self.socket.disconnect()
          return
        }
      
      print("iTerm: credentials (\(cookie),\(key)")
      
       Timer.delayWithSeconds(1) {
          self.socket.send(message: self.handshakeRequest(cookie: cookie, key: key))
       }
       
     } else {
        print("iTerm: could not connect...")
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
    print("iTerm: frame processed")
    switch event {
    case .frame(let frame):
      let message = try! Iterm2_ServerOriginatedMessage(serializedData: frame.payload)
//      print("iTerm: # of windows = \(message.listSessionsResponse.windows.count)")
      let focusChangedEvent = message.notification.focusChangedNotification
      print("iTerm: focus event! - \(focusChangedEvent.session) ")
      //forEach { print("iTerm: \($0.windowID)") }
    
      let session = focusChangedEvent.session
      if session.count > 0 {
        self.sessionId = session
      }
    case .error(let err):
      print("iTerm: an error occurred - \(err) ")
    }
  }
  
  
}

class iTermEventStream {
  static func notificationRequest() {
    var notificationRequest = Iterm2_NotificationRequest()
    notificationRequest.session = "all"
    notificationRequest.notificationType = .notifyOnFocusChange
    
  }

}

extension iTermIntegration: UnixSocketDelegate {

  func socket(_ socket: UnixSocketClient, didReceive message: String) {
    print("iTerm: recieved message, '\(message)'")
    
    guard !message.contains("HTTP/1.1 401 Unauthorized") else {
      print("iTerm: disconnecting because connection refused")
      socket.disconnect()
      return
    }
    
    //HTTP/1.1 101 Switching Protocols -> Success
    if (message.contains("HTTP/1.1 101 Switching Protocols")) {
      print("iTerm: connection accepted!")
      var message = Iterm2_ClientOriginatedMessage()
      message.id = 0
      
      var notificationRequest = Iterm2_NotificationRequest()
      notificationRequest.session = "all"
      notificationRequest.subscribe = true
      notificationRequest.notificationType = .notifyOnFocusChange
      message.notificationRequest = notificationRequest
      
      let payload = try! message.serializedData()
      let frame = ws.createWriteFrame(opcode: .binaryFrame,
                                      payload: payload,
                                      isCompressed: false)

      socket.send(data: frame)
    }

  }
  
  func socket(_ socket: UnixSocketClient, didReceive data: Data) {
    print("iTerm: recieved data")
    
    ws.add(data: data)
  }
  
  func socketDidClose(_ socket: UnixSocketClient) {
    // schedule attempts to reconnnect
    //attemptToConnectToDocker()
  }
}


extension iTermIntegration: IntegrationProvider {
  static func install(withRestart: Bool, inBackground: Bool, completion: (() -> Void)?) {
    
  }
  
  static var isInstalled: Bool {
    return false
  }
  
  static func promptToInstall(completion: (()->Void)?) {
    
  }
}
