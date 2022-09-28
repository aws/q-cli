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
  static let unixSocket: URL = URL(fileURLWithPath: "/var/tmp/fig/\(NSUserName())/fig.socket")

  static let shared = IPC()
  fileprivate var buffer: Data = Data()
  fileprivate let legacyServer = UnixSocketServer(
    path: FileManager.default.temporaryDirectory.appendingPathComponent("fig.socket").path)
  fileprivate let server = UnixSocketServer(path: unixSocket.path)
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

  func received(data: Data, on socket: Socket, using encoding: FigProtoEncoding) {
    var message: LocalMessage?
    switch encoding {
    case .binary:
      message = try? LocalMessage(serializedData: data)
    case .json:
      message = try? LocalMessage(jsonUTF8Data: data)
    }
    guard let message = message else {
      return
    }

    do {
      try self.handle(message, from: socket, using: encoding)
    } catch {
      Logger.log(
        message: "Error handling IPC message: \(error.localizedDescription)",
        subsystem: .unix)
    }
  }

  func handle(_ message: LocalMessage, from socket: Socket, using encoding: FigProtoEncoding) throws {
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

  func onCloseConnection(socket: Socket) {}

  func handleCommand(_ message: Local_Command, from socket: Socket, using encoding: FigProtoEncoding)
  throws {

    Logger.log(message: "Received command message!", subsystem: .unix)

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
      response = CommandHandlers.openUiElement(uiElement: request.element,
                                               route: request.hasRoute ? request.route : nil)
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
    case .openBrowser:
      // Only used on linux
      break
    case .none:
      break
    }

    guard !message.noResponse else { return }

    if var resp = response {
      resp.id = messageId
      try self.server.send(resp, to: socket, encoding: encoding)
    }
  }

  func handleHook(_ message: Local_Hook) {
    Logger.log(message: "Received hook message!", subsystem: .unix)

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
        name: PseudoTerminal.receivedCallbackNotification,
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
    case .focusedWindowData:
      // Used in Fig Tauri
      break
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

extension FigCommon_ShellContext {
  func activeContext() -> FigCommon_ShellContext {
    return self
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
