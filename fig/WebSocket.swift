//
//  WebSocket.swift
//  fig
//
//  Created by Matt Schrage on 5/22/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//
import Foundation
//import Combine

//protocol WebSocketConnection {
//    func send(text: String)
//    func send(data: Data)
//    func connect()
//    func disconnect()
//    var delegate: WebSocketConnectionDelegate? {
//        get
//        set
//    }
//}
//
//protocol WebSocketConnectionDelegate {
//    func onConnected(connection: WebSocketConnection)
//    func onDisconnected(connection: WebSocketConnection, error: Error?)
//    func onError(connection: WebSocketConnection, error: Error)
//    func onMessage(connection: WebSocketConnection, text: String)
//    func onMessage(connection: WebSocketConnection, data: Data)
//}
//
//class WebSocketTaskConnection: NSObject, WebSocketConnection, URLSessionWebSocketDelegate {
//    var delegate: WebSocketConnectionDelegate?
//    var webSocketTask: URLSessionWebSocketTask!
//    var urlSession: URLSession!
//    let delegateQueue = OperationQueue()
//
//    init(url: URL) {
//        super.init()
//        urlSession = URLSession(configuration: .default, delegate: self, delegateQueue: delegateQueue)
//        webSocketTask = urlSession.webSocketTask(with: url)
//    }
//
//    func urlSession(_ session: URLSession, webSocketTask: URLSessionWebSocketTask, didOpenWithProtocol protocol: String?) {
//        self.delegate?.onConnected(connection: self)
//    }
//
//    func urlSession(_ session: URLSession, webSocketTask: URLSessionWebSocketTask, didCloseWith closeCode: URLSessionWebSocketTask.CloseCode, reason: Data?) {
//        self.delegate?.onDisconnected(connection: self, error: nil)
//    }
//
//    func connect() {
//        webSocketTask.resume()
//
//        listen()
//    }
//
//    func disconnect() {
//        webSocketTask.cancel(with: .goingAway, reason: nil)
//    }
//
//    func listen()  {
//        webSocketTask.receive { result in
//            switch result {
//            case .failure(let error):
//                self.delegate?.onError(connection: self, error: error)
//            case .success(let message):
//                switch message {
//                case .string(let text):
//                    self.delegate?.onMessage(connection: self, text: text)
//                case .data(let data):
//                    self.delegate?.onMessage(connection: self, data: data)
//                @unknown default:
//                    fatalError()
//                }
//
//                self.listen()
//            }
//        }
//    }
//
//    func send(text: String) {
//        webSocketTask.send(URLSessionWebSocketTask.Message.string(text)) { error in
//            if let error = error {
//                self.delegate?.onError(connection: self, error: error)
//            }
//        }
//    }
//
//    func send(data: Data) {
//        webSocketTask.send(URLSessionWebSocketTask.Message.data(data)) { error in
//            if let error = error {
//                self.delegate?.onError(connection: self, error: error)
//            }
//        }
//    }
//}
//let url = URL(string: "wss://localhost:8765")!
