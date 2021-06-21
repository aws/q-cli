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

protocol UnixSocketServerDelegate {
  func recieved(string: String)
}

class UnixSocketServer {
  static let bufferSize = 4096
  let path: String
  var delegate: UnixSocketServerDelegate?
  var listenSocket: Socket? = nil
  var continueRunningValue = true
  var connectedSockets = [Int32: Socket]()
  let socketLockQueue = DispatchQueue(label: "com.fig.socketLockQueue")
  var continueRunning: Bool {
    set(newValue) {
      socketLockQueue.sync {
        self.continueRunningValue = newValue
      }
    }
    get {
      return socketLockQueue.sync {
        self.continueRunningValue
      }
    }
  }

  init(path: String) {
    self.path = path
  }
  
  deinit {
    // Close all open sockets...
    for socket in connectedSockets.values {
      socket.close()
    }
    self.listenSocket?.close()
  }
  
  func run() {
    
    let queue = DispatchQueue.global(qos: .userInitiated)
    
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
          let newSocket = try socket.acceptClientConnection()
          
//          print("Accepted connection from: \(newSocket.remoteHostname) on port \(newSocket.remotePort)")
//          print("Socket Signature: \(String(describing: newSocket.signature?.description))")
          
          self.addNewConnection(socket: newSocket)
          
        } while self.continueRunning
        
      }
      catch let error {
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
        // Write the welcome string...
//        try socket.write(from: "Hello, type 'QUIT' to end session\nor 'SHUTDOWN' to stop server.\n")
        
        repeat {
          let bytesRead = try socket.read(into: &readData)
          
          if bytesRead > 0 {
            guard let response = String(data: readData, encoding: .utf8) else {
              
              print("Error decoding response...")
              readData.count = 0
              break
            }

//            let reply = "Server response: \n\(response)\n"
            self.delegate?.recieved(string: response)
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
        
      }
      catch let error {
        guard let socketError = error as? Socket.Error else {
          print("Unexpected error by connection at \(socket.remoteHostname):\(socket.remotePort)...")
          return
        }
        if self.continueRunning {
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
