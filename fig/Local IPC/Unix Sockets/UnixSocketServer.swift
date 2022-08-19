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
import SwiftProtobuf

enum FigProtoEncoding: String {
  case binary = "pbuf"
  case json = "json"

  var type: String {
    return self.rawValue
  }

  var typeBytes: Data {
    return self.rawValue.data(using: .utf8)!
  }

  static var typeSize: Int {
    return 4
  }

  static var headerPrefix: Data {
    return "\u{1B}@fig-".data(using: .utf8)!
  }
  // \efig-(pbuf|json)
  static var headerSize: Int {
    return headerPrefix.count + typeSize + 8
  }
}

protocol UnixSocketServerDelegate: AnyObject {
  func received(data: Data, on socket: Socket, using encoding: FigProtoEncoding)
  func onCloseConnection(socket: Socket)
}

class UnixSocketServer {
  static let bufferSize = 4096
  let path: String
  weak var delegate: UnixSocketServerDelegate?
  var listenSocket: Socket?
  var continueRunningValue = true
  var connectedSockets = [Int32: Socket]()
  let socketLockQueue = DispatchQueue(label: "com.fig.socketLockQueue")
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

  init(path: String) {
    self.path = path
    let url = URL(fileURLWithPath: self.path)
    try? FileManager.default.createDirectory(at: url.deletingLastPathComponent(),
                                             withIntermediateDirectories: true,
                                             // Create with drwxrwxrwt permissions so folder
                                             // can be reused from separate accounts
                                             // https://github.com/withfig/fig/issues/1140
                                             attributes: [ .posixPermissions: 0o1777 ])
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

  // send a response to a socket that conforms to the IPC protocol
  func send(_ response: SwiftProtobuf.Message, to socket: Socket, encoding: FigProtoEncoding) throws {
    var data: Data!
    switch encoding {
    case .binary:
      data = try response.serializedData()
    case .json:
      let json = try response.jsonString()
      data = json.data(using: .utf8)
    }

    try socket.write(from: "\u{001b}@fig-\(encoding.type)")
    try socket.write(from: Data(from: Int64(data.count).bigEndian))
    try socket.write(from: data)
  }

  func processRawBytes(rawBytes: Data) throws -> (Data, FigProtoEncoding)? {
    var header = rawBytes.subdata(in: 0...FigProtoEncoding.headerSize)

    guard header.starts(with: FigProtoEncoding.headerPrefix) else {
      return nil
    }

    header = header.advanced(by: FigProtoEncoding.headerPrefix.count)

    let type = header.subdata(in: 0..<FigProtoEncoding.typeSize)
    let encoding: FigProtoEncoding!
    switch type {
    case FigProtoEncoding.binary.typeBytes:
      encoding = .binary
    case FigProtoEncoding.json.typeBytes:
      encoding = .json
    default:
      return nil
    }

    header = header.advanced(by: FigProtoEncoding.typeSize)

    let packetSizeData = header.subdata(in: 0..<8)
    guard let packetSizeLittleEndian = packetSizeData.to(type: Int64.self) else {
      return nil
    }

    let packetSize = Int64(bigEndian: packetSizeLittleEndian)

    guard packetSize <= rawBytes.count - FigProtoEncoding.headerSize && packetSize >= 0 else {
      return nil
    }

    return (rawBytes.subdata(in: FigProtoEncoding.headerSize...FigProtoEncoding.headerSize + Int(packetSize)), encoding)
  }

  func addNewConnection(socket: Socket) {
    socketLockQueue.sync { [unowned self, socket] in
      self.connectedSockets[socket.socketfd] = socket
    }

    let queue = DispatchQueue.global(qos: .default)

    queue.async { [unowned self, socket] in

      var shouldKeepRunning = true

      var readData = Data(capacity: UnixSocketServer.bufferSize)

      do {
        repeat {
          let bytesRead = try socket.read(into: &readData)

          if bytesRead > 0 {
            if let (message, encoding) = try? processRawBytes(rawBytes: readData) {
              self.delegate?.received(data: message, on: socket, using: encoding)
            }
          }

          if bytesRead == 0 {
            shouldKeepRunning = false
            break
          }

          readData.count = 0

        } while shouldKeepRunning

        socket.close()
        self.delegate?.onCloseConnection(socket: socket)

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
