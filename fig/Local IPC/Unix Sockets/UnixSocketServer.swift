//
//  UnixSocketServer.swift
//  fig
//
//  Created by Matt Schrage on 4/8/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

import Socket
import Dispatch

protocol UnixSocketServerDelegate: AnyObject {
  func recieved(string: String, on socket: Socket?)
  func recieved(data: Data, on socket: Socket?)
}

class UnixSocketServer {
  static let bufferSize = 4096
  let path: String
  weak var delegate: UnixSocketServerDelegate?
  var listenSocket: Socket?
  var continueRunningValue = true
  var connectedSockets = [Int32: Socket]()
  let socketLockQueue = DispatchQueue(label: "com.fig.socketLockQueue")
  let bidirectional: Bool
  var continueRunning: Bool {
    get {
      return socketLockQueue.sync {
        self.continueRunningValue
      }
    }
    set(newValue) {
      socketLockQueue.sync {
        self.continueRunningValue = newValue
      }
    }
  }

  init(path: String, bidirectional: Bool = false) {
    self.path = path
    let url = URL(fileURLWithPath: self.path)
    try? FileManager.default.createDirectory(at: url.deletingLastPathComponent(),
                                            withIntermediateDirectories: true,
                                            attributes: nil)
    self.bidirectional = bidirectional
  }

  deinit {
    // Close all open sockets...
    for socket in connectedSockets.values {
      socket.close()
    }
    self.listenSocket?.close()
  }

  func run() {
    try? FileManager.default.removeItem(at: URL(fileURLWithPath: path))
    let queue = DispatchQueue(label: "io.fig.unix-server", qos: .userInitiated)

    queue.async { [unowned self] in

      do {
        try self.listenSocket = Socket.create(family: .unix)

        guard let socket = self.listenSocket else {

          print("Unable to unwrap socket...")
          return
        }
        try socket.listen(on: self.path)

        print("Listening on port: \(socket.remotePath ?? "")")

        repeat {

          // Prevent server from closing when a client fails to connect
          // Fixes issue related to setting set_sockopt NO_SIGPIPE
          var newSocket: Socket?
          do {
            newSocket = try socket.acceptClientConnection()
          } catch let error {
            Logger.log(message: "connection could not be made!", subsystem: .unix)
            if let socketError = error as? Socket.Error {
              Logger.log(message: "Code: \(socketError.errorCode) - \(socketError.errorReason ?? "")", subsystem: .unix)
            }
          }

          if let newSocket = newSocket {
            self.addNewConnection(socket: newSocket)
          }

        } while self.continueRunning

      } catch let error {
        guard let socketError = error as? Socket.Error else {
          print("Unexpected error...")
          return
        }

        if self.continueRunning {

          print("Error reported:\n \(socketError.description)")

        }
      }
    }
  }

  func addNewConnection(socket: Socket) {

    // Add the new socket to the list of connected sockets...
    socketLockQueue.sync { [unowned self, socket] in
      self.connectedSockets[socket.socketfd] = socket
    }

    // Get the global concurrent queue...
    let queue = DispatchQueue.global(qos: .default)

    // Create the run loop work item and dispatch to the default priority global queue...
    queue.async { [unowned self, socket] in

      var shouldKeepRunning = true

      var readData = Data(capacity: UnixSocketServer.bufferSize)

      do {

        repeat {
          let bytesRead = try socket.read(into: &readData)

          if bytesRead > 0 {

            // maintain old behavior for legacy ~/fig.socket
            // can be removed after v1.0.53
            if !bidirectional {
              guard let response = String(data: readData, encoding: .utf8) else {

                print("Error decoding response...")
                readData.count = 0
                break
              }

              self.delegate?.recieved(string: response, on: self.bidirectional ? socket : nil)
              Logger.log(message: "recieved message \"\(response)\"", subsystem: .unix)

            }

            self.delegate?.recieved(data: readData, on: self.bidirectional ? socket : nil)

          }

          if bytesRead == 0 {

            shouldKeepRunning = false
            break
          }

          readData.count = 0

        } while shouldKeepRunning

        socket.close()

        self.socketLockQueue.sync { [unowned self, socket] in
          self.connectedSockets[socket.socketfd] = nil
        }

      } catch let error {
        guard let socketError = error as? Socket.Error else {
          print("Unexpected error by connection at \(socket.remoteHostname):\(socket.remotePort)...")
          return
        }
        if self.continueRunning {
          // swiftlint:disable line_length
          print("Error reported by connection at \(socket.remoteHostname):\(socket.remotePort):\n \(socketError.description)")
        }
      }
    }
  }

  func shutdownServer() {
    print("\nShutdown in progress...")

    self.continueRunning = false

    // Close all open sockets...
    for socket in connectedSockets.values {

      self.socketLockQueue.sync { [unowned self, socket] in
        self.connectedSockets[socket.socketfd] = nil
        socket.close()
      }
    }

  }
}
