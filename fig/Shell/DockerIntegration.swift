//
//  DockerIntegration.swift
//  fig
//
//  Created by Matt Schrage on 1/28/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class DockerIntegration: CommandIntegration {
    static var command = "com.docker.cli"
    var container: String?
    func runUsingPrefix() -> String? {
        if let container = container {
            return "docker"
        }
        
        return nil
    }

    func update(tty: TTY, for process: proc) {
        if tty.pty == nil {
            print("Starting PTY...!")
            tty.pty = PseudoTerminalHelper()
            tty.pty?.start(with: [:])
            return
        }
        
        let semaphore = DispatchSemaphore(value: 0)

        
        let scriptPath = Bundle.main.path(forResource: "remote_cwd", ofType: "sh")!
        guard let prefix = self.runUsingPrefix() else {
            return
        }
      
        let connection = lsof.arguments(fromPid: process.pid)
        print(connection)
      
        tty.pty!.execute("\(prefix) bash -s < \(scriptPath)") { output in
            print("remote_machine:", output)
            guard tty.pid == process.pid else {
                print("Process out of sync, abort update")
                semaphore.signal()
                return
            }
            tty.cwd = output
            tty.cmd = process.cmd
            tty.pid = process.pid
            tty.isShell = process.isShell
            tty.runUsingPrefix = prefix
            semaphore.signal()
        }
        semaphore.wait()
        

    }



}
