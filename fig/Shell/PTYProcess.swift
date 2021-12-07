//
//  InteractiveProcess.swift
//  fig
//
//  Created by Matt Schrage on 11/15/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class PTYProcess {
  var logFile : String
  var dispatchQueue: DispatchQueue
  var running: Bool = false
  var pid: pid_t = -1
  var fd: Int32 = -1

  func startProcess(executable: String, args: [String], environment: [String]) {
    if running {
      return
    }
    
    var shellArgs = args
    shellArgs.insert(executable, at: 0)
    
    if let (shell_pid, master_fd) = PseudoTerminalHelpers.fork(andExec: executable, args: shellArgs, env: environment) {
      let log_pid = pty_init(master_fd, self.logFile)
      PseudoTerminal.log("C PTY pid \(log_pid)")
      self.pid = shell_pid
      self.fd = master_fd
    }
  }
  
  init(logFile: String, dispatchQueue: DispatchQueue? = nil) {
    self.dispatchQueue = dispatchQueue ?? DispatchQueue.init(label: "InteractiveProcess", qos: .userInitiated)
    self.logFile = logFile
  }
  
  func stop(block: @escaping () -> Void) {
    if (self.pid > -1) {
      pty_free(self.fd, self.pid)
      var n: Int32 = 0
      waitpid(self.pid, &n, 0)
    }
    self.pid = -1
    self.fd = -1
   
    block()
  }
  
  func send(_ input: String, handlerId: String? = nil) {
    PseudoTerminal.log("[SEND-\(handlerId ?? "0")] Queuing data to write: \(input.count)")
    self.dispatchQueue.async { [weak self] in
      guard let strongSelf = self else { return }
      guard strongSelf.fd > -1 else { return }
      let bytesWritten = pty_send(strongSelf.fd, input, Int32(input.utf8.count))
      PseudoTerminal.log("[SEND-\(handlerId ?? "0")] Wrote \(bytesWritten) bytes")
    }
  }
  
  deinit {
    pty_free(self.fd, self.pid)
  }
}
