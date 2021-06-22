//
//  iTermIntegration.swift
//  fig
//
//  Created by Matt Schrage on 6/9/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class iTermIntegration {
  static let shared = iTermIntegration()
  static let iTermBundleId = Integrations.iTerm
  
  // Installation
  fileprivate static let scriptName = "fig-iterm-integration"

  fileprivate static let iTermAutoLaunchDirectory = "\(NSHomeDirectory())/Library/Application Support/iTerm2/Scripts/AutoLaunch/"
  fileprivate static let autoLaunchScriptTarget = iTermAutoLaunchDirectory + scriptName + ".scpt"
  static let bundleAppleScriptFilePath = Bundle.main.path(forResource: scriptName, ofType: "scpt")!
  // Do we want to store the Applescript in the bundle or in withfig/fig? eg. "\(NSHomeDirectory())/.fig/tools/\(scriptName).scpt"
  fileprivate static let plistVersionKey = "iTerm Version"
  fileprivate static let plistAPIEnabledKey = "EnableAPIServer"
  fileprivate static let minimumSupportedVersion:[Int] = [3,3,0]
  
  
  fileprivate static let legacyIntegrationPath = iTermAutoLaunchDirectory + scriptName + ".py"

  // API
  var isConnectedToAPI = false {
    didSet {
      if isConnectedToAPI {
        // Remove legacy integration!
        try? FileManager.default.removeItem(at: URL(fileURLWithPath: iTermIntegration.legacyIntegrationPath))
      }
    }
  }
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
          ShellHookManager.shared.keyboardFocusDidChange(to: sessionId, in: window)

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
    
  }
    
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

    guard appIsRunning else {
      Logger.log(message: "target app is not running...", subsystem: .iterm)
      return
    }
    
    if socket.connect() {
        Logger.log(message: "connected to socket", subsystem: .iterm)

        guard let (cookie, key) =  credentials() else {
          Logger.log(message: "could not find credentials", subsystem: .iterm)

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
      
  static func install(withRestart: Bool, inBackground: Bool, completion: (() -> Void)? = nil) {
    guard NSWorkspace.shared.applicationIsInstalled(iTermBundleId) else {
      completion?()
      return
    }
    // Check version number
    guard let iTermDefaults = UserDefaults(suiteName: iTermBundleId),
          let version = iTermDefaults.string(forKey: plistVersionKey) else {
      if !inBackground {
        // Alert that version couldn't be found
        Alert.show(title: "Could not install iTerm Integration",
                   message: "Could not read iTerm plist file to determine version")
      }
      completion?()
      return
    }
    
    // Version Check
    let semver = version.split(separator: ".").map { Int($0) }
    guard semver.count == 3 else {
      if !inBackground {
        Alert.show(title: "Could not install iTerm Integration",
                   message: "iTerm version (\(version)) was invalid")
      }
      completion?()
      return
    }
    
    for idx in 0...2 {
      if semver[idx]! < minimumSupportedVersion[idx] {
        if !inBackground {
          Alert.show(title: "Could not install iTerm Integration",
                     message: "iTerm version \(version) is not supported. Must be 3.3.0 or above.")
        }
        completion?()
        return
      }
      
      if semver[idx]! > minimumSupportedVersion[idx] {
        break
      }
    }
    
    // Update API preferences
    iTermDefaults.setValue(true, forKey: plistAPIEnabledKey)
    iTermDefaults.synchronize()
    
    // Create directory if it does not exist.
    try? FileManager.default.createDirectory(at: iTermAutoLaunchDirectory,
                                             withIntermediateDirectories: true,
                                             attributes: nil)
    
    try? FileManager.default.createSymbolicLink(atPath: autoLaunchScriptTarget,
                                                withDestinationPath: bundleAppleScriptFilePath)
  
    let destination = try? FileManager.default.destinationOfSymbolicLink(atPath: autoLaunchScriptTarget)
    
    // Check if symlink exists and is pointing to the correct location
    guard destination == bundleAppleScriptFilePath else {
      if !inBackground {
        Alert.show(title: "Could not install iTerm Integration",
                   message: "Could not create symlink to '\(autoLaunchScriptTarget)'")
      }
      completion?()
      return
      
    }
    
    
    guard withRestart else {
      completion?()
      return
    }
    
    let iTerm = Restarter(with: iTermBundleId)
    iTerm.restart(launchingIfInactive: false, completion: completion)
    
  }
  
  static var isInstalled: Bool {
    guard let symlinkDestination = try? FileManager.default.destinationOfSymbolicLink(atPath: iTermIntegration.autoLaunchScriptTarget) else {
      return false
    }
    
    return symlinkDestination == bundleAppleScriptFilePath
  }
  
  static func promptToInstall(completion: (()->Void)? = nil) {
    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: iTermBundleId) else {
      completion?()
      return
    }
    
    let icon = NSImage(imageLiteralResourceName: "NSSecurity")

    let app = NSWorkspace.shared.icon(forFile: url.path)
    
    let shouldInstall = Alert.show(title: "Install iTerm Integration?",
                                 message: "Fig will add a plugin to iTerm that tracks which terminal session is active.\n\n",
                                 okText: "Install plugin",
                                 icon: icon.overlayImage(app),
                                 hasSecondaryOption: true)
    
    if shouldInstall {
      install(withRestart: true,
              inBackground: false,
              completion: completion)
    }
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

extension NSWorkspace {
  func applicationIsInstalled(_ bundleId: String?) -> Bool {
    guard let bundleId = bundleId else {
      return false
    }
    
    return  NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleId) != nil
  }
}
