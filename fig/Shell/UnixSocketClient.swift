//
//  UnixSocketClient.swift
//  fig
//
//  Created by Matt Schrage on 2/10/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

//import Foundation
//import ServerAddress
//https://github.com/MasterBel2/UberserverClientCore/blob/b2f3dcad2810e311fd923a6b336ba976db661287/Sources/UberserverClientCore/Server/Socket.swift
protocol UnixSocketDelegate: AnyObject {
    func socket(_ socket: UnixSocketClient, didReceive data: Data)
    func socket(_ socket: UnixSocketClient, didReceive message: String)
    func socketDidClose(_ socket: UnixSocketClient)
}

final class UnixSocketClient: NSObject, StreamDelegate {
  
  // MARK: - Properties
  
  weak var delegate: UnixSocketDelegate?
  
  let path: String
  
  private var inputStream: InputStream?
  private var outputStream: OutputStream?
  
  private let messageBuffer = NSMutableData(capacity: 256)!
  
  private var isConnected: Bool { return inputStream != nil && outputStream != nil }
  
  // MARK: - Lifecycle
  
  init(path: String) {
    self.path = path
  }
  
  // MARK: - Public API
    /// Instructs the socket to connect to the server
  func connect() -> Bool {
    guard !isConnected else {
      return false
    }

    var cfReadStream : Unmanaged<CFReadStream>?
    var cfWriteStream : Unmanaged<CFWriteStream>?
    
    let socketFileDescriptor = socket(AF_UNIX, SOCK_STREAM, 0)


    var addr = sockaddr_un()
    addr.sun_len = UInt8(MemoryLayout<sockaddr_un>.size)
    addr.sun_family = sa_family_t(AF_UNIX)
    
    let lengthOfPath = path.utf8.count

    // Validate the length...
    guard lengthOfPath < MemoryLayout.size(ofValue: addr.sun_path) else {
      print("Pathname supplied is too long.");
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
              print("Socket: Error connecting to socket, \(errno)")
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


    
    guard let inputStream = inputStream, let outputStream = outputStream else {
      print("Failed to get input & output streams")
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
  }
  
  func send(message: String) {
    guard let outputStream = outputStream else {
      print("Error: Not Connected")
      return
    }
    guard let data = message.data(using: String.Encoding.utf8, allowLossyConversion: false) else {
      print("Cannot convert message into data to send: invalid format?")
      return
    }
    print("Socket: send '\(message)'")
    var bytes = Array<UInt8>(repeating: 0, count: data.count)
    (data as NSData).getBytes(&bytes, length: data.count)
    outputStream.write(&bytes, maxLength: data.count)
  }
  
  // MARK: - StreamDelegate
  
  func stream(_ stream: Stream, handle eventCode: Stream.Event) {
    switch eventCode {
      
    case Stream.Event():
      break
      
    case Stream.Event.openCompleted:
      print("Socket: openCompleted")
      break
      
    case Stream.Event.hasBytesAvailable:
      print("Socket: hasBytesAvailable")

      guard let input = stream as? InputStream else { break }
      
      var byte: UInt8 = 0
      while input.hasBytesAvailable {
        let bytesRead = input.read(&byte, maxLength: 1)
        messageBuffer.append(&byte, length: bytesRead)
      }
      
      print("Socket: buffer = \(String(data: messageBuffer as Data, encoding: String.Encoding.utf8) ?? "")")
      // only inform our delegate of complete messages (must end in newline character)
      if let message = String(data: messageBuffer as Data, encoding: String.Encoding.utf8), message.hasSuffix("\n") {
        delegate?.socket(self, didReceive: messageBuffer as Data
        )
        delegate?.socket(self, didReceive: message)
        messageBuffer.length = 0
      }
      
    case Stream.Event.hasSpaceAvailable:
      print("Socket: hasSpaceAvailable ")
      break
    case Stream.Event.endEncountered:
      print("Socket: endEncountered ")

    case Stream.Event.errorOccurred:
      print("Socket: errorOccurred ")
      
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
