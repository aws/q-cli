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
        guard let shellPidStr = msg.options?[safe: 1], let shellPid = Int32(shellPidStr) else { return }

//        self.beginTrackingShellSession(with: <#T##pid_t#>, in: ttyId)
        if let ttyId = ttyId {
            // This delay is added because when a new terminal window is created, we recieve this event before
            Timer.delayWithSeconds(0.2) {
                self.startedNewTerminalSession(sessionId: msg.session, ttyDescriptor: String(ttyId), shellPid: shellPid)
            }
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
            Logger.log(message: "No window included when attempting to link to TTY, don't link!", priority: .info, subsystem: .tty)
            return nil
        }
        
        let hash = window.hash
        let sessionIdForWindow = getSessionId(for: hash)
        
        guard sessionIdForWindow == nil else {
            // a different session Id is already associated with window, don't link!
            Logger.log(message: "A different session Id is already associated with window, don't link!", priority: .info, subsystem: .tty)
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

        // try to find associated window, but don't necessarily link with the topmost window! (prompt can return when window is in background)
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
        // This doesn't work because of timing issues. If the user types to quickly, the first keypress will be overwritten.
        // KeypressProvider.shared.keyBuffer(for: hash).buffer = ""
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
        
        let tty = link(sessionId, hash, ttyDescriptor)

        tty.startedNewShellSession(for: shellPid)

    }
    
    func shellWillExecuteCommand(_ msg: ShellMessage) {

        guard let ttyId = msg.options?[safe: 2]?.split(separator: "/").last else { return }
        guard let shellPidStr = msg.options?[safe: 1], let shellPid = Int32(shellPidStr) else { return }
        
        guard let hash = attemptToLinkToAssociatedWindow(for: msg.session,
                                                         currentTopmostWindow: AXWindowServer.shared.whitelistedWindow) else {
            
            Logger.log(message: "Could not link to window on new terminal session.", priority: .notify, subsystem: .tty)
            return
        }
        
        let tty = self.tty[hash] ?? link(msg.session, hash, String(ttyId))
        
        tty.preexec()
    }
    
    func validWindowHash(_ hash: ExternalWindowHash) -> Bool {
        guard let components = hash.components() else { return false }
        let windowHasNoTabs = (tabs[components.windowId] == nil && components.tab == nil)
        let windowHasTabs = (tabs[components.windowId] != nil && components.tab != nil)
        return windowHasNoTabs || windowHasTabs
    }
}
