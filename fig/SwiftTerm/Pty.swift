//
//  File.swift
//  
//
//  Created by Miguel de Icaza on 3/4/20.
//

import Foundation

public class PseudoTerminalHelpers {
    /* Taken from Swift's StdLib: https://github.com/apple/swift/blob/master/stdlib/private/SwiftPrivate/SwiftPrivate.swift */
    static func scan<
      S : Sequence, U
    >(_ seq: S, _ initial: U, _ combine: (U, S.Iterator.Element) -> U) -> [U] {
      var result: [U] = []
      result.reserveCapacity(seq.underestimatedCount)
      var runningResult = initial
      for element in seq {
        runningResult = combine(runningResult, element)
        result.append(runningResult)
      }
      return result
    }

    /* Taken from Swift's StdLib: https://github.com/apple/swift/blob/master/stdlib/private/SwiftPrivate/SwiftPrivate.swift */
    public static func withArrayOfCStrings<R>(
      _ args: [String], _ body: ([UnsafeMutablePointer<CChar>?]) -> R
    ) -> R {
      let argsCounts = Array(args.map { $0.utf8.count + 1 })
      let argsOffsets = [ 0 ] + scan(argsCounts, 0, +)
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

    public static func fork (andExec: String, args: [String], env: [String]) -> (pid: pid_t, masterFd: Int32)? {
        var master: Int32 = 0
        
        let pid = forkpty(&master, nil, nil, nil)
        if pid < 0 {
            return nil
        }
        if pid == 0 {
            withArrayOfCStrings(args, { pargs in
                withArrayOfCStrings(env, { penv in
                    let _ = execve(andExec, pargs, penv)
                })
            })
        }
        return (pid, master)
    }
}
