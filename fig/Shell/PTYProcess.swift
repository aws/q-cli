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

    /* Taken from Swift's StdLib: https://github.com/apple/swift/blob/master/stdlib/private/SwiftPrivate/SwiftPrivate.swift */
    public static func withArrayOfCStrings<R>(
      _ args: [String], _ body: ([UnsafeMutablePointer<CChar>?]) -> R
    ) -> R {
        let argsCounts = Array(args.map { $0.utf8.count + 1 })
        let argsOffsets = argsCounts.reduce([ 0 ]) { (result, elm) in result + [result.last! + elm] }
        let argsBufferSize = argsOffsets.last!

        var argsBuffer: [UInt8] = []
        argsBuffer.reserveCapacity(argsBufferSize)
        for arg in args {
            argsBuffer.append(contentsOf: arg.utf8)
            argsBuffer.append(0)
        }

        return argsBuffer.withUnsafeMutableBufferPointer {
            (argsBuffer) in
            let ptr = UnsafeMutableRawPointer(argsBuffer.baseAddress!).bindMemory(
            to: CChar.self, capacity: argsBuffer.count)
            var cStrings: [UnsafeMutablePointer<CChar>?] = argsOffsets.map { ptr + $0 }
            cStrings[cStrings.count - 1] = nil
            return body(cStrings)
        }
    }

    func startProcess(executable: String, args: [String], environment: [String]) {
        if running {
            return
        }
        
        var shellArgs = args
        shellArgs.insert(executable, at: 0)

        var parent: Int32 = 0
        
        let pid = forkpty(&parent, nil, nil, nil)
        guard pid >= 0 else {
            return;
        }

        if pid == 0 {
          PTYProcess.withArrayOfCStrings(shellArgs, { pargs in
              PTYProcess.withArrayOfCStrings(environment, { penv in
                  let _ = execve(executable, pargs, penv)
              })
          })
        }

        pty_init(parent, logFile)
        self.pid = pid
        self.fd = parent
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
