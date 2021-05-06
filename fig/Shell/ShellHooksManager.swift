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
    func components() -> (windowId: CGWindowID, tab: String?, pane: String?)? {
      
        let tokens = self.components(separatedBy: CharacterSet(charactersIn: "/%"))
        guard let windowId = CGWindowID(tokens[0]) else { return nil }
        let tabString = tokens[safe: 1]
        var tab: String? = nil
        if (tabString != nil && tabString!.count > 0) {
            tab = String(tabString!)
        }
      
        let paneString = tokens[safe: 2]
        var pane: String? = nil
        if (paneString != nil && paneString!.count > 0) {
            pane = String(paneString!)
        }
        
        return (windowId: windowId, tab: tab, pane: pane)
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
    fileprivate var panes: [ExternalWindowHash: String] = [:]
    fileprivate var tabs: [CGWindowID: String] = [:]
    fileprivate var tty: [ExternalWindowHash: TTY] = [:]
    fileprivate var sessions = BiMap<String>()
  
    private let queue = DispatchQueue(label: "com.withfig.shellhooks", attributes: .concurrent)
  
    fileprivate var observer: WindowObserver?
    fileprivate let semaphore = DispatchSemaphore(value: 1)

}

// handle concurrency
extension ShellHookManager {
  func hashFor(_ windowId: CGWindowID) -> ExternalWindowHash {
    let tab = self.tabs[windowId]
    let pane = self.panes["\(windowId)/\(tab ?? "")"]
    return "\(windowId)/\(tab ?? "")\(pane ?? "%")"
  }
  
  func pane(for windowHash: ExternalWindowHash) -> String? {
      return self.panes[windowHash]
  }

  func tab(for windowID: CGWindowID) -> String? {
    return self.tabs[windowID]
  }
  
  func setActivePane(_ pane: String, for windowID: CGWindowID) {
    let tab = self.tab(for: windowID)
    let key = "\(windowID)/\(tab ?? "")"
    if pane == "%" {
      self.panes.removeValue(forKey: key)
    } else {
      self.panes[key] = pane
    }
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
    
    func currentTabDidChange(_ info: ShellMessage, includesBundleId: Bool = false) {
        Logger.log(message: "currentTabDidChange")
        
        // Need time for whitelisted window to change
        Timer.delayWithSeconds(0.1) {
            if let window = AXWindowServer.shared.whitelistedWindow {
                if let id = info.options?.last {
                  
                  if includesBundleId {
                    let tokens = id.split(separator: ":")
                    let bundleId = String(tokens.first!)
                    
                    guard bundleId == window.bundleId ?? "" else {
                      print("tab: bundleId from message did not match bundle id associated with current window ")
                      return
                    }
                  }

                  
                  
                  let VSCodeTerminal = (window.bundleId == Integrations.VSCode || window.bundleId == Integrations.VSCodeInsiders) && id.hasPrefix("code:")
                  let HyperTab = window.bundleId == Integrations.Hyper &&  id.hasPrefix("hyper:")
                  let iTermTab = window.bundleId == Integrations.iTerm && !id.hasPrefix("code:") && !id.hasPrefix("hyper:") && !includesBundleId
                  guard VSCodeTerminal || iTermTab || HyperTab || includesBundleId else { return }
                    Logger.log(message: "tab: \(window.windowId)/\(id)")
//                    self.tabs[window.windowId] = id
                    self.setActiveTab(id, for: window.windowId)
                  
                    // Manually ensuring that values set prior to tab are updated
                    // Make sure oldHash is equal to whatever the default value of the hash would be
                    if (!VSCodeTerminal) {
                      self.updateHashMetadata(oldHash: "\(window.windowId)/%", hash: window.hash)
                    }
                  
                    // refresh cache
                    if Integrations.electronTerminals.contains(window.bundleId ?? "") {
                        let _ = Accessibility.findXTermCursorInElectronWindow(window, skipCache: true)
                    }
                    
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
      
        // Set version (used for checking compatibility)
        tty.shellIntegrationVersion = info.shellIntegrationVersion
        
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
      
        // Set version (used for checking compatibility)
        tty.shellIntegrationVersion = info.shellIntegrationVersion
            
        KeypressProvider.shared.keyBuffer(for: hash).backedByShell = false

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
        
        var delay:TimeInterval!
        
        switch bundleId {
          case Integrations.Hyper:
            delay = Settings.shared.getValue(forKey: Settings.hyperDelayKey) as? TimeInterval ?? 2
          case Integrations.VSCode:
            delay = Settings.shared.getValue(forKey: Settings.vscodeDelayKey) as? TimeInterval ?? 1
          default:
            delay = 0.2
        }
      
        // no delay is needed because the command is being run by the user, so the window is already active
        if info.viaFigCommand {
            delay = 0
        }
      
        observer = WindowObserver(with: bundleId)
      
        let bundleIdBasedOnTermProgram = info.potentialBundleId
      
        // We need to wait for window to appear if the terminal emulator is being launched for the first time. Can this be handled more robustly?
        observer?.windowDidAppear(timeoutAfter: delay, completion: {
            // ensuring window bundleId & frontmostApp bundleId match fixes case where a slow launching application (eg. Hyper) will init shell before window is visible/tracked
            Logger.log(message: "Awaited window did appear", priority: .notify, subsystem: .tty)

            guard let window = AXWindowServer.shared.whitelistedWindow, window.bundleId == NSWorkspace.shared.frontmostApplication?.bundleIdentifier
                else {
                Logger.log(message: "Cannot track a new terminal session if topmost window isn't whitelisted.", priority: .notify, subsystem: .tty)
                return
            }
          
            guard window.bundleId == bundleIdBasedOnTermProgram else {
              Logger.log(message: "Cannot track a new terminal session if topmost window '\(window.bundleId ?? "?")' doesn't correspond to $TERM_PROGRAM '\(bundleIdBasedOnTermProgram ?? "?")'", priority: .notify, subsystem: .tty)
                return
            }
          
            Logger.log(message: "Linking \(ttyDescriptor) with \(window.hash) for \(sessionId)", priority: .notify, subsystem: .tty)

            let tty = self.link(sessionId, window.hash, ttyDescriptor)
            tty.startedNewShellSession(for: shellPid)
          
            // Set version (used for checking compatibility)
            tty.shellIntegrationVersion = info.shellIntegrationVersion
          
          
            DispatchQueue.main.async {
              NotificationCenter.default.post(Notification(name: PseudoTerminal.recievedEnvironmentVariablesFromShellNotification,
                                                           object: info.env?.jsonStringToDict() ?? [:]))
            }

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
      
        // Set version (used for checking compatibility)
        tty.shellIntegrationVersion = info.shellIntegrationVersion
      
        // update keybuffer backing
        if (KeypressProvider.shared.keyBuffer(for: hash).backedByShell) {
          
            // ZLE doesn't handle signals sent to shell, like control+c
            // So we need to manually force an update when the line changes
            DispatchQueue.main.async {
               Autocomplete.update(with: ("", 0), for: hash)
               Autocomplete.position()
            }
            KeypressProvider.shared.keyBuffer(for: hash).backedByShell = false
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
      
        // Set version (used for checking compatibility)
        tty.shellIntegrationVersion = info.shellIntegrationVersion
      
        KeypressProvider.shared.keyBuffer(for: hash).backedByShell = false

    }
    
    func clearKeybuffer(_ info: ShellMessage) {
        guard let hash = attemptToFindToAssociatedWindow(for: info.session) else {
             Logger.log(message: "Could not link to window on new shell session.", priority: .notify, subsystem: .tty)
             return
        }
      
        let keybuffer = KeypressProvider.shared.keyBuffer(for: hash)
        keybuffer.buffer = ""
    }
  
    func updateKeybuffer(_ info: ShellMessage, backing: KeystrokeBuffer.Backing) {
        guard let hash = attemptToFindToAssociatedWindow(for: info.session) else {
              Logger.log(message: "Could not link to window on new shell session.", priority: .notify, subsystem: .tty)
              return
          }
      
      // prevents fig window from popping up if we don't have an associated shell process
      guard let tty = tty[hash], tty.isShell ?? false else {
        return
      }
      
      // Set version (used for checking compatibility)
      tty.shellIntegrationVersion = info.shellIntegrationVersion
      
      // ignore events if secure keyboard is enabled
      guard !SecureKeyboardInput.enabled else {
        return
      }
        
      let keybuffer = KeypressProvider.shared.keyBuffer(for: hash)
      if let (buffer, cursor, histno) = info.parseKeybuffer() {
          let previousHistoryNumber = keybuffer.shellHistoryNumber

          keybuffer.backedByShell = true
          keybuffer.backing = backing
          keybuffer.buffer = buffer
          keybuffer.shellCursor = cursor
          keybuffer.shellHistoryNumber = histno
        
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
  
  func tmuxPaneChanged(_ info: ShellMessage) {
    guard let window = AXWindowServer.shared.whitelistedWindow else { return }
    let oldHash =  window.hash
    
    if let newPane = info.arguments[safe: 0], let (windowId, _, oldPane) = oldHash.components() {
        
      if oldPane != nil {
        // user is switching between panes in tmux
        if newPane == "%" {
          print("tmux: closing tmux session")
        } else {
          print("tmux: user is switching between panes %\(oldPane!) -> \(newPane)")
        }

      } else {
        print("tmux: launched new session")
      }
      
      setActivePane(newPane, for: windowId)

      // trigger updates elsewhere (this is cribbed from the tabs logic)
      DispatchQueue.main.async {
        WindowManager.shared.windowChanged()
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
            self.sessions[sessionId] = nil // unlink sessionId from any previous windowHash
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
        let windowHasNoPanes = (panes["\(components.windowId)/\(components.tab ?? "")"] == nil && components.pane == nil)
        let windowHasPanes = (panes["\(components.windowId)/\(components.tab ?? "")"] != nil && components.pane != nil)
        return (windowHasNoTabs && (windowHasNoPanes || windowHasPanes))
            || (windowHasTabs   && (windowHasNoPanes || windowHasPanes))
    }
}
