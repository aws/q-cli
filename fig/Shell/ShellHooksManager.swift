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
  // swiftlint:disable large_tuple
  func components() -> (windowId: CGWindowID, tab: String?, pane: String?)? {

    let tokens = self.components(separatedBy: CharacterSet(charactersIn: "/%"))
    guard let windowId = CGWindowID(tokens[0]) else { return nil }
    let tabString = tokens[safe: 1]
    var tab: String?
    if tabString != nil && tabString!.count > 0 {
      tab = String(tabString!)
    }

    let paneString = tokens[safe: 2]
    var pane: String?
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
      _ = Accessibility.findXTermCursorInElectronWindow(window, skipCache: true)
      print("xterm-cursor: updating due to tab changed?")

    }

    if Integrations.providers.keys.contains(window.bundleId ?? ""),
       let provider = Integrations.providers[window.bundleId ?? ""] {
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

    // Need time for allowlisted window to change
    Timer.delayWithSeconds(0.1) {
      if let window = AXWindowServer.shared.allowlistedWindow {
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
          let hyperTab = window.bundleId == Integrations.Hyper && id.hasPrefix("hyper:")
          let iTermTab =
            window.bundleId == Integrations.iTerm && !id.hasPrefix("code:")
            && !id.hasPrefix("hyper:") && !includesBundleId
          guard VSCodeTerminal || iTermTab || hyperTab || includesBundleId else { return }
          Logger.log(message: "tab: \(window.windowId)/\(id)")
          self.keyboardFocusDidChange(to: id, in: window)
        }
      }
    }
  }

  // If this changes, make sure to reflect changes in iTermIntegration.sessionId setter
  func currentTabDidChange(applicationIdentifier: String, sessionId: String) {
    Logger.log(message: "currentTabDidChange")

    // Need time for allowlisted window to change
    Timer.delayWithSeconds(0.1) {
      if let window = AXWindowServer.shared.allowlistedWindow {
        // todo(mschrage): remove codepath for handling legacy VSCode extension
        // that does not distinguish which type of VSCode instance is running
        let codeTerminal =
          [Integrations.VSCode, Integrations.VSCodeInsiders, Integrations.VSCodium]
          .contains(window.bundleId)
          && applicationIdentifier == "code"

        guard let provider = Integrations.providers[window.bundleId ?? ""],
                  provider.id == applicationIdentifier || codeTerminal else {
          Logger.log(message: "Terminal Integration Provider not found for '\(applicationIdentifier)'." +
                              "(\(window.bundleId ?? "<unknown bundle>")," +
                              "\(Integrations.providers[window.bundleId ?? ""]?.id ?? "<unknown id>"))")

          return
        }

        Logger.log(message: "tab: \(window.windowId)/\(applicationIdentifier):\(sessionId)")

        self.keyboardFocusDidChange(to: "\(applicationIdentifier):\(sessionId)", in: window)
      }
    }
  }

  func updateHashMetadata(oldHash: ExternalWindowHash, hash: ExternalWindowHash) {

    // queue.async(flags: [.barrier]) {
    guard oldHash != hash else { return }
    guard let device = self.tty[oldHash] else { return }
    guard let sessionId = self.sessions[oldHash] else { return }

    // remove out-of-date values
    self.tty.removeValue(forKey: oldHash)
    self.sessions[oldHash] = nil

    // reassign tty to new hash
    self.sessions[hash] = sessionId
    self.tty[hash] = device
    // }
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
        message: "Could not parse out shellHook metadata", priority: .warn, subsystem: .tty)
      return
    }

    shellPromptWillReturn(
      context: Local_ShellContext.with({ ctx in
        ctx.ttys = ttyDescriptor
        ctx.pid = shellPid
        ctx.sessionID = sessionId
        ctx.processName = info.shell ?? ""
        ctx.integrationVersion = Int32(info.shellIntegrationVersion ?? 0)
      }))
  }

  func shellPromptWillReturn(context: Local_ShellContext) {
    // try to find associated window, but don't necessarily link with the topmost window! (prompt can return when window
    // is in background)
    guard
      let hash = attemptToFindToAssociatedWindow(
        for: context.sessionID,
        currentTopmostWindow: AXWindowServer.shared.allowlistedWindow)
    else {
      Logger.log(
        message: "Could not link to window on shell prompt return.", priority: .warn,
        subsystem: .tty)
      return
    }

    // window hash is valid, we should have an associated TTY (or we can create it)
    let tty = self.tty(for: hash) ?? link(context.sessionID, hash, context.ttys)

    // Window is linked with TTY session
    // update tty's active process to current shell
    tty.returnedToShellPrompt(for: context.pid)

    // Set version (used for checking compatibility)
    tty.shellIntegrationVersion = Int(context.integrationVersion)

    // post notification to API
    API.notifications.post(
      Fig_ShellPromptReturnedNotification.with({ notification in
        notification.sessionID = context.sessionID
      }))
  }

  func startedNewShellSession(_ info: ShellMessage) {
    guard let (shellPid, ttyDescriptor, sessionId) = info.parseShellHook() else {
      Logger.log(
        message: "Could not parse out shellHook metadata", priority: .warn, subsystem: .tty)
      return
    }

    guard let hash = attemptToFindToAssociatedWindow(for: sessionId) else {
      Logger.log(
        message: "Could not link to window on new shell session.", priority: .warn,
        subsystem: .tty)
      return
    }

    // window hash is valid, we should have an associated TTY (or we can create it)
    let tty = self.tty(for: hash) ?? link(sessionId, hash, ttyDescriptor)
    tty.startedNewShellSession(for: shellPid)

    // Set version (used for checking compatibility)
    tty.shellIntegrationVersion = info.shellIntegrationVersion ?? 0
  }

  func startedNewTerminalSessionLegacy(_ info: ShellMessage) {
    guard let (shellPid, ttyDescriptor, sessionId) = info.parseShellHook() else {
      Logger.log(
        message: "Could not parse out shellHook metadata", priority: .warn, subsystem: .tty)
      return
    }

    let ctx = Local_ShellContext.with { context in
      context.sessionID = sessionId
      context.ttys = ttyDescriptor
      context.pid = shellPid
    }
    let calledDirect = info.viaFigCommand
    let bundle = info.potentialBundleId
    let env = info.env?.parseAsJSON() ?? [:]
    let envMap = env.mapValues { val in
      val as? String
    }
    // swiftlint:disable force_cast
    let envFilter =
      envMap.filter { (_, val) in
        val != nil
      } as! [String: String]

    startedNewTerminalSession(
      context: ctx, calledDirect: calledDirect, bundle: bundle, env: envFilter)
  }

  func startedNewTerminalSession(context: Local_ShellContext,
                                 calledDirect: Bool,
                                 bundle: String?,
                                 env: [String: String]) {

    DispatchQueue.main.async {
      NotificationCenter.default.post(
        Notification(
          name: PseudoTerminal.recievedEnvVarsFromShellNotification,
          object: env))
    }

  }

  func shellWillExecuteCommandLegacy(_ info: ShellMessage) {
    guard let (_, ttyDescriptor, sessionId) = info.parseShellHook() else {
      Logger.log(
        message: "Could not parse out shellHook metadata", priority: .warn, subsystem: .tty)
      return
    }

    let context = Local_ShellContext.with { ctx in
      ctx.integrationVersion = Int32(info.shellIntegrationVersion ?? 0)
      ctx.ttys = ttyDescriptor
      ctx.sessionID = sessionId
    }

    shellWillExecuteCommand(context: context)
  }

  func shellWillExecuteCommand(context: Local_ShellContext) {
    guard
      let hash = attemptToFindToAssociatedWindow(
        for: context.sessionID,
        currentTopmostWindow: AXWindowServer.shared.allowlistedWindow)
    else {

      Logger.log(
        message: "Could not link to window on new terminal session.", priority: .warn,
        subsystem: .tty)
      return
    }

    let tty = self.tty(for: hash) ?? link(context.sessionID, hash, context.ttys)
    tty.preexec()

    // Set version (used for checking compatibility)
    tty.shellIntegrationVersion = Int(context.integrationVersion)
    API.notifications.editbufferChanged(buffer: "",
                                        cursor: 0,
                                        session: context.sessionID,
                                        context: context)

    DispatchQueue.main.async {
      Autocomplete.position()
    }
  }

  func startedNewSSHConnectionLegacy(_ info: ShellMessage) {
    startedNewSSHConnection(info)
  }

  func startedNewSSHConnection(_ info: ShellMessage) {
    Logger.log(message: "starting new SSH connection...")
  }

  func updateKeybufferLegacy(_ info: ShellMessage) {
    if let (buffer, cursor, histno) = info.parseKeybuffer() {
      updateKeybuffer(
        context: Local_ShellContext.with({ ctx in
          ctx.sessionID = info.session
          ctx.processName = info.shell ?? ""
          ctx.integrationVersion = Int32(info.shellIntegrationVersion ?? 0)
          ctx.ttys = info.environmentVariable(for: "TTY") ?? ""
          let pidString = info.environmentVariable(for: "PID")
          ctx.pid = Int32(pidString ?? "") ?? -1
        }), text: buffer, cursor: cursor, histno: histno)
    }
  }

  func updateKeybuffer(context: Local_ShellContext, text: String, cursor: Int, histno: Int) {
    // invariant: frontmost allowlisted window is assumed to host shell session which sent this edit buffer event.
    guard let window = AXWindowServer.shared.allowlistedWindow else {
      Logger.log(
        message: "Could not link to window on new shell session.", priority: .warn,
        subsystem: .tty)
      return
    }

    let hash = window.hash
    var ttyHandler: TTY? = tty[hash]

    if ttyHandler == nil, let trimmedDescriptor = context.ttys.split(separator: "/").last {
      Logger.log(message: "linking sessionId (\(context.sessionID)) to window hash: \(hash)", subsystem: .tty)
      ttyHandler = self.link(context.sessionID, hash, String(trimmedDescriptor))
      ttyHandler?.startedNewShellSession(for: context.pid)
    }

    guard let tty = ttyHandler else {
      return
    }

    // Set version (used for checking compatibility)
    tty.shellIntegrationVersion = Int(context.integrationVersion)

    // ignore events if secure keyboard is enabled
    guard !SecureKeyboardInput.enabled else {
      return
    }

    // What need to be true for us to send notification!
    guard let session = TerminalSessionLinker.shared.getTerminalSession(for: context.sessionID),
          let editBuffer = session.editBuffer,
          Defaults.shared.loggedIn,
          Defaults.shared.useAutocomplete else {
      return
    }

    API.notifications.editbufferChanged(buffer: editBuffer.text,
                                        cursor: editBuffer.cursor,
                                        session: session.terminalSessionId,
                                        context: session.shellContext?.ipcContext)

    DispatchQueue.main.async {
      Autocomplete.position()
    }

  }

  func tmuxPaneChangedLegacy(_ info: ShellMessage) {
    guard let window = AXWindowServer.shared.allowlistedWindow else { return }
    let oldHash = window.hash

    if let newPane = info.arguments[safe: 0],
       let (windowId, sessionHash, oldPane) = oldHash.components() {

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

  func eventHook(event: String?) {
    if let event = event {
      TelemetryProvider.shared.track(event: event, with: [:])
    } else {
      print("No event")
    }
  }

}

extension ShellHookManager {

  fileprivate func attemptToFindToAssociatedWindow(
    for sessionId: SessionId, currentTopmostWindow: ExternalWindow? = nil
  ) -> ExternalWindowHash? {

    guard let session = TerminalSessionLinker.shared.getTerminalSession(for: sessionId) else {
      Logger.log(message: "Could not find hash for sessionId '\(sessionId)'", subsystem: .tty)

      return nil
    }

    let hash = session.generateLegacyWindowHash()

    Logger.log(message: "Found WindowHash '\(hash)' for sessionId '\(sessionId)'", subsystem: .tty)

    return hash
  }

  fileprivate func link(
    _ sessionId: SessionId, _ hash: ExternalWindowHash, _ ttyDescriptor: TTYDescriptor
  ) -> TTY {
    let device = TTY(fd: ttyDescriptor)

    // tie tty & sessionId to windowHash
    // queue.async(flags: [.barrier]) {
    semaphore.wait()
    self.tty[hash] = device
    self.sessions[sessionId] = nil  // unlink sessionId from any previous windowHash
    self.sessions[hash] = sessionId  // sessions is bidirectional between sessionId and windowHash
    semaphore.signal()
    // }
    return device
  }

  func getSessionId(for windowHash: ExternalWindowHash) -> SessionId? {
    return self.sessions[windowHash]
  }

  fileprivate func getWindowHash(for sessionId: SessionId) -> ExternalWindowHash? {
    var hash: ExternalWindowHash?
    // queue.sync {
    hash = self.sessions[sessionId]
    // }

    return hash
  }

}
