//
//  ShellHooksManager.swift
//  fig
//
//  Created by Matt Schrage on 8/28/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa

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
    fileprivate var tabs: [CGWindowID: String] = [:]
    fileprivate var tty: [ExternalWindowHash: TTY] = [:]
    fileprivate var sessions = BiMap<String>()
  
    private let queue = DispatchQueue(label: "com.withfig.shellhooks", attributes: .concurrent)
  
    fileprivate var observer: WindowObserver?
    fileprivate let semaphore = DispatchSemaphore(value: 1)

}

// handle concurrency
extension ShellHookManager {

  func tab(for windowID: CGWindowID) -> String? {
    return self.tabs[windowID]
    var tab: String?
    queue.sync {
      tab = self.tabs[windowID]
    }
    return tab
  }
  
  func setActiveTab(_ tab: String, for windowID: CGWindowID) {
    self.tabs[windowID] = tab
    return
    
//    queue.async(flags: [.barrier]) {
//      self.tabs[windowID] = tab
//    }
  }
  
  func ttys() -> [ExternalWindowHash: TTY] {
    return self.tty
//    var ttys: [ExternalWindowHash: TTY]!
//    queue.sync {
//      ttys = self.tty
//    }
//    return ttys
  }
  
  func tty(for windowHash: ExternalWindowHash) -> TTY? {
    return self.tty[windowHash]
//    var tty: TTY?
//    queue.sync {
//      tty = self.tty[windowHash]
//    }
//    return tty
  }
  
  func setTTY(_ tty:TTY, for window: ExternalWindowHash) {
    self.tty[window] = tty
    return
    
//    queue.sync(flags: [.barrier]) {
//      self.tty[window] = tty
//    }
  }
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
                if let id = info.options?.last {
                  let VSCodeTerminal = window.bundleId == "com.microsoft.VSCode" && id.hasPrefix("code:")
                  let HyperTab = window.bundleId == "co.zeit.hyper" &&  id.hasPrefix("hyper:")
                  let iTermTab = window.bundleId == "com.googlecode.iterm2" && !id.hasPrefix("code:") && !id.hasPrefix("hyper:")
                  guard VSCodeTerminal || iTermTab || HyperTab else { return }
                    Logger.log(message: "tab: \(window.windowId)/\(id)")
//                    self.tabs[window.windowId] = id
                    self.setActiveTab(id, for: window.windowId)
                  
                    self.updateHashMetadata(oldHash: "\(window.windowId)/", hash: window.hash)
                    
                    DispatchQueue.main.async {
                      
                        // If leaving visor mode in iTerm, we need to manually check which window is on top
//                        if let app = NSWorkspace.shared.frontmostApplication {
//                            AXWindowServer.shared.register(app, fromActivation: true)
//                        }

                        WindowManager.shared.windowChanged()
                    }
                }
            }
        }
    }
    
    func updateHashMetadata(oldHash: ExternalWindowHash, hash: ExternalWindowHash) {
        
        //queue.async(flags: [.barrier]) {
            guard oldHash != hash else { return }
            guard let device = self.tty[oldHash] else { return }
            guard let sessionId = self.sessions[oldHash] else { return }
            
            // remove out-of-date values
            self.tty.removeValue(forKey: oldHash)
            self.sessions[oldHash] = nil
            
            // reassign tty to new hash
            self.sessions[hash] = sessionId
            self.tty[hash] = device
        //}
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
        let tty = self.tty(for: hash) ?? link(sessionId, hash, ttyDescriptor)
        
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
        let tty = self.tty(for: hash) ?? link(sessionId, hash, ttyDescriptor)
        tty.startedNewShellSession(for: shellPid)
            
        KeypressProvider.shared.keyBuffer(for: hash).backedByZLE = false

    }
    
    func startedNewTerminalSession(_ info: ShellMessage) {

        guard let (shellPid, ttyDescriptor, sessionId) = info.parseShellHook() else {
            Logger.log(message: "Could not parse out shellHook metadata", priority: .notify, subsystem: .tty)
            return
        }
      
        guard let bundleId = NSWorkspace.shared.frontmostApplication?.bundleIdentifier else {
          Logger.log(message: "Could not get bundle id", priority: .notify, subsystem: .tty)
          return
        }
        
        var delay:TimeInterval? = 0.2
      
        if Integrations.Hyper == bundleId {
            delay = 2
        }
      
        observer = WindowObserver(with: bundleId)
      
        // We need to wait for window to appear if the terminal emulator is being launched for the first time. Can this be handled more robustly?
        observer?.windowDidAppear(timeoutAfter: delay, completion: {
            // ensuring window bundleId & frontmostApp bundleId match fixes case where a slow launching application (eg. Hyper) will init shell before window is visible/tracked
            Logger.log(message: "Awaited window did appear", priority: .notify, subsystem: .tty)

            guard let window = AXWindowServer.shared.whitelistedWindow, window.bundleId == NSWorkspace.shared.frontmostApplication?.bundleIdentifier
                else {
                Logger.log(message: "Cannot track a new terminal session if topmost window isn't whitelisted.", priority: .notify, subsystem: .tty)
                return
            }
          
            Logger.log(message: "Linking \(ttyDescriptor) with \(window.hash) for \(sessionId)", priority: .notify, subsystem: .tty)

            let tty = self.link(sessionId, window.hash, ttyDescriptor)
            tty.startedNewShellSession(for: shellPid)
        })

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
        
        let tty = self.tty(for: hash) ?? link(sessionId, hash, ttyDescriptor)
        tty.preexec()
      
        // update keybuffer backing
        if (KeypressProvider.shared.keyBuffer(for: hash).backedByZLE) {
          
            // ZLE doesn't handle signals sent to shell, like control+c
            // So we need to manually force an update when the line changes
            DispatchQueue.main.async {
               Autocomplete.update(with: ("", 0), for: hash)
               Autocomplete.position()
            }
            KeypressProvider.shared.keyBuffer(for: hash).backedByZLE = false
        }
    }
    
    func startedNewSSHConnection(_ info: ShellMessage) {
        guard let hash = attemptToFindToAssociatedWindow(for: info.session) else {
              Logger.log(message: "Could not link to window on new shell session.", priority: .notify, subsystem: .tty)
              return
          }
        
        guard let tty = self.tty(for: hash) else { return }
        guard let sshIntegration = tty.integrations["ssh"] as? SSHIntegration else { return }
        sshIntegration.newConnection(with: info, in: tty)
      
        KeypressProvider.shared.keyBuffer(for: hash).backedByZLE = false

    }
  
    func updateKeybuffer(_ info: ShellMessage) {
        guard let hash = attemptToFindToAssociatedWindow(for: info.session) else {
              Logger.log(message: "Could not link to window on new shell session.", priority: .notify, subsystem: .tty)
              return
          }
        
      let keybuffer = KeypressProvider.shared.keyBuffer(for: hash)
      if let (buffer, cursor, histno) = info.parseKeybuffer() {
          let previousHistoryNumber = keybuffer.zleHistoryNumber

          keybuffer.backedByZLE = true
          keybuffer.buffer = buffer
          keybuffer.zleCursor = cursor
          keybuffer.zleHistoryNumber = histno
        
          // Prevent Fig from immediately when the user navigates through history
          // Note that Fig is hidden in response to the "history-line-set" zle hook
        
          // If buffer is empty, line is being reset (eg. ctrl+c) and event should be processed :/
          guard buffer == "" || previousHistoryNumber == histno else {
            print("ZLE: history numbers do not match")
            return
          }
          
          // write only prevents autocomplete from recieving keypresses
          guard !keybuffer.writeOnly else {
            print("ZLE: keybuffer is write only")
            return
          }
          
          print("ZLE: \(buffer) \(cursor) \(histno)")
        
        guard Defaults.loggedIn, Defaults.useAutocomplete else {
          return
        }
        
        DispatchQueue.main.async {
           Autocomplete.update(with: (buffer, cursor), for: hash)
           Autocomplete.position()
//          keybuffer.
        }

    }
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
            //queue.async(flags:[.barrier]) {
                self.sessions[hash] = nil
                self.tty.removeValue(forKey: hash)
            //}


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
        //queue.async(flags: [.barrier]) {
         semaphore.wait()
            self.tty[hash] = device
            self.sessions[hash] = sessionId // sessions is bidirectional between sessionId and windowHash
         semaphore.signal()
        //}
        return device
    }
    
    func getSessionId(for windowHash: ExternalWindowHash) -> SessionId? {
        var id: SessionId?
        //queue.sync {
            id = self.sessions[windowHash]
        //}
      
        return id
    }
    
    fileprivate func getWindowHash(for sessionId: SessionId) -> ExternalWindowHash? {
        var hash: ExternalWindowHash?
        //queue.sync {
            hash = self.sessions[sessionId]
        //}
      
        return hash
    }
    
    func validWindowHash(_ hash: ExternalWindowHash) -> Bool {
        guard let components = hash.components() else { return false }
        let windowHasNoTabs = (tabs[components.windowId] == nil && components.tab == nil)
        let windowHasTabs = (tabs[components.windowId] != nil && components.tab != nil)
        return windowHasNoTabs || windowHasTabs
    }
}
