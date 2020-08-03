//
//  SocketServer.swift
//  figcli
//
//  Created by Matt Schrage on 7/26/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Starscream

protocol WebSocketConnection {
    func send(text: String)
    func send(data: Data)
    func connect()
    func disconnect()
    var delegate: WebSocketConnectionDelegate? {
        get
        set
    }
}

protocol WebSocketConnectionDelegate {
    func onConnected(connection: WebSocketConnection)
    func onDisconnected(connection: WebSocketConnection, error: Error?)
    func onError(connection: WebSocketConnection, error: Error)
    func onMessage(connection: WebSocketConnection, text: String)
    func onMessage(connection: WebSocketConnection, data: Data)
}

enum WebSocketError : Error {
    case error(reason: String, code: UInt16)
}
class WebSocketStarscreamConnection: WebSocketDelegate, WebSocketConnection {
    func didReceive(event: WebSocketEvent, client: WebSocket) {
        switch event {
        case .connected:
            self.delegate?.onConnected(connection: self)
        case .disconnected(let reason, let code):
            self.delegate?.onDisconnected(connection: self, error: WebSocketError.error(reason: reason, code: code))
        case .text(let string):
            self.delegate?.onMessage(connection: self, text: string)
        case .binary(let data):
            self.delegate?.onMessage(connection: self, data: data)
        case .ping(_):
            break
        case .pong(_):
            break
        case .viabilityChanged(_):
            break
        case .reconnectSuggested(_):
            break
        case .cancelled:
            self.delegate?.onDisconnected(connection: self, error: nil)
        case .error(let error):
            self.delegate?.onDisconnected(connection: self, error: error)
            
        }
    }
    
    func send(text: String) {
        socket.write(string: text)
    }
    
    func send(data: Data) {
        socket.write(stringData: data, completion: nil)
    }
    
    func connect() {
        socket.connect()
    }
    
    func disconnect() {
        socket.disconnect()

    }
    
    var delegate: WebSocketConnectionDelegate?
    let socket: WebSocket!
    init(url: URL) {
        var request = URLRequest(url: url)
        request.timeoutInterval = 5
        socket = WebSocket(request: request)
        socket.delegate = self
        socket.callbackQueue = DispatchQueue.global(qos: .default) // this is important!
        // also make sure not to call socket.connect() twice
        
    }

}
