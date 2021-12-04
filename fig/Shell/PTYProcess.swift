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
  fileprivate var process: UnsafeMutablePointer<Pty>?
  var dispatchQueue: DispatchQueue
  var running: Bool = false

  func startProcess(executable: String, args: [String], environment: [String]) {
    if running {
      return
    }
    
    var shellArgs = args
    shellArgs.insert(executable, at: 0)

    PseudoTerminalHelpers.withArrayOfCStrings(shellArgs) { pargs in
        PseudoTerminalHelpers.withArrayOfCStrings(environment) { penv in
          if let process = pty_init(executable, pargs, penv, self.logFile) {
                self.process = process
                self.running = true
            }
        }
    }
  }
  
  init(logFile: String, dispatchQueue: DispatchQueue? = nil) {
    self.dispatchQueue = dispatchQueue ?? DispatchQueue.init(label: "InteractiveProcess", qos: .userInitiated)
    self.logFile = logFile
  }
  
  func stop(block: @escaping () -> Void) {
    if let old_pid = self.process?.pointee.process_pid {
      pty_free(self.process)
      self.process = nil
      var n: Int32 = 0
      waitpid(old_pid, &n, 0)
      self.running = false
    }
    block()
  }
  
  func send(_ input: String, handlerId: String? = nil) {
    PseudoTerminal.log("[SEND-\(handlerId ?? "0")] Queuing data to write: \(input.count)")
    self.dispatchQueue.async { [weak self] in
      guard let strongSelf = self else { return }
      guard let process = strongSelf.process else { return }
      let bytesWritten = pty_send(process, input, Int32(input.utf8.count))
      PseudoTerminal.log("[SEND-\(handlerId ?? "0")] Wrote \(bytesWritten) bytes")
    }
  }
  
  deinit {
    guard let process = self.process else { return }
    pty_free(process)
    self.process = nil
  }
}
