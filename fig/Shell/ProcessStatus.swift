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
  // C -> Swift Bridging
  static func getProcesses(for tty: TTYDescriptor? = nil) -> [proc] {

    // Checks that tty is descriptor name and not full path
    guard let tty = tty?.split(separator: "/").last else {
      return []
    }

    var size: Int32 = 0
    if let ptr = getProcessInfo(String(tty), &size) {
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
        print("proc: ", process.pid, cwd, cmd, tty)
        return proc(pid: process.pid, cmd: cmd, cwd: cwd, tty: tty)
      }
      free(ptr)
      return processes
    }
    return []
  }

  static func workingDirectory(for pid: Int32) -> String {
    var pathinfo = proc_vnodepathinfo()

    let st = proc_pidinfo(pid,
                          PROC_PIDVNODEPATHINFO,
                          0,
                          &pathinfo,
                          Int32(MemoryLayout<proc_vnodepathinfo>.size))

    guard st > 0 else {
      Logger.log(message: "Error \(st): could not retrieve working directory for pid (\(pid)")
      return "/"
    }

    return withUnsafePointer(to: pathinfo.pvi_cdir.vip_path) {
      $0.withMemoryRebound(to: CChar.self, capacity: MemoryLayout.size(ofValue: $0)) {
        String(cString: $0)
      }
    }
  }
}
