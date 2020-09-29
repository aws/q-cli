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
        connections.removeValue(forKey: connection.id)
    }

    public func received(message: Data, from: WebSocketConnection) {
        from.close(reason: .invalidDataType, description: "Chat-Server only accepts text messages")

        connections.removeValue(forKey: from.id)
    }

    public func received(message: String, from: WebSocketConnection) {
        print("msg:", message)
          let decoder = JSONDecoder()
                do {
                    let firstPass = try decoder.decode(SocketMessage.self, from: message.data(using: .utf8)!)
                    switch firstPass.type {
                        case "request":
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
                        if (msg.options?.first?.hasPrefix("bg:") ?? false) {
                            NotificationCenter.default.post(name: .currentDirectoryDidChange, object: msg)
                            from.send(message: "disconnect")

                        } else {
                            NotificationCenter.default.post(name: .recievedDataFromPipe, object: msg)
                        }
//                        from.send(message: "disconnect")
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
        //            if msg.type == "pipe" {
        //                print(msg.data)
        //                NotificationCenter.default.post(name: .recievedDataFromPipe, object: msg)
        //            }
                    
                    

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
