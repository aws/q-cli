//
//  WebSocket2.swift
//  fig
//
//  Created by Matt Schrage on 6/9/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa

import KituraNet
import KituraWebSocket
import Sentry
class ShellBridgeServerDelegate: ServerDelegate {
    public func handle(request: ServerRequest, response: ServerResponse) {}
}

class WebSocketServer {
    //lsof -i tcp:8765
  static let bridge = { () -> WebSocketServer in
      var port = 0
      for i in 50000..<65000 {
        let (isFree, _) = WebSocketServer.checkPortForListener(port: UInt16(i))
          if isFree == true {
              port = i
              break;
          }
      }
      
      Defaults.port = port == 0 ? 8765 : port
      return WebSocketServer(port: Defaults.port)
    }
    let service: ShellBridgeSocketService
    
    var connections:[String: WebSocketConnection] {
        get {
            return service.allConnections()
        }
    }
  
    static func checkPortForListener(port: in_port_t) -> (Bool, descr: String) {

        let socketFileDescriptor = socket(AF_INET, SOCK_STREAM, 0)
        if socketFileDescriptor == -1 {
            return (false, "SocketCreationFailed, \(descriptionOfLastError())")
        }

        var addr = sockaddr_in()
        let sizeOfSockkAddr = MemoryLayout<sockaddr_in>.size
        addr.sin_len = __uint8_t(sizeOfSockkAddr)
        addr.sin_family = sa_family_t(AF_INET)
        addr.sin_port = Int(OSHostByteOrder()) == OSLittleEndian ? _OSSwapInt16(port) : port
        addr.sin_addr = in_addr(s_addr: inet_addr("0.0.0.0"))
        addr.sin_zero = (0, 0, 0, 0, 0, 0, 0, 0)
        var bind_addr = sockaddr()
        memcpy(&bind_addr, &addr, Int(sizeOfSockkAddr))

        if Darwin.bind(socketFileDescriptor, &bind_addr, socklen_t(sizeOfSockkAddr)) == -1 {
            let details = descriptionOfLastError()
            release(socket: socketFileDescriptor)
            return (false, "\(port), BindFailed, \(details)")
        }
        if listen(socketFileDescriptor, SOMAXCONN ) == -1 {
            let details = descriptionOfLastError()
            release(socket: socketFileDescriptor)
            return (false, "\(port), ListenFailed, \(details)")
        }
        release(socket: socketFileDescriptor)
        return (true, "\(port) is free for use")
    }

    fileprivate static func release(socket: Int32) {
        Darwin.shutdown(socket, SHUT_RDWR)
        close(socket)
    }

    fileprivate static func descriptionOfLastError() -> String {
        return String.init(cString: (UnsafePointer(strerror(errno))))
    }

    init(port: Int) {
        service = ShellBridgeSocketService()
        WebSocket.register(service: service, onPath: "/")

        DispatchQueue.global(qos: .background).async {
            let server = HTTP.createServer()
                   server.delegate = ShellBridgeServerDelegate()
                   do {
                       try server.listen(on: port, address: "localhost")
                       ListenerGroup.waitForListeners()
                   } catch {
                        SentrySDK.capture(message: "Error listening on port \(port): \(error).")

                        DispatchQueue.main.async {
                            if let delegate = NSApp.delegate as? AppDelegate {
                              let _ = delegate.dialogOKCancel(question: "Could not link with terminal", text: "A process is already listening on port \(Defaults.port).\nRun `lsof -i tcp:\(Defaults.port)` to identify it.\n\nPlease email hello@withfig.com for help debugging.", prompt: "", noAction: true, icon: nil)
                            }
                        }
                   }
        }
       
    }
    
    func send(sessionId: String, command: String) {
        if let connection = self.service.connection(for: sessionId) {
            connection.send(message: command)
        }
    }
}

class ShellBridgeSocketService: WebSocketService {
    private let queue = DispatchQueue(label: "com.withfig.socket", attributes: .concurrent)

    var connections = [String: WebSocketConnection]()
    var sessionIds: [String : String] = [:]
    func connection(for sessionId: String) -> WebSocketConnection? {
        return connections[sessionIds[sessionId] ?? ""]
    }
  
    func allConnections() -> [String: WebSocketConnection] {
      var allConnections: [String: WebSocketConnection]!
      queue.sync {
        allConnections = self.connections
      }
      
      return allConnections
  }
    
    let connectionTimeout: Int? = 60

    public func connected(connection: WebSocketConnection) {
        print("connected:",connection.id)
      
        queue.async(flags: [.barrier]) {
          self.connections[connection.id] = connection
        }
    }

    public func disconnected(connection: WebSocketConnection, reason: WebSocketCloseReasonCode) {
        print("disconnected:",connection.id)
        // exec bad access error occured here
        queue.async(flags: [.barrier]) {
          self.connections.removeValue(forKey: connection.id)
        }
    }

    public func received(message: Data, from: WebSocketConnection) {
        from.close(reason: .invalidDataType, description: "Fig only accepts text messages")

        queue.async(flags: [.barrier]) {
          self.connections.removeValue(forKey: from.id)
        }

    }

    public func received(message: String, from: WebSocketConnection) {
          let decoder = JSONDecoder()
                do {
                    let firstPass = try decoder.decode(SocketMessage.self, from: message.data(using: .utf8)!)
                    switch firstPass.type {
                        case "request":
                            guard Defaults.loggedIn else {
                                from.send(message: "disconnect")
                                return
                            }
                            
                            guard let username = firstPass.username, let slug = firstPass.slug else {
                                break;
                            }
                            DispatchQueue.main.async {

                                let alert = NSAlert()
                                alert.messageText = "Open @\(username)'s Runbook?"
                                alert.informativeText = "Would you like to open @\(username)'s runbook '\(slug)'? It may execute shell scripts on your behalf."
                                alert.alertStyle = .warning
                                alert.addButton(withTitle: "Open")
                                alert.addButton(withTitle: "Copy Command")
                                alert.addButton(withTitle: "Not now")
                                let res = alert.runModal()
                                if (res == .alertFirstButtonReturn) {
                                    WindowManager.shared.bringTerminalWindowToFront()
                                    print("OPEN \(username)/\(slug)")
                                    Timer.delayWithSeconds(0.5) {
                                        ShellBridge.injectStringIntoTerminal("fig @\(username) \(slug)", runImmediately: true)

                                    }
                                } else if (res == .alertSecondButtonReturn) {
                                    NSPasteboard.general.clearContents()
                                    NSPasteboard.general.setString("fig @\(username) \(slug)", forType: .string)
                                }
                            }
                    default:
                        break;
                    }

                    let msg = try decoder.decode(ShellMessage.self, from: message.data(using: .utf8)!)
                    print(msg)
                    
                    switch msg.type {
                    case "hello":
                        self.sessionIds[msg.session] = from.id
                    case "pipe":
                        print("Handle CLI command: fig \((msg.options ?? []).joined(separator: " "))")
                        guard Defaults.loggedIn else {
                            from.send(message: "disconnect")
                            return
                        }
                        if let subcommand = msg.options?.first {
                            guard !subcommand.hasPrefix("bg:") else {
                                switch subcommand {
                                case "bg:event":
                                    if let event = msg.options?[safe: 1] {
                                        TelemetryProvider.track(event: event, with: [:])
                                    } else {
                                        print("No event")
                                    }
                                    case "bg:cd":
                                        ShellHookManager.shared.currentDirectoryDidChange(msg)
                                    case "bg:tab":
                                        ShellHookManager.shared.currentTabDidChange(msg)
                                    case "bg:init":
                                        ShellHookManager.shared.startedNewTerminalSession(msg)
                                    case "bg:prompt":
                                        ShellHookManager.shared.shellPromptWillReturn(msg)
                                    case "bg:exec":
                                        ShellHookManager.shared.shellWillExecuteCommand(msg)
                                    case "bg:zsh-keybuffer":
                                        ShellHookManager.shared.updateKeybuffer(msg)
                                    case "bg:ssh":
                                        ShellHookManager.shared.startedNewSSHConnection(msg)
                                    case "bg:vscode":
                                        ShellHookManager.shared.currentTabDidChange(msg)
                                    case "bg:alert":
                                        if let title = msg.options?[safe: 1], let text = msg.options?[safe: 2]  {
                                            DispatchQueue.main.async {
                                                if let delegate =  NSApp.delegate as? AppDelegate {
                                                    let _ = delegate.dialogOKCancel(question: title, text: text, noAction: true, icon: NSImage(imageLiteralResourceName: NSImage.applicationIconName))
                                                }
                                            }
                                        } else {
                                            Logger.log(message: "bg:alert requires <title> <text>")
                                        }
                                    default:
                                        print("Uknown background command 'fig \(subcommand)'")
                                }
                                    
                                from.send(message: "disconnect")
                                return
                            }
                        }
                        
                        // native fig CLI commands
                        if let command = NativeCLI.Command(rawValue: msg.options?.first ?? NativeCLI.index) {
                            NativeCLI.route(command, with: msg, from: from)
                            return
                        }
                        
                        // Legacy routing to fig apps
                        NotificationCenter.default.post(name: .recievedDataFromPipe, object: msg)

                    case "pty":
                        if let io = msg.io {
                            if io == "i" {
                                NotificationCenter.default.post(name: .recievedUserInputFromTerminal, object: msg)
                            } else if io == "o" {
                                NotificationCenter.default.post(name: .recievedStdoutFromTerminal, object: msg)
                            }
                        }
                        
                    default:
                        print("Unhandled match from Websocket Message")
                    }
                    
                    
                } catch {
                    print("oops: couldn't parse '\(message)'")
                }
    }
}

struct SocketMessage: Codable {
    var type: String
    var username: String?
    var slug: String?

}
