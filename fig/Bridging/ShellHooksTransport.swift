//
//  ShellHooksTransport.swift
//  fig
//
//  Created by Matt Schrage on 4/8/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class ShellHookTransport: UnixSocketServerDelegate {
  
  static let shared = ShellHookTransport()
  fileprivate let server = UnixSocketServer(path: "/tmp/fig.socket") // "/tmp/fig.socket"
  
  init() {
    server.delegate = self
    server.run()
  }
  
  func recieved(string: String) {
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
    
    func packetType() -> ShellMessage.PacketType {
      switch self {
        case .fishKeybuffer, .ZSHKeybuffer, .bashKeybuffer:
          return .keypress
        case .prompt, .initialize, .exec:
          return .shellhook
        default:
          return .standard
      }
    }
  }
}


extension ShellMessage {
  enum PacketType {
    case keypress
    case shellhook
    case standard
  }
  
  static func from(raw: String) -> ShellMessage? {
    guard let decodedData = Data(base64Encoded: raw, options: .ignoreUnknownCharacters),
          let decodedString = String(data: decodedData, encoding: .utf8) else { return nil }
    print("unix: '\(decodedString)'")
    let tokens: [String] = decodedString.split(separator: " ", maxSplits: Int.max, omittingEmptySubsequences: false).map(String.init)
    
    guard let subcommand = tokens[safe: 1],  let session = tokens[safe: 2], let integration = tokens[safe: 3] else { return nil }
  
    
    switch ShellHookTransport.Hook(rawValue: subcommand)?.packetType() {
      case .keypress:
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
