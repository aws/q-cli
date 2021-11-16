//
//  LocalProcess.swift
//  
// This file contains the supporting infrastructure to run local processes that can be connected
// to a Termianl
//
//  Created by Miguel de Icaza on 4/5/20.
//
import Foundation

public protocol LocalProcessDelegate {
  func processTerminated (_ source: LocalProcess, exitCode: Int32?)
}

public class LocalProcess {
  /* Our buffer for reading data from the child process */
  var readBuffer: [UInt8] = Array.init (repeating: 0, count: 8192)

  /* The file descriptor used to communicate with the child process */
  var childfd: Int32 = -1
  var handle: FileHandle?

  /* The PID of our subprocess */
  var shellPid: pid_t = 0
  var delegate: LocalProcessDelegate
  var dispatchQueue: DispatchQueue
  var io: DispatchIO?

  public func send (data: ArraySlice<UInt8>) {
    guard running else {
      return
    }
    let copy = sendCount
    sendCount += 1
    data.withUnsafeBytes { ptr in
      let ddata = DispatchData(bytes: ptr)
      let copyCount = ddata.count
      if debugIO {
        print ("[SEND-\(copy)] Queuing data to client: \(data) ")
      }
    }
  }

  public init(delegate: LocalProcessDelegate, dispatchQueue: DispatchQueue? = nil) {
    self.delegate = delegate
    self.dispatchQueue = dispatchQueue ?? DispatchQueue.init(label: "LocalProcess", qos: .userInitiated)
  }

  public func send(data: ArraySlice<UInt8>, handlerId: String? = nil) {
    guard running else {
      return
    }
    PseudoTerminal.log("[SEND-\(handlerId ?? "0")] Queuing data to client: \(data.count) ")
    dispatchQueue.async {
      if #available(macOS 10.15.4, *) {
        try? self.handle?.write(contentsOf: data)
      } else {
        // Fallback on earlier versions
      }
      PseudoTerminal.log("[SEND-\(handlerId ?? "0")] Sent data to client: \(data.count) ")
    }
  }

  var childMonitor: DispatchSourceProcess?

  var running: Bool = false {
    didSet {
      PseudoTerminal.log("[PTY] process is\(running ? "" : " not") running")
    }
  }

  public func startProcess(executable: String = "/bin/bash", args: [String] = [], environment: [String], size: inout winsize) {
    if running {
      return
    }

    var shellArgs = args
    shellArgs.insert(executable, at: 0)

    if let (shellPid, childfd) = PseudoTerminalHelpers.fork(andExec: executable, args: shellArgs, env: environment, desiredWindowSize: &size) {
      self.handle = FileHandle.init(fileDescriptor: childfd)
      childMonitor = DispatchSource.makeProcessSource(identifier: shellPid, eventMask: .exit, queue: dispatchQueue)
      childMonitor?.setEventHandler {
        var n: Int32 = 0
        waitpid(self.shellPid, &n, WNOHANG)
        self.delegate.processTerminated(self, exitCode: n)
        self.running = false
      }

      running = true
      self.childfd = childfd
      self.shellPid = shellPid
    }
  }
}
