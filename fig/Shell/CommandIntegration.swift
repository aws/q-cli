//
//  CommandIntegration.swift
//  fig
//
//  Created by Matt Schrage on 1/12/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

protocol CommandIntegration {
    func update(tty: TTY, for process: proc)
    func runUsingPrefix() -> String?
    static var command: String { get }

}

class SSHIntegration: CommandIntegration {

    static let command = "ssh"
    
    func runUsingPrefix() -> String? {
        if let controlPath = self.controlPath {
            //-o KbdInteractiveAuthentication=no -o ChallengeResponseAuthentication=no -o BatchMode=yes
            return "ssh -o PasswordAuthentication=no -q -o 'ControlPath=\(controlPath)' dest "
        }
        
        return nil
    }
    
    func update(tty: TTY, for process: proc) {
        let semaphore = DispatchSemaphore(value: 0)
        if tty.pty == nil {
            tty.pty = PseudoTerminalHelper()
            tty.pty?.start(with: [:])
        }
        
        let scriptPath = Bundle.main.path(forResource: "remote_cwd", ofType: "sh")!
        guard let prefix = self.runUsingPrefix() else {
            return
        }
        
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
    
    func newConnection(with info: ShellMessage, in tty: TTY) {
        // fig bg:ssh ~/.ssh/tmp/...
        self.controlPath = info.arguments.first
        tty.update()
    }
    
    var controlPath: String?
}

//class DockerIntegration: CommandIntegration {
//    func update(tty: TTY, for process: proc) {
//        let connection = lsof.arguments(fromPid: runningProcess.pid)
//
//    }
//
//    func runUsingPrefix() -> String? {
//
//    }
//
//    static var command = "docker"
//
//
//}
