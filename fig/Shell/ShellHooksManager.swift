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
        return "/usr/sbin/lsof -p \(self.pid) | awk '$4==\"cwd\" { print $9 }'".runAsCommand()
    }
    
    var isShell: Bool {

        return ["zsh","fish","bash"].reduce(into: false) { (res, shell) in
            res = res || cmd.contains(shell)
        }
    }
}
struct TTY {
    let descriptor: String
    init(fd: String) {
        descriptor = fd
    }
    
    var processes: [proc] {
        let out = "ps | awk '$2==\"\(self.descriptor)\" { print $2, $1, $4 }'".runAsCommand()
        return out.split(separator: "\n").map { return proc(line: String($0)) }
    }
    
    var running: proc? {
        return processes.last
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

class ShellHookManager : NSObject {
    static let shared = ShellHookManager()
    var tabs: [CGWindowID: String] = [:]
    var tty: [ExternalWindowHash: TTY] = [:]
    var sessions: [ExternalWindowHash: String] = [:]
    override init() {
        super.init()
        NotificationCenter.default.addObserver(self, selector: #selector(currentDirectoryDidChange(_:)), name: .currentDirectoryDidChange, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(currentTabDidChange(_:)), name: .currentTabDidChange, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(startedNewTerminalSession(_:)), name: .startedNewTerminalSession, object: nil)
    }

}

extension ShellHookManager : ShellBridgeEventListener {
     @objc func startedNewTerminalSession(_ notification: Notification) {
        let msg = (notification.object as! ShellMessage)
        if let window = WindowServer.shared.topmostWhitelistedWindow() {
            if let ttyId = msg.options?.last?.split(separator: "/").last {
                print("tty: \(window.hash) = \(ttyId)")
                let ttys = TTY(fd: String(ttyId))
//                print("tty: Running directory = ", ttys.running?.cwd)
//                print("tty: procs = ", ttys.processes.map { $0.cmd }.joined(separator: ", "))

                tty[window.hash] = ttys
                sessions[window.hash] = msg.session
            }
        }
    }
    
    @objc func currentTabDidChange(_ notification: Notification) {
        let msg = (notification.object as! ShellMessage)
        if let window = WindowServer.shared.topmostWhitelistedWindow() {
            guard window.bundleId == "com.googlecode.iterm2" else { return }
            if let id = msg.options?.last {
                print("tab: \(window.windowId)/\(id)")
                tabs[window.windowId] = id
                WindowManager.shared.windowChanged()
            }
            
        }

        
    }
    
    @objc func recievedDataFromPipe(_ notification: Notification) { }
    
    @objc func recievedUserInputFromTerminal(_ notification: Notification) { }
    
    @objc func recievedStdoutFromTerminal(_ notification: Notification) { }
    
    @objc func recievedDataFromPty(_ notification: Notification) { }
    
    @objc func currentDirectoryDidChange(_ notification: Notification) {
        let msg = (notification.object as! ShellMessage)
        
        print("directoryDidChange:\(msg.session) -- \(msg.env?.jsonStringToDict()?["PWD"] ?? "")")
        
        DispatchQueue.main.async {
            if let window = WindowServer.shared.topmostWhitelistedWindow() {
                let tab = self.tabs[window.windowId];
                
                WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("fig.directoryChanged(`\(msg.env?.jsonStringToDict()?["PWD"] ?? "")`,'\(window.windowId)/\(tab ?? "")')", completionHandler: nil)

            }
            WindowManager.shared.sidebar?.webView?.evaluateJavaScript("fig.directoryChanged(`\(msg.env?.jsonStringToDict()?["PWD"] ?? "")`)", completionHandler: nil)
        }

    }
    
    
}
