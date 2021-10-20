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
      case binary = "proto"
      case json = "json"
    
    var type: String {
      return self.rawValue
    }
  }
  
  static let shared = IPC()
  fileprivate var buffer: Data = Data()
  fileprivate let legacyServer = UnixSocketServer(path: "/tmp/fig.socket")
  fileprivate let server = UnixSocketServer(path: FileManager.default.temporaryDirectory.appendingPathComponent("fig.socket").path,
                                            bidirectional: true)
  init() {
    legacyServer.delegate = self
    legacyServer.run()
    
    server.delegate = self
    server.run()
  }
  
  func recieved(data: Data, on socket: Socket?) {
    guard let socket = socket,
          let (message, encoding) = try? retriveMessage(rawBytes: data) else { return }
    
    do {
      try self.handle(message, from: socket, using: encoding)
    } catch {
      Logger.log(message: "Error handling IPC message: \(error.localizedDescription)",
                 subsystem: .unix)
    }
  }
  
  // send a response to a socket that conforms to the IPC protocol
  func send(_ response: CommandResponse, to socket: Socket, encoding: IPC.Encoding) throws {
    try socket.write(from: "\u{001b}@fig-\(encoding.type)")
    switch encoding {
    case .binary:
      let data = try response.serializedData()
      try socket.write(from: data)
    case .json:
      let json = try response.jsonString()
      try socket.write(from: json)
    }
    try socket.write(from: "\u{001b}\\")

  }
  
  // attempt to decode the bytes as a packet, if not possible add to buffer
  func retriveMessage(rawBytes: Data) throws -> (LocalMessage, IPC.Encoding)? {
    buffer.append(rawBytes)
    
    guard let rawString = String(data: buffer, encoding: .utf8) else {
      return nil
    }
        
    let pattern = "\\x1b@fig-(json|proto)([^\\x1b]+)\\x1b\\\\"
    let regex = try! NSRegularExpression(pattern: pattern, options: [])
    
    let matches = regex.matches(in: rawString,
                                options: [],
                                range: NSMakeRange(0, rawString.utf16.count))
    
    guard let match = matches.first else { return nil }
    
    let groups = match.groups(testedString: rawString)
    
    guard groups.count == 3 else { return nil }
    let packet = match.range(at: 0)
    let encoding = IPC.Encoding(rawValue: groups[1])
    let message = groups[2]

    // remove consumed packet from buffer
    self.buffer.removeFirst(packet.location + packet.length)
    switch encoding {
    case .binary:
        guard let data = message.data(using: .utf8) else { return nil }
        return (try LocalMessage(serializedData: data), encoding!)
    case .json:
        return (try LocalMessage(jsonString: message), encoding!)
    case .none:
      return nil
    }

  }
  
func handle(_ message: LocalMessage, from socket: Socket, using encoding: IPC.Encoding) throws {
      
      switch message.type {
      case .command(let command):
        try handleCommand(command, from: socket, using: encoding)
      case .hook(let hook):
        handleHook(hook)
      case .none:
        break;

    }
  }

func handleCommand(_ message: Local_Command, from socket: Socket, using encoding: IPC.Encoding) throws {
    let id = message.id
    var response: CommandResponse!
    switch message.command {
      case .terminalIntegrationUpdate(let request):
        response = try Integrations.providers[request.identifier]?.handleIntegrationRequest(request)
      case .none:
        break
    }
  
    response.id = id
  
    guard !message.noResponse else { return }
  
    try self.send(response, to: socket, encoding: encoding)

  }


  func handleHook(_ message: Local_Hook) {
      Logger.log(message: "Recieved hook message!", subsystem: .unix)
      let json = try? message.jsonString()
      Logger.log(message: json ?? "Could not decode message", subsystem: .unix)
  }
}

extension NSTextCheckingResult {
    func groups(testedString:String) -> [String] {
        var groups = [String]()
        for i in  0 ..< self.numberOfRanges
        {
            let group = String(testedString[Range(self.range(at: i), in: testedString)!])
            groups.append(group)
        }
        return groups
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
              ShellHookManager.shared.currentTabDidChange(shellMessage)
          case .initialize:
              ShellHookManager.shared.startedNewTerminalSession(shellMessage)
          case .prompt:
              ShellHookManager.shared.shellPromptWillReturn(shellMessage)
          case .exec:
              ShellHookManager.shared.shellWillExecuteCommand(shellMessage)
          case.ZSHKeybuffer:
              ShellHookManager.shared.updateKeybuffer(shellMessage, backing: .zle)
          case .fishKeybuffer:
              ShellHookManager.shared.updateKeybuffer(shellMessage, backing: .fish)
          case .bashKeybuffer:
              ShellHookManager.shared.updateKeybuffer(shellMessage, backing: .bash)
          case .ssh:
              ShellHookManager.shared.startedNewSSHConnection(shellMessage)
          case .vscode:
              ShellHookManager.shared.currentTabDidChange(shellMessage)
          case .hyper:
              ShellHookManager.shared.currentTabDidChange(shellMessage)
          case .callback:
            NotificationCenter.default.post(name: PseudoTerminal.recievedCallbackNotification,
                                            object: [
                                              "handlerId" : shellMessage.options?[0] ?? nil,
                                              "filepath"  : shellMessage.options?[1] ?? nil,
                                              "exitCode"  : shellMessage.options?[safe: 2] ?? nil])
          case .tmux:
              ShellHookManager.shared.tmuxPaneChanged(shellMessage)
          case .hide:
              Autocomplete.hide()
          case .clearKeybuffer:
              ShellHookManager.shared.clearKeybuffer(shellMessage)
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
          let decodedString = String(data: decodedData, encoding: .utf8) else { return nil }
    let tokens: [String] = decodedString.split(separator: " ", maxSplits: Int.max, omittingEmptySubsequences: false).map(String.init)
    
    return ["handlerId" : tokens[1], "filepath" : tokens[2]]
  }
  
  static func from(raw: String) -> ShellMessage? {
    guard let decodedData = Data(base64Encoded: raw, options: .ignoreUnknownCharacters),
          let decodedString = String(data: decodedData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) else { return nil }
    print("unix: '\(decodedString)'")
    let tokens: [String] = decodedString.split(separator: " ", maxSplits: Int.max, omittingEmptySubsequences: false).map(String.init)
    
    guard let subcommand = tokens[safe: 1],  let session = tokens[safe: 2], let integration = tokens[safe: 3] else { return nil }
    
    let integrationNumber = Int(integration) ?? 0
    
    switch IPC.Hook(rawValue: subcommand)?.packetType(for: integrationNumber) {
      case .callback:
        return ShellMessage(type: "pipe",
                            source: "",
                            session: "",
                            env: "",
                            io: nil,
                            data: "",
                            options: [String(session), String(integration), tokens[safe: 4] ?? "-1" ],
                            hook: subcommand)
      case .keypress:
        guard let tty = tokens[safe: 4],
              let pid = tokens[safe: 5],
              let histno = tokens[safe: 6],
              let cursor = tokens[safe: 7] else { return nil }
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
        
        return ShellMessage(type: "pipe",
                            source: "",
                            session: String(session),
                            env: "{\"FIG_INTEGRATION_VERSION\":\"\(integration)\",\"TTY\":\"\(tty)\",\"PID\":\"\(pid)\"}",
                            io: nil,
                            data: "",
                            options: [String(subcommand), String(cursor), String(buffer), String(histno)],
                            hook: subcommand)
      case .legacyKeypress:
        guard let histno = tokens[safe: 4],
              let cursor = tokens[safe: 5] else { return nil }
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
        
        return ShellMessage(type: "pipe",
                            source: "",
                            session: String(session),
                            env: "{\"FIG_INTEGRATION_VERSION\":\"\(integration)\"}",
                            io: nil,
                            data: "",
                            options: [String(subcommand), String(cursor), String(buffer), String(histno)],
                            hook: subcommand)
      default:
        return ShellMessage(type: "pipe",
                            source: "",
                            session: String(session),
                            env: "{\"FIG_INTEGRATION_VERSION\":\"\(integration)\"}",
                            io: nil,
                            data: "",
                            options: [ subcommand ] + Array(tokens.suffix(from: 4)),
                            hook: subcommand)
      
    }

  }
}
