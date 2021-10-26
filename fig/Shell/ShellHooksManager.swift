//
//  ShellHooksManager.swift
//  fig
//
//  Created by Matt Schrage on 8/28/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
import FigAPIBindings
import Foundation

extension ExternalWindowHash {
  func components() -> (windowId: CGWindowID, tab: String?, pane: String?)? {

    let tokens = self.components(separatedBy: CharacterSet(charactersIn: "/%"))
    guard let windowId = CGWindowID(tokens[0]) else { return nil }
    let tabString = tokens[safe: 1]
    var tab: String? = nil
    if tabString != nil && tabString!.count > 0 {
      tab = String(tabString!)
    }

    let paneString = tokens[safe: 2]
    var pane: String? = nil
    if paneString != nil && paneString!.count > 0 {
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

class ShellHookManager: NSObject {
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

  func setTTY(_ tty: TTY, for window: ExternalWindowHash) {
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

  func keyboardFocusDidChange(to uuid: String, in window: ExternalWindow) {
    let isHyper = window.bundleId == Integrations.Hyper

    self.setActiveTab(uuid, for: window.windowId)

    // Manually ensuring that values set prior to tab are updated
    // Make sure oldHash is equal to whatever the default value of the hash would be
    // Why Hyper? Any terminal integration that reports the sessionId without waiting for
    // the session to change, can be included here!
    //
    // Launched App                      Changed Tabs
    // 123/%    123/abc%                   123/def%
    // |-------->---------------------------->--------------------
    //          SessionId for current Tab    SessionId for new tab
    if isHyper {
      self.updateHashMetadata(oldHash: "\(window.windowId)/%", hash: window.hash)
    }

    // refresh cache! Why don't we us Accessibility.resetCache()?
    if Integrations.electronTerminals.contains(window.bundleId ?? "") {
      let _ = Accessibility.findXTermCursorInElectronWindow(window, skipCache: true)
      print("xterm-cursor: updating due to tab changed?")

    }

    if Integrations.providers.keys.contains(window.bundleId ?? ""),
      let provider = Integrations.providers[window.bundleId ?? ""]
    {
      provider.runtimeValidationOccured()
    }

    DispatchQueue.main.async {
      // If leaving visor mode in iTerm, we need to manually check which window is on top
      // if let app = NSWorkspace.shared.frontmostApplication {
      //     AXWindowServer.shared.register(app, fromActivation: true)
      // }

      WindowManager.shared.windowChanged()
    }
  }
  
  func currentTabDidChangeLegacy(_ info: ShellMessage, includesBundleId: Bool = false) {
    Logger.log(message: "currentTabDidChange")

    // Need time for whitelisted window to change
    Timer.delayWithSeconds(0.1) {
      if let window = AXWindowServer.shared.whitelistedWindow {
        if let id = info.options?.last {
          
          if includesBundleId {
            let tokens = id.split(separator: ":")
            let bundleId = String(tokens.first!)

            guard bundleId == window.bundleId ?? "" else {
              print(
                "tab: bundleId from message did not match bundle id associated with current window "
              )
              return
            }
          }

          let VSCodeTerminal =
            [Integrations.VSCode, Integrations.VSCodeInsiders, Integrations.VSCodium].contains(
              window.bundleId) && id.hasPrefix("code:")
          let HyperTab = window.bundleId == Integrations.Hyper && id.hasPrefix("hyper:")
          let iTermTab =
            window.bundleId == Integrations.iTerm && !id.hasPrefix("code:")
            && !id.hasPrefix("hyper:") && !includesBundleId
          guard VSCodeTerminal || iTermTab || HyperTab || includesBundleId else { return }
          Logger.log(message: "tab: \(window.windowId)/\(id)")
          self.keyboardFocusDidChange(to: id, in: window)
        }
      }
    }
  }
  
  // If this changes, make sure to reflect changes in iTermIntegration.sessionId setter
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
              print(
                "tab: bundleId from message did not match bundle id associated with current window "
              )
              return
            }
          }

          let VSCodeTerminal =
            [Integrations.VSCode, Integrations.VSCodeInsiders, Integrations.VSCodium].contains(
              window.bundleId) && id.hasPrefix("code:")
          let HyperTab = window.bundleId == Integrations.Hyper && id.hasPrefix("hyper:")
          let iTermTab =
            window.bundleId == Integrations.iTerm && !id.hasPrefix("code:")
            && !id.hasPrefix("hyper:") && !includesBundleId
          guard VSCodeTerminal || iTermTab || HyperTab || includesBundleId else { return }
          Logger.log(message: "tab: \(window.windowId)/\(id)")
          self.keyboardFocusDidChange(to: id, in: window)
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
    Logger.log(
      message: "Transfering \(oldHash) metadata to \(hash).", priority: .info, subsystem: .tty)

  }

  func currentDirectoryDidChange(_ info: ShellMessage) {
    let workingDirectory = info.getWorkingDirectory() ?? ""

    Logger.log(message: "directoryDidChange:\(info.session) -- \(workingDirectory)")

    // We used to pass this to javascript. Now working directory is determined using tty/shellPid

  }

  func shellPromptWillReturnLegacy(_ info: ShellMessage) {
    guard let (shellPid, ttyDescriptor, sessionId) = info.parseShellHook() else {
      Logger.log(
        message: "Could not parse out shellHook metadata", priority: .notify, subsystem: .tty)
      return
    }

    shellPromptWillReturn(
      context: Local_ShellContext.with({ ctx in
        ctx.ttys = ttyDescriptor
        ctx.pid = shellPid
        ctx.sessionID = sessionId
        ctx.shell = info.shell ?? ""
        ctx.integrationVersion = String(info.shellIntegrationVersion ?? 0)
      }))
  }

  func shellPromptWillReturn(context: Local_ShellContext) {
    // try to find associated window, but don't necessarily link with the topmost window! (prompt can return when window is in background)
    guard
      let hash = attemptToFindToAssociatedWindow(
        for: context.sessionID,
        currentTopmostWindow: AXWindowServer.shared.whitelistedWindow)
    else {
      Logger.log(
        message: "Could not link to window on shell prompt return.", priority: .notify,
        subsystem: .tty)
      return
    }

    // window hash is valid, we should have an associated TTY (or we can create it)
    let tty = self.tty(for: hash) ?? link(context.sessionID, hash, context.ttys)

    // Window is linked with TTY session
    // update tty's active process to current shell
    tty.returnedToShellPrompt(for: context.pid)

    // Set version (used for checking compatibility)
    tty.shellIntegrationVersion = context.integrationVersion

    // post notification to API
    API.notifications.post(
      Fig_ShellPromptReturnedNotification.with({ notification in
        notification.sessionID = context.sessionID
      }))

    // if the user has returned to the shell, their keypress buffer must be reset (for instance, if they exited by pressing 'q' rather than return)
    // This doesn't work because of timing issues. If the user types too quickly, the first keypress will be overwritten.
    // KeypressProvider.shared.keyBuffer(for: hash).buffer = ""

    // if Fig should emulate shell autocomplete behavior and only appear when tab is pressed, set keybuffer to writeOnly
    KeypressProvider.shared.keyBuffer(for: hash).writeOnly =
      (Settings.shared.getValue(forKey: Settings.onlyShowOnTabKey) as? Bool) ?? false
  }

  func startedNewShellSession(_ info: ShellMessage) {
    guard let (shellPid, ttyDescriptor, sessionId) = info.parseShellHook() else {
      Logger.log(
        message: "Could not parse out shellHook metadata", priority: .notify, subsystem: .tty)
      return
    }

    guard let hash = attemptToFindToAssociatedWindow(for: sessionId) else {
      Logger.log(
        message: "Could not link to window on new shell session.", priority: .notify,
        subsystem: .tty)
      return
    }

    // window hash is valid, we should have an associated TTY (or we can create it)
    let tty = self.tty(for: hash) ?? link(sessionId, hash, ttyDescriptor)
    tty.startedNewShellSession(for: shellPid)

    // Set version (used for checking compatibility)
    tty.shellIntegrationVersion = String(info.shellIntegrationVersion ?? 0)

    KeypressProvider.shared.keyBuffer(for: hash).backedByShell = false

  }
  
  func startedNewTerminalSessionLegacy(_ info: ShellMessage) {
    guard let (shellPid, ttyDescriptor, sessionId) = info.parseShellHook() else {
      Logger.log(
        message: "Could not parse out shellHook metadata", priority: .notify, subsystem: .tty)
      return
    }
    
    let ctx = Local_ShellContext.with { context in
      context.sessionID = sessionId
      context.ttys = ttyDescriptor
      context.pid = shellPid
    }
    let calledDirect = info.viaFigCommand
    let bundle = info.potentialBundleId
    let env = info.env?.jsonStringToDict() ?? [:]
    let envMap = env.mapValues { val in
      val as? String
    }
    let envFilter = envMap.filter { (_, val) in
      val != nil
    } as! [String: String]
    
    startedNewTerminalSession(context: ctx, calledDirect: calledDirect, bundle: bundle, env: envFilter)
  }

  func startedNewTerminalSession(context: Local_ShellContext, calledDirect: Bool, bundle: String?, env: [String: String]) {

    guard let bundleId = NSWorkspace.shared.frontmostApplication?.bundleIdentifier else {
      Logger.log(message: "Could not get bundle id", priority: .notify, subsystem: .tty)
      return
    }

    var delay: TimeInterval!

    switch bundleId {
    case Integrations.Hyper:
      delay = Settings.shared.getValue(forKey: Settings.hyperDelayKey) as? TimeInterval ?? 2
    case Integrations.VSCode:
      delay = Settings.shared.getValue(forKey: Settings.vscodeDelayKey) as? TimeInterval ?? 1
    default:
      delay = 0.2
    }

    // no delay is needed because the command is being run by the user, so the window is already active
    if calledDirect {
      delay = 0
    }

    observer = WindowObserver(with: bundleId)

    // We need to wait for window to appear if the terminal emulator is being launched for the first time. Can this be handled more robustly?
    observer?.windowDidAppear(
      timeoutAfter: delay,
      completion: {
        // ensuring window bundleId & frontmostApp bundleId match fixes case where a slow launching application (eg. Hyper) will init shell before window is visible/tracked
        Logger.log(message: "Awaited window did appear", priority: .notify, subsystem: .tty)

        guard let window = AXWindowServer.shared.whitelistedWindow,
          window.bundleId == NSWorkspace.shared.frontmostApplication?.bundleIdentifier
        else {
          Logger.log(
            message: "Cannot track a new terminal session if topmost window isn't whitelisted.",
            priority: .notify, subsystem: .tty)
          return
        }

        guard window.bundleId == bundle else {
          Logger.log(
            message:
              "Cannot track a new terminal session if topmost window '\(window.bundleId ?? "?")' doesn't correspond to $TERM_PROGRAM '\(bundle ?? "?")'",
            priority: .notify, subsystem: .tty)
          return
        }

        Logger.log(
          message: "Linking \(context.ttys) with \(window.hash) for \(context.sessionID)",
          priority: .notify, subsystem: .tty)

        let tty = self.link(context.sessionID, window.hash, context.ttys)
        tty.startedNewShellSession(for: context.pid)

        // Set version (used for checking compatibility)
        tty.shellIntegrationVersion = context.integrationVersion

        DispatchQueue.main.async {
          NotificationCenter.default.post(
            Notification(
              name: PseudoTerminal.recievedEnvironmentVariablesFromShellNotification,
              object: env))
        }

      })

  }
  
  func shellWillExecuteCommandLegacy(_ info: ShellMessage) {
    guard let (_, ttyDescriptor, sessionId) = info.parseShellHook() else {
      Logger.log(
        message: "Could not parse out shellHook metadata", priority: .notify, subsystem: .tty)
      return
    }
    
    let context = Local_ShellContext.with { ctx in
      ctx.integrationVersion = String(info.shellIntegrationVersion ?? 0)
      ctx.ttys = ttyDescriptor
      ctx.sessionID = sessionId
    }
    
    shellWillExecuteCommand(context: context)
  }

  func shellWillExecuteCommand(context: Local_ShellContext) {
    guard
      let hash = attemptToFindToAssociatedWindow(
        for: context.sessionID,
        currentTopmostWindow: AXWindowServer.shared.whitelistedWindow)
    else {

      Logger.log(
        message: "Could not link to window on new terminal session.", priority: .notify,
        subsystem: .tty)
      return
    }

    let tty = self.tty(for: hash) ?? link(context.sessionID, hash, context.ttys)
    tty.preexec()

    // Set version (used for checking compatibility)
    tty.shellIntegrationVersion = context.integrationVersion

    // update keybuffer backing
    if KeypressProvider.shared.keyBuffer(for: hash).backedByShell {

      // ZLE doesn't handle signals sent to shell, like control+c
      // So we need to manually force an update when the line changes
      DispatchQueue.main.async {
        Autocomplete.update(with: ("", 0), for: hash)
        Autocomplete.position()
      }
      KeypressProvider.shared.keyBuffer(for: hash).backedByShell = false
    }
  }

  func startedNewSSHConnectionLegacy(_ info: ShellMessage) {
    startedNewSSHConnection(info)
  }
  
  func startedNewSSHConnection(_ info: ShellMessage) {
    guard let hash = attemptToFindToAssociatedWindow(for: info.session) else {
      Logger.log(
        message: "Could not link to window on new shell session.", priority: .notify,
        subsystem: .tty)
      return
    }

    guard let tty = self.tty(for: hash) else { return }
    guard let sshIntegration = tty.integrations["ssh"] as? SSHIntegration else { return }
    sshIntegration.newConnection(with: info, in: tty)

    // Set version (used for checking compatibility)
    tty.shellIntegrationVersion = String(info.shellIntegrationVersion ?? 0)

    KeypressProvider.shared.keyBuffer(for: hash).backedByShell = false

  }
  
  func clearKeybufferLegacy(_ info: ShellMessage) {
    clearKeybuffer(info)
  }

  func clearKeybuffer(_ info: ShellMessage) {
    guard let hash = attemptToFindToAssociatedWindow(for: info.session) else {
      Logger.log(
        message: "Could not link to window on new shell session.", priority: .notify,
        subsystem: .tty)
      return
    }

    let keybuffer = KeypressProvider.shared.keyBuffer(for: hash)
    keybuffer.buffer = ""
  }

  func updateKeybufferLegacy(_ info: ShellMessage) {
    if let (buffer, cursor, histno) = info.parseKeybuffer() {
      updateKeybuffer(
        context: Local_ShellContext.with({ ctx in
          ctx.sessionID = info.session
          ctx.shell = info.shell ?? ""
          ctx.integrationVersion = String(info.shellIntegrationVersion ?? 0)
        }), text: buffer, cursor: cursor, histno: histno)
    }
  }

  func updateKeybuffer(context: Local_ShellContext, text: String, cursor: Int, histno: Int) {
    Logger.log(message: "Keybuffer update")
    guard
      let hash = attemptToFindToAssociatedWindow(
        for: context.sessionID,
        currentTopmostWindow: AXWindowServer.shared.whitelistedWindow)
    else {
      Logger.log(
        message: "Could not link to window on new shell session.", priority: .notify,
        subsystem: .tty)
      return
    }

    var ttyHandler: TTY? = tty[hash]

    // stop process if user is definitely in a shell process
    //      guard ttyHandler?.isShell != true else {
    //        return
    //      }

    if ttyHandler == nil,
      let trimmedDescriptor = context.ttys.split(separator: "/").last
    {
      //        print("tty",  ?? "?")
      //        print("tty", info.env?.jsonStringToDict()?["PID"] ?? "?")
      //        ttyDescriptor.split(sep)
      print("tty: linking")
      ttyHandler = self.link(context.sessionID, hash, String(trimmedDescriptor))

      ttyHandler?.startedNewShellSession(for: context.pid)

    }

    // prevents fig window from popping up if we don't have an associated shell process
    //      guard let tty = tty[hash], tty.isShell ?? false else {
    //        return
    //      }
    guard let tty = ttyHandler else {
      return
    }

    // Set version (used for checking compatibility)
    tty.shellIntegrationVersion = context.integrationVersion

    // ignore events if secure keyboard is enabled
    guard !SecureKeyboardInput.enabled else {
      return
    }

    let keybuffer = KeypressProvider.shared.keyBuffer(for: hash)

    let previousHistoryNumber = keybuffer.shellHistoryNumber

    keybuffer.backedByShell = true
    // Is this okay??? TODO Grant
    //keybuffer.backing = context.shell
    keybuffer.buffer = text
    keybuffer.shellCursor = cursor
    keybuffer.shellHistoryNumber = histno

    // Prevent Fig from immediately when the user navigates through history
    // Note that Fig is hidden in response to the "history-line-set" zle hook

    let isFirstCharacterOfNewLine = previousHistoryNumber != histno && text.count == 1

    // If buffer is empty, line is being reset (eg. ctrl+c) and event should be processed :/
    guard text == "" || previousHistoryNumber == histno || isFirstCharacterOfNewLine else {
      print("ZLE: history numbers do not match")
      return
    }

    // write only prevents autocomplete from recieving keypresses
    // if buffer is empty, make sure autocomplete window is hidden
    // when writeOnly is the default starting state (eg. fig.settings.autocomplete.onlyShowOnTab)
    guard text == "" || !keybuffer.writeOnly else {
      print("ZLE: keybuffer is write only")
      return
    }

    print("ZLE: \(text) \(cursor) \(histno)")

    guard Defaults.loggedIn, Defaults.useAutocomplete else {
      return
    }
    API.notifications.post(
      Fig_EditBufferChangedNotification.with({ notification in
        if let (buffer, cursor) = keybuffer.currentState {
          notification.buffer = buffer
          notification.cursor = Int32(cursor)
        }

        notification.sessionID = context.sessionID
      }))
    DispatchQueue.main.async {
      Autocomplete.update(with: (text, cursor), for: hash)
      Autocomplete.position()

    }
  }

  func tmuxPaneChangedLegacy(_ info: ShellMessage) {
    guard let window = AXWindowServer.shared.whitelistedWindow else { return }
    let oldHash = window.hash

    if let newPane = info.arguments[safe: 0],
      let (windowId, sessionHash, oldPane) = oldHash.components()
    {

      if oldPane != nil {
        // user is switching between panes in tmux
        if newPane == "%" {
          Logger.log(message: "closing tmux session", subsystem: .tmux)

          // Remove all associated panes by filtering out all panes with prefix
          // corresponding to current window hash
          let stalePrefix = "\(windowId)/\(sessionHash ?? "")%"
          let staleWindowHashes = self.tty.keys.filter {
            $0.count > stalePrefix.count && $0.hasPrefix(stalePrefix)
          }

          Logger.log(
            message: "removing \(staleWindowHashes.count) stale window hashes", subsystem: .tmux)

          staleWindowHashes.forEach {
            Logger.log(message: $0, subsystem: .tmux)
            self.tty.removeValue(forKey: $0)
          }
        } else {
          Logger.log(
            message: "user is switching between panes %\(oldPane!) -> \(newPane)", subsystem: .tmux)
        }

      } else {
        Logger.log(message: "launched new session", subsystem: .tmux)
      }

      setActivePane(newPane, for: windowId)

      // trigger updates elsewhere (this is cribbed from the tabs logic)
      DispatchQueue.main.async {
        WindowManager.shared.windowChanged()
      }

    }

  }
  
  func integrationReadyHook(identifier: String) {
    switch identifier {
    case "iterm":
      iTermIntegration.default.attemptToConnect()
    default:
      break
    }
    
  }

}

extension ShellHookManager {

  fileprivate func attemptToFindToAssociatedWindow(
    for sessionId: SessionId, currentTopmostWindow: ExternalWindow? = nil
  ) -> ExternalWindowHash? {

    if let hash = getWindowHash(for: sessionId) {
      guard !validWindowHash(hash) else {
        // the hash is valid and is linked to a session
        Logger.log(message: "WindowHash '\(hash)' is valid", subsystem: .tty)

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
          Logger.log(message: "A different session Id (\(sessionIdForWindow!) is already associated with window (\(hash)), don't link new session (\(sessionId)!", priority: .info, subsystem: .tty)
            return nil
        }
        
        Logger.log(message: "Found WindowHash '\(hash)' for sessionId '\(sessionId)'", subsystem: .tty)
        return hash

      }

      // hash exists, but is invalid (eg. should have tab component and it doesn't)

      Logger.log(
        message: "\(hash) is not a valid window hash, attempting to find previous value",
        priority: .info, subsystem: .tty)

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
      Logger.log(
        message: "No window included when attempting to link to TTY, don't link!", priority: .info,
        subsystem: .tty)
      return nil
    }

    let hash = window.hash
    let sessionIdForWindow = getSessionId(for: hash)

    guard sessionIdForWindow == nil else {
      // a different session Id is already associated with window, don't link!
      Logger.log(
        message: "A different session Id is already associated with window, don't link!",
        priority: .info, subsystem: .tty)
      return nil
    }

    Logger.log(message: "Found WindowHash '\(hash)' for sessionId '\(sessionId)'", subsystem: .tty)
    return hash

  }

  fileprivate func link(
    _ sessionId: SessionId, _ hash: ExternalWindowHash, _ ttyDescriptor: TTYDescriptor
  ) -> TTY {
    let device = TTY(fd: ttyDescriptor)

    // tie tty & sessionId to windowHash
    //queue.async(flags: [.barrier]) {
    semaphore.wait()
    self.tty[hash] = device
    self.sessions[sessionId] = nil  // unlink sessionId from any previous windowHash
    self.sessions[hash] = sessionId  // sessions is bidirectional between sessionId and windowHash
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
    let windowHasNoPanes =
      (panes["\(components.windowId)/\(components.tab ?? "")"] == nil && components.pane == nil)
    let windowHasPanes =
      (panes["\(components.windowId)/\(components.tab ?? "")"] != nil && components.pane != nil)
    return (windowHasNoTabs && (windowHasNoPanes || windowHasPanes))
      || (windowHasTabs && (windowHasNoPanes || windowHasPanes))
  }
}
