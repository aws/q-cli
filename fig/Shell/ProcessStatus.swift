//
//  ProcessStatus.swift
//  fig
//
//  Created by Matt Schrage on 11/17/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

typealias TTYDescriptor = String
class ProcessStatus {
    static func getProcesses(for tty: TTYDescriptor? = nil) -> [proc] {
        var size: Int32 = 0
        
        if let ptr = getProcessInfo(tty ?? "", &size) {
            let buffer = UnsafeMutableBufferPointer<fig_proc_info>(start: ptr, count: Int(size))

            let processes = buffer.map { (p) -> proc in
                var process = p

                let cwd = withUnsafeBytes(of: &process.cwd) { (rawPtr) -> String in
                    let ptr = rawPtr.baseAddress!.assumingMemoryBound(to: CChar.self)
                    return String(cString: ptr)
                }

                let cmd = withUnsafeBytes(of: &process.cmd) { (rawPtr) -> String in
                    let ptr = rawPtr.baseAddress!.assumingMemoryBound(to: CChar.self)
                    return String(cString: ptr)
                }

                let tty = withUnsafeBytes(of: &process.tty) { (rawPtr) -> String in
                    let ptr = rawPtr.baseAddress!.assumingMemoryBound(to: CChar.self)
                    return String(cString: ptr)
                }

                print("proc: ",  process.pid, cwd, cmd, tty)
                
                return proc(pid: process.pid, cmd: cmd, cwd: cwd)
            }

            free(ptr)
            return processes
        }
        
        return []
    }
}
