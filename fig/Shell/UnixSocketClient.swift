//
//  UnixSocketClient.swift
//  fig
//
//  Created by Matt Schrage on 2/10/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

// import Foundation
// import ServerAddress
//https://github.com/MasterBel2/UberserverClientCore/blob/b2f3dcad2810e311fd923a6b336ba976db661287/Sources/UberserverClientCore/Server/Socket.swift
protocol UnixSocketDelegate: AnyObject {
  func socket(_ socket: UnixSocketClient, didReceive data: Data)
  func socket(_ socket: UnixSocketClient, didReceive message: String)
  func socketDidClose(_ socket: UnixSocketClient)
}

final class UnixSocketClient: NSObject, StreamDelegate {
  static func log(_ message: String) {
    Logger.log(message: message, subsystem: .unix)
  }

  // MARK: - Properties

  weak var delegate: UnixSocketDelegate?

  let path: String

  private var inputStream: InputStream?
  private var outputStream: OutputStream?

  private let messageBuffer = NSMutableData(capacity: 256)!
  private let waitForNewline: Bool // maintain old behavior for DockerIntegration

  var isConnected: Bool { return inputStream != nil && outputStream != nil }

  // MARK: - Lifecycle

  init(path: String, waitForNewline: Bool = true) {
    self.path = path
    self.waitForNewline = waitForNewline
  }

  // MARK: - Public API
  /// Instructs the socket to connect to the server
  func connect() -> Bool {
    guard !isConnected else {
      return false
    }

    var cfReadStream: Unmanaged<CFReadStream>?
    var cfWriteStream: Unmanaged<CFWriteStream>?

    let socketFileDescriptor = socket(AF_UNIX, SOCK_STREAM, 0)

    var addr = sockaddr_un()
    addr.sun_len = UInt8(MemoryLayout<sockaddr_un>.size)
    addr.sun_family = sa_family_t(AF_UNIX)

    let lengthOfPath = path.utf8.count

    // Validate the length...
    guard lengthOfPath < MemoryLayout.size(ofValue: addr.sun_path) else {
      UnixSocketClient.log("Pathname supplied is too long.")
      return false
    }

    // Copy the path to the remote address...
    _ = withUnsafeMutablePointer(to: &addr.sun_path.0) { ptr in
      path.withCString {
        strncpy(ptr, $0, lengthOfPath)
      }
    }

    var error = false
    withUnsafeMutablePointer(to: &addr) {
      $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
        guard Darwin.connect(socketFileDescriptor, $0, socklen_t(MemoryLayout<sockaddr_un>.stride)) != -1 else {
          UnixSocketClient.log("Error connecting to socket, \(errno)")
          error = true
          return
        }
      }
    }

    guard !error else {
      return false
    }

    CFStreamCreatePairWithSocket(kCFAllocatorDefault, socketFileDescriptor, &cfReadStream, &cfWriteStream)

    inputStream = cfReadStream!.takeRetainedValue()
    outputStream = cfWriteStream!.takeRetainedValue()

    // causes socket to close automatically (no need to use POSIX `shutdown` API)
    inputStream?.setProperty(kCFBooleanTrue, forKey: kCFStreamPropertyShouldCloseNativeSocket as Stream.PropertyKey)
    outputStream?.setProperty(kCFBooleanTrue, forKey: kCFStreamPropertyShouldCloseNativeSocket as Stream.PropertyKey)

    guard let inputStream = inputStream, let outputStream = outputStream else {
      UnixSocketClient.log("Failed to get input & output streams")
      return false
    }

    inputStream.delegate = self
    outputStream.delegate = self

    inputStream.schedule(in: .current, forMode: .default)
    outputStream.schedule(in: .current, forMode: .default)

    inputStream.open()
    outputStream.open()

    return true
  }

  /// Instructs the socket to disconnect to the server
  func disconnect() {
    guard isConnected else {
      return
    }
    if let input = inputStream {
      input.close()
      input.remove(from: .current, forMode: .default)
      inputStream = nil
    }
    if let output = outputStream {
      output.close()
      output.remove(from: .current, forMode: .default)
      outputStream = nil
    }

    // zero buffer
    messageBuffer.resetBytes(in: NSRange(location: 0, length: messageBuffer.length))
  }

  func send(message: String) {
    guard let outputStream = outputStream else {
      UnixSocketClient.log("Error: Not Connected")
      return
    }
    guard let data = message.data(using: String.Encoding.utf8, allowLossyConversion: false) else {
      UnixSocketClient.log("Cannot convert message into data to send: invalid format?")
      return
    }
    UnixSocketClient.log("send '\(message)'")
    var bytes = [UInt8](repeating: 0, count: data.count)
    (data as NSData).getBytes(&bytes, length: data.count)
    outputStream.write(&bytes, maxLength: data.count)
  }

  func send(data: Data) {
    guard let outputStream = outputStream else {
      UnixSocketClient.log("Error: Not Connected")
      return
    }

    var bytes = [UInt8](repeating: 0, count: data.count)
    (data as NSData).getBytes(&bytes, length: data.count)
    outputStream.write(&bytes, maxLength: data.count)
  }

  // MARK: - StreamDelegate

  func stream(_ stream: Stream, handle eventCode: Stream.Event) {
    switch eventCode {

    case Stream.Event():
      break

    case Stream.Event.openCompleted:
      UnixSocketClient.log("openCompleted")
      break

    case Stream.Event.hasBytesAvailable:
      print("Socket: hasBytesAvailable")

      guard let input = stream as? InputStream else { break }

      var byte: UInt8 = 0
      while input.hasBytesAvailable {
        let bytesRead = input.read(&byte, maxLength: 1)
        messageBuffer.append(&byte, length: bytesRead)
      }

      UnixSocketClient.log("buffer = \(String(data: messageBuffer as Data, encoding: String.Encoding.utf8) ?? "")")
      delegate?.socket(self, didReceive: messageBuffer as Data)
      // only inform our delegate of complete messages (must end in newline character)
      if let message = String(data: messageBuffer as Data, encoding: String.Encoding.utf8),
         message.hasSuffix("\n") {

        delegate?.socket(self, didReceive: message)
        messageBuffer.length = 0
      } else {
        messageBuffer.length = 0
      }

    case Stream.Event.hasSpaceAvailable:
      UnixSocketClient.log("hasSpaceAvailable ")
      break
    case Stream.Event.endEncountered:
      UnixSocketClient.log("endEncountered ")

    case Stream.Event.errorOccurred:
      UnixSocketClient.log("errorOccurred ")

      stream.close()
      stream.remove(from: .current, forMode: .default)
      if stream == inputStream {
        inputStream = nil
      } else if stream == outputStream {
        outputStream = nil
      }

      if inputStream == nil && outputStream == nil {
        delegate?.socketDidClose(self)
      }
    default:
      print(eventCode)
    }
  }
}
