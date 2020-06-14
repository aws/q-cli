//
//  WebSocket2.swift
//  fig
//
//  Created by Matt Schrage on 6/9/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

import KituraNet
import KituraWebSocket

class ShellBridgeServerDelegate: ServerDelegate {
    public func handle(request: ServerRequest, response: ServerResponse) {}
}

class WebSocketServer {
    static let bridge = WebSocketServer(port: 8765)
    
    init(port: Int) {
        
        WebSocket.register(service: ShellBridgeSocketService(), onPath: "/")

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
}

class ShellBridgeSocketService: WebSocketService {

    private var connections = [String: WebSocketConnection]()
    
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
                    let msg = try decoder.decode(ShellMessage.self, from: message.data(using: .utf8)!)
                    print(msg)
                    
                    switch msg.type {
                    case "pipe":
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
        //            if msg.type == "pipe" {
        //                print(msg.data)
        //                NotificationCenter.default.post(name: .recievedDataFromPipe, object: msg)
        //            }
                    
                    

                } catch {
                    print("oops: couldn't parse '\(message)'")
                }
    }
}
