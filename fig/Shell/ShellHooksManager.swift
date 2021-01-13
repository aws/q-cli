//
//  ShellHooksManager.swift
//  fig
//
//  Created by Matt Schrage on 8/28/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

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
        return ShellHookManager.shared.sessions[self]
    }
}

class ShellHookManager : NSObject {
    static let shared = ShellHookManager()
    var tabs: [CGWindowID: String] = [:]
    var tty: [ExternalWindowHash: TTY] = [:]
    var sessions = BiMap<String>()

}

extension Dictionary where Value: Equatable {
    func someKey(forValue val: Value) -> Key? {
        return first(where: { $1 == val })?.key
    }
}

extension ShellHookManager {
    
    func currentTabDidChange(_ info: ShellMessage) {
        Logger.log(message: "currentTabDidChange")
        
        // Need time for whitelisted window to change
        Timer.delayWithSeconds(0.1) {
            if let window = AXWindowServer.shared.whitelistedWindow {
                // So far, only iTerm has a tab integration.
                guard window.bundleId == "com.googlecode.iterm2" else { return }
                if let id = info.options?.last {
                    Logger.log(message: "tab: \(window.windowId)/\(id)")
                    self.tabs[window.windowId] = id
                    
                    self.updateHashMetadata(oldHash: "\(window.windowId)/", hash: window.hash)
                    
                    DispatchQueue.main.async {
                        WindowManager.shared.windowChanged()
                    }
                }
            }
        }
    }
    
    func updateHashMetadata(oldHash: ExternalWindowHash, hash: ExternalWindowHash) {
        guard oldHash != hash else { return }
        guard let device = self.tty[oldHash] else { return }
        guard let sessionId = self.sessions[oldHash] else { return }
        
        // remove out-of-date values
        self.tty.removeValue(forKey: oldHash)
        self.sessions[oldHash] = nil
        
        // reassign tty to new hash
        self.sessions[hash] = sessionId
        self.tty[hash] = device
        
        Logger.log(message: "Transfering \(oldHash) metadata to \(hash).", priority: .info, subsystem: .tty)

    }
            
    func currentDirectoryDidChange(_ info: ShellMessage) {
        let workingDirectory = info.getWorkingDirectory() ?? ""

        Logger.log(message: "directoryDidChange:\(info.session) -- \(workingDirectory)")
        
        // We used to pass this to javascript. Now working directory is determined using tty/shellPid

    }
    
    
    func shellPromptWillReturn(_ info: ShellMessage) {

        guard let (shellPid, ttyDescriptor, sessionId) = info.parseShellHook() else {
            Logger.log(message: "Could not parse out shellHook metadata", priority: .notify, subsystem: .tty)
            return
        }
        
        // try to find associated window, but don't necessarily link with the topmost window! (prompt can return when window is in background)
        guard let hash = attemptToFindToAssociatedWindow(for: sessionId,
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
        // This doesn't work because of timing issues. If the user types too quickly, the first keypress will be overwritten.
        // KeypressProvider.shared.keyBuffer(for: hash).buffer = ""
    }
    
    func startedNewShellSession(_ info: ShellMessage) {
        guard let (shellPid, ttyDescriptor, sessionId) = info.parseShellHook() else {
            Logger.log(message: "Could not parse out shellHook metadata", priority: .notify, subsystem: .tty)
            return
        }

        
        guard let hash = attemptToFindToAssociatedWindow(for: sessionId) else {
            Logger.log(message: "Could not link to window on new shell session.", priority: .notify, subsystem: .tty)
            return
        }
        
        
        // window hash is valid, we should have an associated TTY (or we can create it)
        let tty = self.tty[hash] ?? link(sessionId, hash, ttyDescriptor)
        tty.startedNewShellSession(for: shellPid)

    }
    
    func startedNewTerminalSession(_ info: ShellMessage) {

        guard let (shellPid, ttyDescriptor, sessionId) = info.parseShellHook() else {
            Logger.log(message: "Could not parse out shellHook metadata", priority: .notify, subsystem: .tty)
            return
        }
        
        // We need to wait for window to appear if the terminal emulator is being launched for the first time. Can this be handled more robustly?
        Timer.delayWithSeconds(0.2) {
            
            guard let window = AXWindowServer.shared.whitelistedWindow else {
                Logger.log(message: "Cannot track a new terminal session if topmost window isn't whitelisted.", priority: .notify, subsystem: .tty)
                return
            }
            
            let tty = self.link(sessionId, window.hash, ttyDescriptor)
            tty.startedNewShellSession(for: shellPid)
        }

    }
    
    func shellWillExecuteCommand(_ info: ShellMessage) {

        guard let (_, ttyDescriptor, sessionId) = info.parseShellHook() else {
            Logger.log(message: "Could not parse out shellHook metadata", priority: .notify, subsystem: .tty)
            return
        }
        
        guard let hash = attemptToFindToAssociatedWindow(for: sessionId,
                                                         currentTopmostWindow: AXWindowServer.shared.whitelistedWindow) else {
            
            Logger.log(message: "Could not link to window on new terminal session.", priority: .notify, subsystem: .tty)
            return
        }
        
        let tty = self.tty[hash] ?? link(sessionId, hash, ttyDescriptor)
        tty.preexec()
    }
    
    func startedNewSSHConnection(_ info: ShellMessage) {
        guard Defaults.SSHIntegrationEnabled else {
            Logger.log(message: "SSH Integration is not enabled", priority: .notify)
            return
        }
        
        guard let hash = attemptToFindToAssociatedWindow(for: info.session) else {
              Logger.log(message: "Could not link to window on new shell session.", priority: .notify, subsystem: .tty)
              return
          }
        
        guard let tty = self.tty[hash] else { return }
        guard let sshIntegration = tty.integrations["ssh"] as? SSHIntegration else { return }
        sshIntegration.newConnection(with: info, in: tty)

    }
    
}


extension ShellHookManager {
    
    fileprivate func attemptToFindToAssociatedWindow(for sessionId: SessionId, currentTopmostWindow: ExternalWindow? = nil) -> ExternalWindowHash? {
        
        if let hash = getWindowHash(for: sessionId) {
            guard !validWindowHash(hash) else {
                // the hash is valid and is linked to a session
                Logger.log(message: "WindowHash '\(hash)' is valid", subsystem: .tty)
                
                
                Logger.log(message: "WindowHash '\(hash)' is linked to sessionId '\(sessionId)'", subsystem: .tty)
                return hash
                

            }
            
            // hash exists, but is invalid (eg. should have tab component and it doesn't)
            
            Logger.log(message: "\(hash) is not a valid window hash, attempting to find previous value", priority: .info, subsystem: .tty)
            
//            // clean up this out-of-date hash
            self.sessions[hash] = nil
            self.tty.removeValue(forKey: hash)

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
        
        Logger.log(message: "Found WindowHash '\(hash)' for sessionId '\(sessionId)'", subsystem: .tty)
        return hash
            

    }
    
    fileprivate func link(_ sessionId: SessionId, _ hash: ExternalWindowHash, _ ttyDescriptor: TTYDescriptor) -> TTY {
        let device = TTY(fd: ttyDescriptor)
        

        // tie tty & sessionId to windowHash
        self.tty[hash] = device
        self.sessions[hash] = sessionId // sessions is bidirectional between sessionId and windowHash
        
        return device
    }
    
    fileprivate func getSessionId(for windowHash: ExternalWindowHash) -> SessionId? {
        return self.sessions[windowHash]
    }
    
    fileprivate func getWindowHash(for sessionId: SessionId) -> ExternalWindowHash? {
        return self.sessions[sessionId]
    }
    
    func validWindowHash(_ hash: ExternalWindowHash) -> Bool {
        guard let components = hash.components() else { return false }
        let windowHasNoTabs = (tabs[components.windowId] == nil && components.tab == nil)
        let windowHasTabs = (tabs[components.windowId] != nil && components.tab != nil)
        return windowHasNoTabs || windowHasTabs
    }
}
