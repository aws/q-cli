//
//  ShellHooksTransport.swift
//  fig
//
//  Created by Matt Schrage on 4/8/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import Socket
import FigAPIBindings
import Cocoa

// 2. When we stop receiving messages on connection, cancel ping timer, remove
//    outgoing handle from TerminalSessionLinkingService
//
// 3. Pass a nonce to ensure that we don't process PTY.execute or Process.run
//    responses twice
//
// 4. Listen for incoming hooks/responses on connection. Handle hooks as usual.
//    Handle reponses to PTY execute and Process.run events as usual (notifications?).
//    Store requests by handler id in a map in SecureIPC.

struct SocketSession {
  let sessionId: String
  let secret: String
  let pingTimer: Timer
  let socket: Socket
  var responseMap: [UInt64: (Secure_Hostbound.Response) -> Void]

  var counter: UInt64 = 0
}

// swiftlint:disable type_body_length
// swiftlint:disable type_name
class SecureIPC: UnixSocketServerDelegate {
  typealias ReceiveType = Secure_Hostbound

  static let unixSocket: URL = URL(fileURLWithPath: "/var/tmp/fig/\(NSUserName())/secure.socket")

  static let shared = SecureIPC()
  fileprivate var buffer: Data = Data()
  fileprivate let server = UnixSocketServer(path: unixSocket.path)
  fileprivate var verifiedSocketSessionmap: [String: Int32] = [:]
  fileprivate var socketSessions: [Int32: SocketSession] = [:]

  init() {
    server.delegate = self
    server.run()

    // Prevent "App Nap" from automatically killing Fig if the computer goes to sleep
    // while the user has disabled the menubar icon
    // See: https://stackoverflow.com/questions/19577541/disabling-timer-coalescing-in-osx-for-a-given-process
    ProcessInfo.processInfo.disableAutomaticTermination(
      "Running unix socket server to handle updates from active terminal sessions."
    )
  }

  fileprivate func getSession(for sessionId: String) -> SocketSession? {
    guard let socketfd = self.verifiedSocketSessionmap[sessionId] else {
      return nil
    }
    return self.socketSessions[socketfd]
  }

  fileprivate func updateSession(for sessionId: String, with callback: (SocketSession) throws -> SocketSession) throws {
    let queue = DispatchQueue(label: "io.fig.secureSocket.\(sessionId)")
    try queue.sync {
      guard let socketfd = self.verifiedSocketSessionmap[sessionId] else {
        throw APIError.generic(message: "No open connection for \(sessionId)")
      }
      guard let session = self.socketSessions[socketfd] else {
        throw APIError.generic(message: "No open connection for \(sessionId)")
      }
      self.socketSessions[socketfd] = try callback(session)
    }
  }

  fileprivate func addResponseHandler(
    for sessionId: String,
    handler: @escaping (Secure_Hostbound.Response) -> Void,
    callback: @escaping ((UInt64) throws -> Void)
  ) throws {
    try self.updateSession(for: sessionId) { (oldSession) in
      var session = oldSession
      session.responseMap[oldSession.counter] = handler
      try callback(oldSession.counter)
      session.counter += 1
      return session
    }
  }

  internal func received(data: Data, on socket: Socket, using encoding: FigProtoEncoding) {
    var message: Secure_Hostbound?
    switch encoding {
    case .binary:
      message = try? Secure_Hostbound(serializedData: data)
    case .json:
      message = try? Secure_Hostbound(jsonUTF8Data: data)
    }
    guard let message = message else {
      return
    }

    do {
      try self.handle(message, from: socket, using: encoding)
    } catch {
      Logger.log(
        message: "Error handling IPC message: \(error.localizedDescription)",
        subsystem: .unix)
    }
  }

  internal func onCloseConnection(socket: Socket) {
    if let socketSession = self.socketSessions.removeValue(forKey: socket.socketfd) {
      socketSession.pingTimer.invalidate()
      self.verifiedSocketSessionmap.removeValue(forKey: socketSession.sessionId)
    }
  }

  internal func handle(_ message: Secure_Hostbound, from socket: Socket, using encoding: FigProtoEncoding) throws {
    let socketSession = self.socketSessions[socket.socketfd]
    switch message.packet {
    case .handshake(let handshake):
      try handleHandshake(handshake, from: socket, using: encoding)
    case .hook(let hook):
      if socketSession != nil {
        DispatchQueue.main.async {
          self.handleHook(hook)
        }
      } else {
        let resp = Secure_Clientbound.with { message in
          message.handshakeResponse = Secure_Clientbound.HandshakeResponse.with({ resp in
            resp.success = false
          })
        }
        try self.server.send(resp, to: socket, encoding: encoding)
      }
    case .response(let response):

      if let session = socketSession, response.hasNonce {
        try sendResponse(for: session.sessionId, with: response)
      }
    case .none:
      break
    }
  }

  func sendResponse(for sessionId: String, with response: Secure_Hostbound.Response) throws {
    try updateSession(for: sessionId) { (oldSession) in
      var session = oldSession
      if let handler = session.responseMap.removeValue(forKey: response.nonce) {
        DispatchQueue.main.async {
          handler(response)
        }
      }
      return session
    }
  }

  func makeExecuteRequest(
    for sessionId: String,
    with request: Fig_PseudoterminalExecuteRequest,
    callback: @escaping ((Fig_PseudoterminalExecuteResponse) -> Void)
  ) throws {
    try self.addResponseHandler(for: sessionId, handler: { (response) in
      if case .pseudoterminalExecute(let res) = response.response {
        callback(res)
      }
    }) { [weak self] (handlerId) in
      let clientRequest = Secure_Clientbound.Request.with { req in
        req.nonce = handlerId
        req.pseudoterminalExecute.command = request.command
        req.pseudoterminalExecute.env = request.env
        if request.hasWorkingDirectory {
          req.pseudoterminalExecute.workingDirectory = request.workingDirectory
        }
        if request.hasIsPipelined {
          req.pseudoterminalExecute.isPipelined = request.isPipelined
        }
        if request.hasBackgroundJob {
          req.pseudoterminalExecute.backgroundJob = request.backgroundJob
        }
      }
      try self?.makeRequest(for: sessionId, with: clientRequest)
    }
  }

  func makeProcessRunRequest(
    for sessionId: String,
    with request: Fig_RunProcessRequest,
    callback: @escaping ((Fig_RunProcessResponse) -> Void)
  ) throws {
    try self.addResponseHandler(for: sessionId, handler: { (response) in
      if case .runProcess(let res) = response.response {
        callback(res)
      }
    }) { [weak self] (handlerId) in
      let clientRequest = Secure_Clientbound.Request.with { req in
        req.nonce = handlerId
        req.runProcess.executable = request.executable
        req.runProcess.arguments = request.arguments
        req.runProcess.env = request.env
        if request.hasWorkingDirectory {
          req.runProcess.workingDirectory = request.workingDirectory
        }
      }
      try self?.makeRequest(for: sessionId, with: clientRequest)
    }
  }

  func makeInsertTextRequest(
    for sessionId: String,
    with request: Figterm_InsertTextCommand
  ) throws {
    let clientRequest = Secure_Clientbound.Request.with { req in
      req.insertText = request
    }
    try self.makeRequest(for: sessionId, with: clientRequest)
  }

  fileprivate func makeRequest(for sessionId: String, with request: Secure_Clientbound.Request) throws {
    guard let session = getSession(for: sessionId) else {
      throw APIError.generic(message: "No open connection for \(sessionId)")
    }

    let clientMessage = Secure_Clientbound.with { message in
      message.request = request
    }

    try self.server.send(clientMessage, to: session.socket, encoding: .binary)
  }

  internal func handleHandshake(_ handshake: Secure_Hostbound.Handshake, from socket: Socket, using encoding: FigProtoEncoding) throws {
    let sessionId = handshake.id
    var success = false
    if let session = self.getSession(for: sessionId) {
      success = session.secret == handshake.secret
    } else {
      let timer = Timer.scheduledTimer(withTimeInterval: 5, repeats: true) { [weak self] (_) in
        let ping = Secure_Clientbound.with { message in
          message.ping = FigCommon_Empty()
        }
        try? self?.server.send(ping, to: socket, encoding: encoding)
      }
      self.socketSessions[socket.socketfd] = SocketSession.init(
        sessionId: sessionId,
        secret: handshake.secret,
        pingTimer: timer,
        socket: socket,
        responseMap: [:]
      )
      self.verifiedSocketSessionmap[sessionId] = socket.socketfd
      success = true
    }

    let resp = Secure_Clientbound.with { message in
      message.handshakeResponse = Secure_Clientbound.HandshakeResponse.with { resp in
        resp.success = success
      }
    }

    try self.server.send(resp, to: socket, encoding: encoding)
  }

  internal func handleHook(_ message: Secure_Hostbound.Hook) {
    Logger.log(message: "Recieved secure hook message!", subsystem: .unix)

    let json = try? message.jsonString()
    Logger.log(message: json ?? "Could not decode message", subsystem: .unix)

    switch message.hook {
    case .editBuffer(let hook):
      // NOTE: IPC notifications update TerminalSessionLinker and MUST occur before everything else!
      IPC.post(notification: .editBuffer, object: hook)

      ShellHookManager.shared.updateKeybuffer(
        context: hook.context.activeContext(),
        text: hook.text,
        cursor: Int(hook.cursor),
        histno: Int(hook.histno))
    case .prompt(let hook):
      IPC.post(notification: .prompt, object: hook)

      ShellHookManager.shared.shellPromptWillReturn(context: hook.context.activeContext())
    case .preExec(let hook):
      IPC.post(notification: .preExec, object: hook)

      ShellHookManager.shared.shellWillExecuteCommand(context: hook.context.activeContext())
    case .interceptedKey:
      // Used in Fig Tauri
      break
    case .none:
      break
    }
  }
}
