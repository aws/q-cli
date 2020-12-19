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
            return !Defaults.ignoreProcessList.contains(process.cmd)
        }).reversed()
    }
    
    var running: proc? {
        return processes.last
    }
    
    func update(for pid: pid_t? = nil) {
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
        self.isShell = runningProcess.isShell
    }
    
    var cwd: String?
    var cmd: String?
    var isShell: Bool?
    var shell: proc?
    
    func startedNewShellSession(for shellPid: pid_t) {
        self.shell = self.processes.filter { $0.pid == shellPid }.first
    }
    
    func returnedToShellPrompt(for shellPid: pid_t) {
        if let shell = shell, shell.pid == shellPid {
            Logger.log(message: "Returned to shell prompt", priority: .info, subsystem: .tty)
            return
        }
        
        self.update(for: shellPid)
        // set self.shell equal to this process
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
extension ExternalWindowHash {
    func components() -> (windowId: CGWindowID, tab: String?)? {
        let tokens = self.split(separator: "/")
        guard let windowId = CGWindowID(tokens[0]) else { return nil }
        let tabString = tokens[safe: 1]
        var tab: String? = nil
        if (tabString != nil) {
            tab = String(tabString!)
        }
        
        return (windowId: windowId, tab: tab)
    }
}

typealias SessionId = String
extension SessionId {
    var isLinked: Bool {
        return self.associatedWindowHash != nil
    }
    
    var associatedWindowHash: ExternalWindowHash? {
        return ShellHookManager.shared.sessions[self]//ShellHookManager.shared.sessions.someKey(forValue: self)
    }
}
class ShellHookManager : NSObject {
    static let shared = ShellHookManager()
    var tabs: [CGWindowID: String] = [:]
    var tty: [ExternalWindowHash: TTY] = [:]
    var sessions = BiMap<String>()// = [:]//[ExternalWindowHash: SessionId] = [:]
    
    fileprivate var originalWindowHashBySessionId: [SessionId: ExternalWindowHash] = [:]
    
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
    
    // Note: shell prompt can return when window is not active window. This is why we get the windowHash using the session identifier.
    func beginTrackingShellSession(with shellPid: pid_t, in tty: String) {
        // there probably needs to be a delay
        
        // get current whitelisted window
        guard let window = AXWindowServer.shared.whitelistedWindow else {
            Logger.log(message: "Could find whitelisted window when tracking new shell session", subsystem: .tty)
            return
        }
        
        let device = TTY(fd: tty)
        device.startedNewShellSession(for: shellPid)
        
//        self.tty[window.hash] = device
//        self.sessions[window.hash] = msg.session
//        self.originalWindowHashBySessionId[msg.session] = window.hash


        
    }
    
    func beginTrackingTerminalSession() {
        
    }
    
    @objc func shellPromptWillReturn(_ notification: Notification) {
        

        let msg = (notification.object as! ShellMessage)
        Logger.log(message: "shellPromptWillReturn")
        guard let ttyId = msg.options?[safe: 2]?.split(separator: "/").last else { return }
        guard let shellPidStr = msg.options?[safe: 1], let shellPid = Int32(shellPidStr) else { return }

        self.shellPromptWillReturn(sessionId: msg.session, ttyDescriptor: String(ttyId), shellPid: shellPid)
        return
        
//        if let window = AXWindowServer.shared.whitelistedWindow {
//            self.sessions[window.hash] = msg.session
//        }


        // Be careful because an old window hash can be returned (eg. 1323/ instead of 1232/1)
        if let windowHash = sessions[msg.session]/*sessions.someKey(forValue: msg.session)*/ {
            print("tty: shellPromptWillReturn for hash = \(windowHash)")
//            print("Updating tty")
            
            // check if valid window hash. (
            guard let components = windowHash.components() else { return }
            let windowHasNoTabs = (tabs[components.windowId] == nil && components.tab == nil)
            let windowHasTabs = (tabs[components.windowId] != nil && components.tab != nil)
            let validHash = windowHasNoTabs || windowHasTabs
            
            if (validHash) {
                // if a tty exists, update it (with delay)
                if let tty = self.tty[windowHash] {
                    guard let shellPidStr = msg.options?[safe: 1], let shellPid = Int32(shellPidStr) else { return }
                    tty.update(for: shellPid)
                    tty.returnedToShellPrompt(for: shellPid)
                    KeypressProvider.shared.keyBuffer(for: windowHash).buffer = ""
                    
                }
            } else {
                // remove tty for hash (because the hash is invalid)
                self.tty.removeValue(forKey: windowHash)
                self.startedNewTerminalSession(notification)

            }
            

        } else {
            print("tty: not working...")
//            print("tty: in = \(msg.options?.joined(separator: " ") ?? "") ; keys=\(sessions.keys.joined(separator: ", ")).")
            self.startedNewTerminalSession(notification)

        }
    }
    
     @objc func startedNewTerminalSession(_ notification: Notification) {
        let msg = (notification.object as! ShellMessage)
        let ttyId = msg.options?[safe: 2]?.split(separator: "/").last
        
//        self.beginTrackingShellSession(with: <#T##pid_t#>, in: ttyId)
        if let ttyId = ttyId {
            self.startedNewShellSession(sessionId: msg.session, ttyDescriptor: String(ttyId), shellPid: 0)

        }
//        Timer.delayWithSeconds(0.2) { // add delay so that window is active
//            if let window = AXWindowServer.shared.whitelistedWindow {
//                if let ttyId = ttyId {
//                    Logger.log(message: "tty: \(window.hash) = \(ttyId)")
//                    Logger.log(message: "session: \(window.hash) = \(msg.session)")
//
//                    let ttys = TTY(fd: String(ttyId))
//    //                print("tty: Running directory = ", ttys.running?.cwd)
//    //                print("tty: procs = ", ttys.processes.map { $0.cmd }.joined(separator: ", "))
//
//                    self.tty[window.hash] = ttys
//                    self.sessions[window.hash] = msg.session
//                    // the hash might be missing its tab component (windowId/tab)
//                    // so record original
//                    self.originalWindowHashBySessionId[msg.session] = window.hash
//                } else {
//                    print("tty: could not parse!")
//                }
//            } else {
//                print("tty: Terminal created but window is not whitelisted.")
//
//                Logger.log(message: "Terminal created but window is not whitelisted.")
//            }
//        }
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
    
    func attemptToLinkToAssociatedWindow(for sessionId: SessionId, currentTopmostWindow: ExternalWindow? = nil) -> ExternalWindowHash? {
        
        if let hash = getWindowHash(for: sessionId) {
            guard !validWindowHash(hash) else {
                // the hash is valid and is linked to a session
                return hash
            }
            
            // hash exists, but is invalid (eg. should have tab component and it doesn't)
            
            // clean up this out-of-date hash
            self.tty.removeValue(forKey: hash)
            self.sessions[hash] = nil
            
        }
        
        // user had this terminal session up prior to launching Fig or has iTerm tab integration set up, which caused original hash to go stale (eg. 16356/ -> 16356/1)
        
        // hash does not exist
        
        // so, lets see if the top window is a supported terminal
        guard let window = currentTopmostWindow else {
            // no terminal window found or passed in, don't link!
            return nil
        }
        
        let hash = window.hash
        let sessionIdForWindow = getSessionId(for: hash)
        
        guard sessionIdForWindow == nil else {
            // a different session Id is already associated with window, don't link!
            return nil
        }
        
//        let _ = link(sessionId, hash, ttyDescriptor)
        
        return hash
            

    }
    
    func link(_ sessionId: SessionId, _ hash: ExternalWindowHash, _ ttyDescriptor: String) -> TTY {
        let device = TTY(fd: ttyDescriptor)


        // tie tty & sessionId to windowHash
        self.tty[hash] = device
        self.sessions[hash] = sessionId
        
        return device

    }
    
    func getSessionId(for windowHash: ExternalWindowHash) -> SessionId? {
        return self.sessions[windowHash]
    }
    
    func getWindowHash(for sessionId: SessionId) -> ExternalWindowHash? {
        return self.sessions[sessionId]
    }
    
    func shellPromptWillReturn(sessionId: SessionId, ttyDescriptor: String, shellPid: pid_t) {

        guard let hash = attemptToLinkToAssociatedWindow(for: sessionId,
                                                         currentTopmostWindow: AXWindowServer.shared.whitelistedWindow) else {
            Logger.log(message: "Could not link to window on shell prompt return.", priority: .notify, subsystem: .tty)
            return
        }
        
        // window hash is valid, we should have an associated TTY (or we can create it)
        let tty = self.tty[hash] ?? link(sessionId, hash, ttyDescriptor)
        
        // Window is linked with TTY session
        // update tty's active process to current shell
        tty.returnedToShellPrompt(for: shellPid)
        
        // if the user has returned to the shell, their keypress buffer must be reset (for instance, if they exited by pressing 'q' rather than return)
        KeypressProvider.shared.keyBuffer(for: hash).buffer = ""
    }
    
    func startedNewShellSession(sessionId: SessionId, ttyDescriptor: String, shellPid: pid_t) {
        // get tty by sessionID
//        guard let windowHash = sessionId.associatedWindowHash else {
//            Logger.log(message: "sessionId '\(sessionId)' has no associated windowHash", priority: .info, subsystem: .tty)
//            return
//        }
//

        
        guard let hash = attemptToLinkToAssociatedWindow(for: sessionId) else {
            Logger.log(message: "Could not link to window on new shell session.", priority: .notify, subsystem: .tty)
            return
        }
        
        
        // window hash is valid, we should have an associated TTY (or we can create it)
        let tty = self.tty[hash] ?? link(sessionId, hash, ttyDescriptor)
        
        tty.startedNewShellSession(for: shellPid)

    }
    
    func startedNewTerminalSession(sessionId: SessionId, ttyDescriptor: String, shellPid: pid_t) {
    
        guard let window = AXWindowServer.shared.whitelistedWindow else {
            Logger.log(message: "Cannot track a new terminal session if topmost window isn't whitelisted.", priority: .notify, subsystem: .tty)
            return
        }
        
        guard let hash = attemptToLinkToAssociatedWindow(for: sessionId,
                                                         currentTopmostWindow: window) else {
            
            Logger.log(message: "Could not link to window on new terminal session.", priority: .notify, subsystem: .tty)
            return
        }
        
        let _ = link(sessionId, hash, ttyDescriptor)

//        tty.startedNewShellSession(for: shellPid)

    }
    
    func validWindowHash(_ hash: ExternalWindowHash) -> Bool {
        guard let components = hash.components() else { return false }
        let windowHasNoTabs = (tabs[components.windowId] == nil && components.tab == nil)
        let windowHasTabs = (tabs[components.windowId] != nil && components.tab != nil)
        return windowHasNoTabs || windowHasTabs
    }
}
