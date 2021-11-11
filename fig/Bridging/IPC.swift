//
//  ShellHooksTransport.swift
//  fig
//
//  Created by Matt Schrage on 4/8/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import Socket

typealias LocalMessage = Local_LocalMessage
typealias CommandResponse = Local_CommandResponse
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
    //\efig-(pbuf|json)
    static var headerSize: Int {
      return headerPrefix.count + typeSize + 8
    }
  }

  static let shared = IPC()
  fileprivate var buffer: Data = Data()
  fileprivate let legacyServer = UnixSocketServer(path: "/tmp/fig.socket")
  fileprivate let server = UnixSocketServer(
    path: FileManager.default.temporaryDirectory.appendingPathComponent("fig.socket").path,
    bidirectional: true)
  init() {
    legacyServer.delegate = self
    legacyServer.run()

    server.delegate = self
    server.run()
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

    //    guard let rawString = String(data: buffer, encoding: .utf8) else {
    //      return nil
    //    }
    //
    //    let pattern = "\\x1b@fig-(json|proto)([^\\x1b]+)\\x1b\\\\"
    //    let regex = try! NSRegularExpression(pattern: pattern, options: [])
    //
    //    let matches = regex.matches(in: rawString,
    //                                options: [],
    //                                range: NSMakeRange(0, rawString.utf16.count))
    //
    //    guard let match = matches.first else { return nil }
    //
    //    let groups = match.groups(testedString: rawString)
    //
    //    guard groups.count == 3 else { return nil }
    //    let packet = match.range(at: 0)
    //    let encoding = IPC.Encoding(rawValue: groups[1])
    //    let message = groups[2]
    //
    //    // remove consumed packet from buffer
    //    self.buffer.removeFirst(packet.location + packet.length)
    //    switch encoding {
    //    case .binary:
    //        guard let data = message.data(using: .utf8) else { return nil }
    //        return (try LocalMessage(serializedData: data), encoding!)
    //    case .json:
    //        return (try LocalMessage(jsonString: message), encoding!)
    //    case .none:
    //      return nil
    //    }

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
    throws
  {
    let id = message.id
    var response: CommandResponse?

    switch message.command {
    case .terminalIntegration(let request):
      response = try Integrations.providers[request.identifier]?.handleIntegrationRequest(request)
    case .listTerminalIntegrations(_):
      response = Integrations.handleListIntegrationsRequest()
    case .logout(_):
      response = CommandHandlers.logoutCommand()
    case .restart(_):
      CommandHandlers.restartCommand()
    case .quit(_):
      CommandHandlers.quitCommand()
    case .update(let request):
      CommandHandlers.updateCommand(request.force)
    case .diagnostics(_):
      response = CommandHandlers.diagnosticsCommand()
    case .reportWindow(let request):
      CommandHandlers.displayReportWindow(message: request.report, path: request.path, figEnvVar: request.figEnvVar, terminal: request.terminal)
    case .restartSettingsListener(_):
      response = CommandHandlers.restartSettingsListenerCommand()
    case .runInstallScript(_):
      response = CommandHandlers.runInstallScriptCommand()
    case .build(let request):
      response = CommandHandlers.buildCommand(build: request.branch)
    case .openUiElement(let request):
      response = CommandHandlers.openUiElement(uiElement: request.element)
    case .resetCache(_):
      response = CommandHandlers.resetCache()
    case .toggleDebugMode(let request):
      response = CommandHandlers.toggleAutocompleteDebugMode(setVal: request.hasDebugMode ? request.debugMode : nil)
    case .none:
      break
    }

    guard !message.noResponse else { return }
    
    if var resp = response {
      resp.id = id
      try self.send(resp, to: socket, encoding: encoding)
    }
  }

  func handleHook(_ message: Local_Hook) {
    Logger.log(message: "Recieved hook message!", subsystem: .unix)
    
    #if DEBUG
      let json = try? message.jsonString()
      Logger.log(message: json ?? "Could not decode message", subsystem: .unix)
    #endif
    
    switch message.hook {
    case .editBuffer(let hook):
      ShellHookManager.shared.updateKeybuffer(
        context: hook.context,
        text: hook.text,
        cursor: Int(hook.cursor),
        histno: Int(hook.histno))
    case .init_p(let hook):
      ShellHookManager.shared.startedNewTerminalSession(
        context: hook.context,
        calledDirect: hook.calledDirect,
        bundle: hook.bundle,
        env: hook.env)
    case .prompt(let hook):
      ShellHookManager.shared.shellPromptWillReturn(context: hook.context)
    case .preExec(let hook):
      ShellHookManager.shared.shellWillExecuteCommand(context: hook.context)
    case .postExec(let hook):
      API.notifications.post(hook.historyNotification)
    case .keyboardFocusChanged(let hook):
      ShellHookManager.shared.currentTabDidChange(applicationIdentifier: hook.appIdentifier, sessionId: hook.focusedSessionID)
    case .tmuxPaneChanged(_):
      break
    case .openedSshConnection(_):
      break
    case .callback(_):
      break
    case .integrationReady(let hook):
      ShellHookManager.shared.integrationReadyHook(identifier: hook.identifier)
    case .hide(_):
      Autocomplete.hide()
    case .event(let hook):
      ShellHookManager.shared.eventHook(event: hook.eventName)
    case .none:
      break
    }
  }
}

extension NSTextCheckingResult {
  func groups(testedString: String) -> [String] {
    var groups = [String]()
    for i in 0..<self.numberOfRanges {
      let group = String(testedString[Range(self.range(at: i), in: testedString)!])
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

extension IPC {
  func recieved(string: String, on socket: Socket?) {

    // handle legacy `fig bg:` messages
    // todo: remove this after v1.0.53
    if socket == nil {
      legacyServerRecieved(string: string)
      return
    }
  }

  func legacyServerRecieved(string: String) {
    guard let shellMessage = ShellMessage.from(raw: string) else { return }
    DispatchQueue.main.async {
      switch Hook(rawValue: shellMessage.hook ?? "") {
      case .event:
        if let event = shellMessage.options?[safe: 1] {
          TelemetryProvider.track(event: event, with: [:])
        } else {
          print("No event")
        }
      case .cd:
        ShellHookManager.shared.currentDirectoryDidChange(shellMessage)
      case .tab:
        ShellHookManager.shared.currentTabDidChangeLegacy(shellMessage)
      case .initialize:
        ShellHookManager.shared.startedNewTerminalSessionLegacy(shellMessage)
      case .prompt:
        ShellHookManager.shared.shellPromptWillReturnLegacy(shellMessage)
      case .exec:
        ShellHookManager.shared.shellWillExecuteCommandLegacy(shellMessage)
      case .ZSHKeybuffer:
        ShellHookManager.shared.updateKeybufferLegacy(shellMessage)
      case .fishKeybuffer:
        ShellHookManager.shared.updateKeybufferLegacy(shellMessage)
      case .bashKeybuffer:
        ShellHookManager.shared.updateKeybufferLegacy(shellMessage)
      case .ssh:
        ShellHookManager.shared.startedNewSSHConnectionLegacy(shellMessage)
      case .vscode:
        ShellHookManager.shared.currentTabDidChangeLegacy(shellMessage)
      case .hyper:
        ShellHookManager.shared.currentTabDidChangeLegacy(shellMessage)
      case .callback:
        NotificationCenter.default.post(
          name: PseudoTerminal.recievedCallbackNotification,
          object: [
            "handlerId": shellMessage.options?[0] ?? nil,
            "filepath": shellMessage.options?[1] ?? nil,
            "exitCode": shellMessage.options?[safe: 2] ?? nil,
          ])
      case .tmux:
        ShellHookManager.shared.tmuxPaneChangedLegacy(shellMessage)
      case .hide:
        Autocomplete.hide()
      case .clearKeybuffer:
        ShellHookManager.shared.clearKeybufferLegacy(shellMessage)
      default:
        print("Unknown background Unix socket")
      }
    }
  }

  enum Hook: String {
    case event = "bg:event"
    case cd = "bg:cd"
    case tab = "bg:tab"
    case initialize = "bg:init"
    case prompt = "bg:prompt"
    case exec = "bg:exec"
    case ZSHKeybuffer = "bg:zsh-keybuffer"
    case fishKeybuffer = "bg:fish-keybuffer"
    case bashKeybuffer = "bg:bash-keybuffer"
    case ssh = "bg:ssh"
    case vscode = "bg:vscode"
    case hyper = "bg:hyper"
    case tmux = "bg:tmux"
    case hide = "bg:hide"
    case clearKeybuffer = "bg:clear-keybuffer"
    case callback = "pty:callback"

    func packetType(for version: Int = 0) -> ShellMessage.PacketType {
      switch self {
      case .fishKeybuffer, .ZSHKeybuffer, .bashKeybuffer:
        return version >= 4 ? .keypress : .legacyKeypress
      case .prompt, .initialize, .exec:
        return .shellhook
      case .callback:
        return .callback
      default:
        return .standard
      }
    }
  }
}

extension ShellMessage {
  enum PacketType {
    case keypress
    case legacyKeypress
    case shellhook
    case standard
    case callback
  }

  static func callback(raw: String) -> [String: String]? {
    guard let decodedData = Data(base64Encoded: raw, options: .ignoreUnknownCharacters),
      let decodedString = String(data: decodedData, encoding: .utf8)
    else { return nil }
    let tokens: [String] = decodedString.split(
      separator: " ", maxSplits: Int.max, omittingEmptySubsequences: false
    ).map(String.init)

    return ["handlerId": tokens[1], "filepath": tokens[2]]
  }

  static func from(raw: String) -> ShellMessage? {
    guard let decodedData = Data(base64Encoded: raw, options: .ignoreUnknownCharacters),
      let decodedString = String(data: decodedData, encoding: .utf8)?.trimmingCharacters(
        in: .whitespacesAndNewlines)
    else { return nil }
    print("unix: '\(decodedString)'")
    let tokens: [String] = decodedString.split(
      separator: " ", maxSplits: Int.max, omittingEmptySubsequences: false
    ).map(String.init)

    guard let subcommand = tokens[safe: 1], let session = tokens[safe: 2],
      let integration = tokens[safe: 3]
    else { return nil }

    let integrationNumber = Int(integration) ?? 0

    switch IPC.Hook(rawValue: subcommand)?.packetType(for: integrationNumber) {
    case .callback:
      return ShellMessage(
        type: "cca",
        source: "",
        session: "",
        env: "",
        io: nil,
        data: "",
        options: [String(session), String(integration), tokens[safe: 4] ?? "-1"],
        hook: subcommand)
    case .keypress:
      guard let tty = tokens[safe: 4],
        let pid = tokens[safe: 5],
        let histno = tokens[safe: 6],
        let cursor = tokens[safe: 7]
      else { return nil }
      // "this is the buffer"\n -- drop quotes and newline
      var buffer = tokens.suffix(from: 8).joined(separator: " ")
      if buffer.first == "\"" {
        buffer.removeFirst()
      }

      if buffer.last == "\n" {
        buffer.removeLast()
      }

      if buffer.last == "\"" {
        buffer.removeLast()
      }

      return ShellMessage(
        type: "pipe",
        source: "",
        session: String(session),
        env:
          "{\"FIG_INTEGRATION_VERSION\":\"\(integration)\",\"TTY\":\"\(tty)\",\"PID\":\"\(pid)\"}",
        io: nil,
        data: "",
        options: [String(subcommand), String(cursor), String(buffer), String(histno)],
        hook: subcommand)
    case .legacyKeypress:
      guard let histno = tokens[safe: 4],
        let cursor = tokens[safe: 5]
      else { return nil }
      // "this is the buffer"\n -- drop quotes and newline
      var buffer = tokens.suffix(from: 6).joined(separator: " ")
      if buffer.first == "\"" {
        buffer.removeFirst()
      }

      if buffer.last == "\n" {
        buffer.removeLast()
      }

      if buffer.last == "\"" {
        buffer.removeLast()
      }

      return ShellMessage(
        type: "pipe",
        source: "",
        session: String(session),
        env: "{\"FIG_INTEGRATION_VERSION\":\"\(integration)\"}",
        io: nil,
        data: "",
        options: [String(subcommand), String(cursor), String(buffer), String(histno)],
        hook: subcommand)
    default:
      return ShellMessage(
        type: "pipe",
        source: "",
        session: String(session),
        env: "{\"FIG_INTEGRATION_VERSION\":\"\(integration)\"}",
        io: nil,
        data: "",
        options: [subcommand] + Array(tokens.suffix(from: 4)),
        hook: subcommand)

    }

  }
}
