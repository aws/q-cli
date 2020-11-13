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

class ShellBridgeServerDelegate: ServerDelegate {
    public func handle(request: ServerRequest, response: ServerResponse) {}
}

class WebSocketServer {
    static let bridge = WebSocketServer(port: 8765)
    let service: ShellBridgeSocketService
    
    var connections:[String: WebSocketConnection] {
        get {
            return service.connections
        }
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
                       print("Error listening on port \(port): \(error).")
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

    var connections = [String: WebSocketConnection]()
    var sessionIds: [String : String] = [:]
    func connection(for sessionId: String) -> WebSocketConnection? {
        return connections[sessionIds[sessionId] ?? ""]
    }
    
    let connectionTimeout: Int? = 60

    public func connected(connection: WebSocketConnection) {
        print("connected:",connection.id)
        connections[connection.id] = connection
    }

    public func disconnected(connection: WebSocketConnection, reason: WebSocketCloseReasonCode) {
        print("disconnected:",connection.id)
        // exec bad access error occured here
        connections.removeValue(forKey: connection.id)
    }

    public func received(message: Data, from: WebSocketConnection) {
        from.close(reason: .invalidDataType, description: "Fig only accepts text messages")

        connections.removeValue(forKey: from.id)
    }

    public func received(message: String, from: WebSocketConnection) {
        print("msg:", message)
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
                        if let subcommand = msg.options?.first {
                            guard !subcommand.hasPrefix("bg:") else {
                                guard Defaults.loggedIn else {
                                    from.send(message: "disconnect")
                                    return
                                }
                                switch subcommand {
                                case "bg:event":
                                    if let event = msg.options?[safe: 1] {
                                        TelemetryProvider.post(event: .viaShell, with: ["name" : event])
                                    } else {
                                        print("No event")
                                    }
                                    case "bg:cd":
                                        NotificationCenter.default.post(name: .currentDirectoryDidChange, object: msg)
                                    case "bg:tab":
                                        NotificationCenter.default.post(name: .currentTabDidChange, object: msg)
                                    case "bg:init":
                                        NotificationCenter.default.post(name: .startedNewTerminalSession, object: msg)
                                    case "bg:prompt":
                                        NotificationCenter.default.post(name: .shellPromptWillReturn, object: msg)
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
