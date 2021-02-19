//
//  SSHIntegration.swift
//  fig
//
//  Created by Matt Schrage on 1/28/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class SSHIntegration: CommandIntegration {

    static let command = "ssh"
    static func install() {
        if let scriptPath = Bundle.main.path(forResource: "ssh", ofType: "sh") {
            Defaults.SSHIntegrationEnabled = true
            let out = "/bin/bash '\(scriptPath)'".runAsCommand()
            print(out)
        }
    }
    
    func runUsingPrefix() -> String? {
        if let controlPath = self.controlPath, Defaults.SSHIntegrationEnabled {
            //-o KbdInteractiveAuthentication=no -o ChallengeResponseAuthentication=no -o BatchMode=yes
            return "ssh -o PasswordAuthentication=no -q -o 'ControlPath=\(controlPath)' dest "
        }
        
        return nil
    }
    

    func update(tty: TTY, for process: proc) {
       guard Defaults.SSHIntegrationEnabled else {
            Logger.log(message: "SSH Integration is not enabled", priority: .notify)
            tty.cwd = process.cwd
            tty.cmd = process.cmd
            tty.pid = process.pid
            tty.isShell = process.isShell
            tty.runUsingPrefix = nil
            return
        }
      
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
  
    func initialize(tty: TTY) {
      
    }
  
    func shouldHandleProcess(_ process: proc) -> Bool {
      return process.name == SSHIntegration.command
    }
    
    func newConnection(with info: ShellMessage, in tty: TTY) {
        // fig bg:ssh ~/.ssh/tmp/...
        self.controlPath = info.arguments.first
        tty.update()
    }
    
    var controlPath: String?
}
