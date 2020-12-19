//
//  TTY.swift
//  fig
//
//  Created by Matt Schrage on 12/18/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

struct proc {
    let pid: pid_t
    let cmd: String
    var name: String {
        return String(cmd.split(separator: "/").last ?? "") 
    }
    var _cwd: String?
    
    init(line: String) {
        let tokens = line.split(separator: " ")
        pid = Int32(String(tokens[1])) ?? -1
        cmd = tokens.suffix(from: 2).joined(separator: " ")
    }
    
    init(pid: pid_t, cmd: String, cwd: String) {
        self.pid = pid
        self.cmd = cmd
        self._cwd = cwd
    }
    
    var cwd: String? {
        guard let cwd = self._cwd else {
            return "/usr/sbin/lsof -an -p \(self.pid) -d cwd -F n | tail -1 | cut -c2-".runAsCommand().trimmingCharacters(in: .whitespaces)
        }
        
        return cwd
//        return "/usr/sbin/lsof -p \(self.pid) | awk '$4==\"cwd\" { print $9 }'".runAsCommand().trimmingCharacters(in: .whitespaces)
    }
    
    /// Run cat /etc/shells
    var isShell: Bool {
        return (Defaults.processWhitelist + ["zsh","fish","bash", "csh","dash","ksh","tcsh", "ssh"]).reduce(into: false) { (res, shell) in
            res = res || cmd.contains(shell)
        }
    }
}
class TTY {
    let descriptor: String
    init(fd: String) {
        descriptor = fd
        self.update() // running this right away may cause fig to be the current process rather than the shell.
    }
    
    var processes: [proc] {
        
        return ProcessStatus.getProcesses(for: self.descriptor).filter({ (process) -> Bool in
            return !(Defaults.ignoreProcessList.contains(process.cmd) || Defaults.ignoreProcessList.contains(process.name))
        }).reversed()
    }
    
    var running: proc? {
        return processes.last
    }
    
    func update(for pid: pid_t? = nil) {
        guard self.shell == nil else {
            // if shell is set, then updating the list of processes is handled through the integration!
            return
        }
        
        let list = self.processes

        var process: proc? = nil
        if let pid = pid {
//            let split = list.firstIndex { $0.pid == pid }
            process = list.filter { $0.pid == pid }.first // there should only be one
        } else {
            process = list.last
            
        }
        
        guard let runningProcess = process else { return }
        let cmd = runningProcess.cmd
        let cwd = runningProcess.cwd
        print("tty: running \(cmd) \(cwd ?? "<none>")")
        self.cwd = cwd
        self.cmd = cmd
        self.pid = runningProcess.pid
        self.isShell = runningProcess.isShell
    }
    
    var cwd: String?
    var cmd: String?
    var pid: pid_t?
    var isShell: Bool?
    var shell: proc?
    
    func startedNewShellSession(for shellPid: pid_t) {
        self.shell = self.processes.filter { $0.pid == shellPid }.first
        precmd()
    }
    
    func precmd() {
        guard let shell = self.shell else { return }
        let updatedShell = self.processes.filter { $0.pid == shell.pid }.first
        
        
        if let runningProcess = updatedShell {
           self.cwd = runningProcess.cwd
           self.cmd = runningProcess.cmd
           self.pid = runningProcess.pid
           self.isShell = runningProcess.isShell
        }
        
    }
    
    func preexec() {
        Timer.delayWithSeconds(0.01) {
            if let runningProcess = self.processes.last {
                self.cwd = runningProcess.cwd
                self.cmd = runningProcess.cmd
                self.pid = runningProcess.pid
                self.isShell = runningProcess.isShell
            }
        }
    }
    
    func returnedToShellPrompt(for shellPid: pid_t) {
        if let shell = shell, shell.pid == shellPid {
            precmd()
            Logger.log(message: "Returned to shell prompt", priority: .info, subsystem: .tty)
            return
        }
        
        self.startedNewShellSession(for: shellPid)

    }
}

extension TTY: Hashable {
    static func == (lhs: TTY, rhs: TTY) -> Bool {
        return lhs.descriptor == rhs.descriptor
    }
    
    func hash(into hasher: inout Hasher) {
         hasher.combine(self.descriptor)
    }
}
