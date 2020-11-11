//
//  ShellHooksManager.swift
//  fig
//
//  Created by Matt Schrage on 8/28/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

protocol ShellHookService {
    func tether(window: CompanionWindow)
    func untether(window: CompanionWindow)
    func close(window: CompanionWindow)
    func shouldAppear(window: CompanionWindow, explicitlyRepositioned: Bool) -> Bool
    func requestWindowUpdate()
    func isSidebar(window: CompanionWindow) -> Bool

//    func shouldReposition(window: CompanionWindow, explicitlyRepositioned: Bool) -> Bool

}


struct proc {
    let pid: pid_t
    let cmd: String
    
    init(line: String) {
        let tokens = line.split(separator: " ")
        pid = Int32(String(tokens[1])) ?? -1
        cmd = tokens.suffix(from: 2).joined(separator: " ")
    }
    
    var cwd: String? {
        return "/usr/sbin/lsof -an -p \(self.pid) -d cwd -F n | tail -1 | cut -c2-".runAsCommand().trimmingCharacters(in: .whitespaces)
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
        self.update() // running this right away my cause fig to be the current process rather than the shell.
    }
    
    func getProcesses() {
        let _ = "ps | awk '$2==\"\(self.descriptor)\" { print $2, $1, $4 }'".runInBackground { (out) in
//            self.processes = out.split(separator: "\n").map { return proc(line: String($0)) }
        }

    }
    
    var processes: [proc] {
        //let out = "ps -t \(self.descriptor) | awk '{ print $2, $1, $4 }'".runAsCommand()
        let out = "ps | awk '$2==\"\(self.descriptor)\" { print $2, $1, $4 }'".runAsCommand()
        return out.split(separator: "\n").map { return proc(line: String($0)) }
    }
    
    var running: proc? {
        return processes.last
    }
    
    func update() {
        guard let running = self.running else { return }
        let cmd = running.cmd
        let cwd = running.cwd
        print("tty: running \(cmd) \(cwd ?? "<none>")")
        self.cwd = cwd
        self.cmd = cmd
        self.isShell = running.isShell
    }
    
    var cwd: String?
    var cmd: String?
    var isShell: Bool?
}

extension TTY: Hashable {
    static func == (lhs: TTY, rhs: TTY) -> Bool {
        return lhs.descriptor == rhs.descriptor
    }
    
    func hash(into hasher: inout Hasher) {
         hasher.combine(self.descriptor)
    }
}

class ShellHookManager : NSObject {
    static let shared = ShellHookManager()
    var tabs: [CGWindowID: String] = [:]
    var tty: [ExternalWindowHash: TTY] = [:]
    var sessions: [ExternalWindowHash: String] = [:]
    fileprivate var originalWindowHashBySessionId: [String: ExternalWindowHash] = [:]
    
    override init() {
        super.init()
        NotificationCenter.default.addObserver(self, selector: #selector(currentDirectoryDidChange(_:)), name: .currentDirectoryDidChange, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(currentTabDidChange(_:)), name: .currentTabDidChange, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(startedNewTerminalSession(_:)), name: .startedNewTerminalSession, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(shellPromptWillReturn(_:)), name: .shellPromptWillReturn, object: nil)
    }

}

extension Dictionary where Value: Equatable {
    func someKey(forValue val: Value) -> Key? {
        return first(where: { $1 == val })?.key
    }
}

extension ShellHookManager : ShellBridgeEventListener {
    @objc func shellPromptWillReturn(_ notification: Notification) {
        
        // check if this
        let msg = (notification.object as! ShellMessage)
        Logger.log(message: "shellPromptWillReturn")
        
//        if let window = AXWindowServer.shared.whitelistedWindow {
//            self.sessions[window.hash] = msg.session
//        }


        if let windowHash = sessions.someKey(forValue: msg.session) {
            print("tty: shellPromptWillReturn for hash = \(windowHash)")
//            print("Updating tty")
//            
//            // if a tty exists, update it (with delay)
            if let tty = self.tty[windowHash] {
                Timer.delayWithSeconds(0.1) {
                   tty.update()
                }
            }
        } else {
            print("tty: not working...")
            print("tty: in = \(msg.options?.joined(separator: " ") ?? "") ; keys=\(sessions.keys.joined(separator: ", ")).")
            self.startedNewTerminalSession(notification)

        }
    }
    
     @objc func startedNewTerminalSession(_ notification: Notification) {
        let msg = (notification.object as! ShellMessage)
        Timer.delayWithSeconds(0.2) { // add delay so that window is active
            if let window = AXWindowServer.shared.whitelistedWindow {
                if let ttyId = msg.options?.last?.split(separator: "/").last {
                    Logger.log(message: "tty: \(window.hash) = \(ttyId)")
                    Logger.log(message: "session: \(window.hash) = \(msg.session)")

                    let ttys = TTY(fd: String(ttyId))
    //                print("tty: Running directory = ", ttys.running?.cwd)
    //                print("tty: procs = ", ttys.processes.map { $0.cmd }.joined(separator: ", "))

                    self.tty[window.hash] = ttys
                    self.sessions[window.hash] = msg.session
                    // the hash might be missing its tab component (windowId/tab)
                    // so record original
                    self.originalWindowHashBySessionId[msg.session] = window.hash
                } else {
                    print("tty: could not parse!")
                }
            } else {
                print("tty: Terminal created but window is not whitelisted.")

                Logger.log(message: "Terminal created but window is not whitelisted.")
            }
        }
    }
    
    @objc func currentTabDidChange(_ notification: Notification) {
        let msg = (notification.object as! ShellMessage)
        Logger.log(message: "currentTabDidChange")
        if let window = AXWindowServer.shared.whitelistedWindow {
            guard window.bundleId == "com.googlecode.iterm2" else { return }
            if let id = msg.options?.last {
                Logger.log(message: "tab: \(window.windowId)/\(id)")
                tabs[window.windowId] = id
                DispatchQueue.main.async {
                    WindowManager.shared.windowChanged()
                }
            }
            
        }

        
    }
    
    @objc func recievedDataFromPipe(_ notification: Notification) { }
    
    @objc func recievedUserInputFromTerminal(_ notification: Notification) { }
    
    @objc func recievedStdoutFromTerminal(_ notification: Notification) { }
    
    @objc func recievedDataFromPty(_ notification: Notification) { }
    
    @objc func currentDirectoryDidChange(_ notification: Notification) {
        let msg = (notification.object as! ShellMessage)
        
       Logger.log(message: "directoryDidChange:\(msg.session) -- \(msg.env?.jsonStringToDict()?["PWD"] ?? "")")
        
        DispatchQueue.main.async {
            if let window = AXWindowServer.shared.whitelistedWindow {                WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("fig.directoryChanged(`\(msg.env?.jsonStringToDict()?["PWD"] ?? "")`,'\(window.hash)')", completionHandler: nil)

            }
            WindowManager.shared.sidebar?.webView?.evaluateJavaScript("fig.directoryChanged(`\(msg.env?.jsonStringToDict()?["PWD"] ?? "")`)", completionHandler: nil)
        }

    }
    
    
}
