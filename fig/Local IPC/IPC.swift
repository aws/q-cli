//
//  ShellHooksTransport.swift
//  fig
//
//  Created by Matt Schrage on 4/8/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import Socket
import FigAPIBindings
import Cocoa

typealias LocalMessage = Local_LocalMessage
typealias CommandResponse = Local_CommandResponse
// swiftlint:disable type_body_length
// swiftlint:disable type_name
class IPC: UnixSocketServerDelegate {

  enum Encoding: String {
    case binary = "pbuf"
    case json = "json"

    var type: String {
      return self.rawValue
    }

    var typeBytes: Data {
      return self.rawValue.data(using: .utf8)!
    }

    static var typeSize: Int {
      return 4
    }

    static var headerPrefix: Data {
      return "\u{1B}@fig-".data(using: .utf8)!
    }
    // \efig-(pbuf|json)
    static var headerSize: Int {
      return headerPrefix.count + typeSize + 8
    }
  }

  static let unixSocket: URL = URL(fileURLWithPath: "/var/tmp/fig/\(NSUserName())/fig.socket")

  static let shared = IPC()
  fileprivate var buffer: Data = Data()
  fileprivate let legacyServer = UnixSocketServer(
    path: FileManager.default.temporaryDirectory.appendingPathComponent("fig.socket").path,
    bidirectional: true)
  fileprivate let server = UnixSocketServer(
    path: unixSocket.path,
    bidirectional: true)
  init() {
    legacyServer.delegate = self
    legacyServer.run()

    server.delegate = self
    server.run()

    // Prevent "App Nap" from automatically killing Fig if the computer goes to sleep
    // while the user has disabled the menubar icon
    // See: https://stackoverflow.com/questions/19577541/disabling-timer-coalescing-in-osx-for-a-given-process
    ProcessInfo.processInfo.disableAutomaticTermination(
      "Running unix socket server to handle updates from active terminal sessions."
    )
  }

  func recieved(data: Data, on socket: Socket?) {
    guard let socket = socket,
          let (message, encoding) = try? retriveMessage(rawBytes: data)
    else { return }

    do {
      try self.handle(message, from: socket, using: encoding)
    } catch {
      Logger.log(
        message: "Error handling IPC message: \(error.localizedDescription)",
        subsystem: .unix)
    }
  }

  func recieved(string: String, on socket: Socket?) {}

  // send a response to a socket that conforms to the IPC protocol
  func send(_ response: CommandResponse, to socket: Socket, encoding: IPC.Encoding) throws {
    var data: Data!
    switch encoding {
    case .binary:
      data = try response.serializedData()
    case .json:
      let json = try response.jsonString()
      data = json.data(using: .utf8)
    }

    try socket.write(from: "\u{001b}@fig-\(encoding.type)")
    try socket.write(from: Data(from: Int64(data.count).bigEndian))
    try socket.write(from: data)

  }

  // attempt to decode the bytes as a packet, if not possible add to buffer
  func retriveMessage(rawBytes: Data) throws -> (LocalMessage, IPC.Encoding)? {
    //    buffer.append(rawBytes)

    var header = rawBytes.subdata(in: 0...IPC.Encoding.headerSize)

    guard header.starts(with: IPC.Encoding.headerPrefix) else {
      return nil
    }

    header = header.advanced(by: IPC.Encoding.headerPrefix.count)

    let type = header.subdata(in: 0..<IPC.Encoding.typeSize)
    let encoding: IPC.Encoding!
    switch type {
    case IPC.Encoding.binary.typeBytes:
      encoding = .binary
    case IPC.Encoding.json.typeBytes:
      encoding = .json
    default:
      return nil
    }

    header = header.advanced(by: IPC.Encoding.typeSize)

    let packetSizeData = header.subdata(in: 0..<8)
    guard let packetSizeLittleEndian = packetSizeData.to(type: Int64.self) else {
      return nil
    }

    let packetSize = Int64(bigEndian: packetSizeLittleEndian)

    guard packetSize <= rawBytes.count - IPC.Encoding.headerSize && packetSize >= 0 else {
      return nil
    }

    let message = rawBytes.subdata(in: IPC.Encoding.headerSize...IPC.Encoding.headerSize + Int(packetSize))

    switch encoding {
    case .binary:
      return (try LocalMessage(serializedData: message), encoding!)
    case .json:
      guard let json = String(data: message, encoding: .utf8) else {
        return nil
      }
      return (try LocalMessage(jsonString: json), encoding!)
    case .none:
      return nil
    }

  }

  func handle(_ message: LocalMessage, from socket: Socket, using encoding: IPC.Encoding) throws {

    switch message.type {
    case .command(let command):
      try handleCommand(command, from: socket, using: encoding)
    case .hook(let hook):
      DispatchQueue.main.async {
        self.handleHook(hook)
      }
    case .none:
      break

    }
  }

  func handleCommand(_ message: Local_Command, from socket: Socket, using encoding: IPC.Encoding)
  throws {

    Logger.log(message: "Recieved command message!", subsystem: .unix)

    let json = try? message.jsonString()
    Logger.log(message: json ?? "Could not decode message", subsystem: .unix)

    let messageId = message.id
    var response: CommandResponse?

    switch message.command {
    case .terminalIntegration(let request):
      response = try Integrations.providers[request.identifier]?.handleIntegrationRequest(request)
    case .listTerminalIntegrations:
      response = Integrations.handleListIntegrationsRequest()
    case .logout:
      response = CommandHandlers.logoutCommand()
    case .restart:
      CommandHandlers.restartCommand()
    case .quit:
      CommandHandlers.quitCommand()
    case .update(let request):
      CommandHandlers.updateCommand(request.force)
    case .diagnostics:
      response = CommandHandlers.diagnosticsCommand()
    case .reportWindow(let request):
      CommandHandlers.displayReportWindow(message: request.report,
                                          path: request.path,
                                          figEnvVar: request.figEnvVar,
                                          terminal: request.terminal)
    case .restartSettingsListener:
      response = CommandHandlers.restartSettingsListenerCommand()
    case .runInstallScript:
      response = CommandHandlers.runInstallScriptCommand()
    case .build(let request):
      response = CommandHandlers.buildCommand(build: request.branch)
    case .openUiElement(let request):
      response = CommandHandlers.openUiElement(uiElement: request.element)
    case .resetCache:
      response = CommandHandlers.resetCache()
    case .debugMode(let request):
      response = CommandHandlers.autocompleteDebugMode(
        setVal: request.hasSetDebugMode ? request.setDebugMode : nil,
        toggleVal: request.hasToggleDebugMode ? request.toggleDebugMode : nil)
    case .promptAccessibility:
      CommandHandlers.promptAccessibility()
    case .inputMethod(let request):
      response = CommandHandlers.inputMethod(request)
    case .uninstall:
      DispatchQueue.main.sync {
        NSApp.appDelegate.uninstall(showDialog: false)
      }
    case .none:
      break
    }

    guard !message.noResponse else { return }

    if var resp = response {
      resp.id = messageId
      try self.send(resp, to: socket, encoding: encoding)
    }
  }

  func handleHook(_ message: Local_Hook) {
    Logger.log(message: "Recieved hook message!", subsystem: .unix)

    let json = try? message.jsonString()
    Logger.log(message: json ?? "Could not decode message", subsystem: .unix)

    switch message.hook {
    case .editBuffer(let hook):
      // NOTE: IPC notifications update TerminalSessionLinker and MUST occur before everything else!
      IPC.post(notification: .editBuffer, object: hook)

      ShellHookManager.shared.updateKeybuffer(
        context: hook.context.activeContext(),
        text: hook.text,
        cursor: Int(hook.cursor),
        histno: Int(hook.histno))
    case .init_p(let hook):
      IPC.post(notification: .initialize, object: hook)

      ShellHookManager.shared.startedNewTerminalSession(
        context: hook.context.activeContext(),
        calledDirect: hook.calledDirect,
        bundle: hook.bundle,
        env: hook.env)
    case .prompt(let hook):
      IPC.post(notification: .prompt, object: hook)

      ShellHookManager.shared.shellPromptWillReturn(context: hook.context.activeContext())
    case .preExec(let hook):
      IPC.post(notification: .preExec, object: hook)

      ShellHookManager.shared.shellWillExecuteCommand(context: hook.context.activeContext())
    case .postExec(let hook):
      IPC.post(notification: .postExec, object: hook)

      API.notifications.post(hook.historyNotification)
    case .keyboardFocusChanged(let hook):
      IPC.post(notification: .keyboardFocusChanged, object: hook)

      ShellHookManager.shared.currentTabDidChange(applicationIdentifier: hook.appIdentifier,
                                                  sessionId: hook.focusedSessionID)
    case .tmuxPaneChanged:
      break
    case .openedSshConnection(let hook):
      IPC.post(notification: .sshConnectionOpened, object: hook)
    case .callback(let hook):

      Logger.log(message: "Callback hook")
      NotificationCenter.default.post(
        name: PseudoTerminal.recievedCallbackNotification,
        object: [
          "handlerId": hook.handlerID,
          "filepath": hook.filepath,
          "exitCode": hook.exitCode
        ])
    case .integrationReady(let hook):
      ShellHookManager.shared.integrationReadyHook(identifier: hook.identifier)
    case .hide:
      Autocomplete.hide()
    case .event(let hook):
      API.notifications.post(hook.eventNotification)
    case .fileChanged(let hook):
      if hook.fileChanged == Local_FileChangedHook.FileChanged.settings {
        Settings.shared.settingsUpdated()
      }
      if hook.fileChanged == Local_FileChangedHook.FileChanged.state {
        LocalState.shared.localStateUpdated()
      }
    case .cursorPosition:
      // Used in Fig Tauri
      break
    case .focusChange:
      // Used in Fig Tauri
      break
    case .interceptedKey:
      // Used in Fig Tauri
      break
    case .none:
      break
    }
  }
}

extension Local_ShellContext {
  func activeContext() -> Local_ShellContext {
    guard self.hasRemoteContext else {
      return self
    }

    return Local_ShellContext.with { context in
      // Do not update session id or integration version (should use local value)
      context.integrationVersion = self.integrationVersion
      context.sessionID = self.sessionID
      context.hostname = self.remoteContext.hostname
      context.pid = self.remoteContext.pid
      context.processName = self.remoteContext.processName
      context.currentWorkingDirectory = self.remoteContext.currentWorkingDirectory
      context.ttys = self.remoteContext.ttys
    }
  }
}

extension NSTextCheckingResult {
  func groups(testedString: String) -> [String] {
    var groups = [String]()
    for idx in 0..<self.numberOfRanges {
      let group = String(testedString[Range(self.range(at: idx), in: testedString)!])
      groups.append(group)
    }
    return groups
  }
}

extension Data {
  func subdata(in range: ClosedRange<Index>) -> Data {
    return subdata(in: range.lowerBound..<range.upperBound)
  }
}

extension Data {

  init<T>(from value: T) {
    self = Swift.withUnsafeBytes(of: value) { Data($0) }
  }

  func to<T>(type: T.Type) -> T? where T: ExpressibleByIntegerLiteral {
    var value: T = 0
    guard count >= MemoryLayout.size(ofValue: value) else { return nil }
    _ = Swift.withUnsafeMutableBytes(of: &value, { copyBytes(to: $0) })
    return value
  }
}
