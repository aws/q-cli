//
//  InteractiveProcess.swift
//  fig
//
//  Created by Matt Schrage on 11/15/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class InteractiveProcess {
  fileprivate var process: UnsafeMutablePointer<Pty>?
  init(logFile: String) {
    
    self.process = pty_init(logFile)
  }
  
  func write(_ buffer: String) {
    let bytesWritten = pty_send(self.process, buffer, Int32(buffer.count))
    PseudoTerminal.log("Wrote \(bytesWritten) bytes")
  }
  
  deinit {
    pty_free(self.process)
  }
}
