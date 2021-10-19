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
class LocalIPC: UnixSocketServerDelegate {
  static let shared = LocalIPC()
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
    guard let socket = socket else { return }
    do {
      try self.handle(rawBytes: data, from: socket)
    } catch {
      Logger.log(message: "Error writing response to socket: \(error.localizedDescription)", subsystem: .unix)
    }
  }
  
  func retriveMessage(rawBytes: Data) -> LocalMessage? {
    buffer.append(rawBytes)
    
    guard let rawString = String(data: buffer, encoding: .utf8) else {
      return nil
    }
    
    let jsonPattern = "\\x1b@fig-json([^\\x1b]+)\\x1b\\"
    let regex = try! NSRegularExpression(pattern: jsonPattern, options: [])
    
    regex.matches(in: rawString, options: <#T##NSRegularExpression.MatchingOptions#>, range: <#T##NSRange#>)
    
    return try? LocalMessage(serializedData: rawBytes)
  }
  func handle(rawBytes: Data, from socket: Socket) throws {
      
      buffer.append(rawBytes)
      

      
      let message = try LocalMessage(serializedData: rawBytes)
      
      switch message.type {
      case .command(let message):
        switch message.command {
        case .terminalIntegrationUpdate(let request):
          let response = try Integrations.providers[request.identifier]?.handleIntegrationRequest(request)
          guard let data = try response?.serializedData() else { return }
          try socket.write(from: data)
        default:
          break
        }
        break
      case .hook(let message):
        Logger.log(message: "Recieved hook message!", subsystem: .unix)
        let json = try? message.jsonString()
        Logger.log(message: json ?? "Could not decode message", subsystem: .unix)
        break
      case .none:
        break;

    }
  }
}

extension LocalIPC {
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
    
    switch LocalIPC.Hook(rawValue: subcommand)?.packetType(for: integrationNumber) {
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
