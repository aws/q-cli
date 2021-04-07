//
//  Notifier.swift
//  fig
//
//  Created by Matt Schrage on 1/21/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class AutocompleteContextNotifier {
  
  static func listenForUpdates() {
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(contextLost),
                                           name: KeystrokeBuffer.contextLostInKeystrokeBufferNotification,
                                           object: nil)
    
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(lineReset),
                                           name: KeystrokeBuffer.lineResetInKeyStrokeBufferNotification,
                                           object: nil)
    
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(lineReset),
                                           name: KeystrokeBuffer.contextRestoredInKeystrokeBufferNotification,
                                           object: nil)
    
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(processUpdated),
                                           name: TTY.processUpdated,
                                           object: nil)
    
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(windowTitleUpdated(_ :)),
                                           name: AXWindowServer.windowTitleUpdatedNotification,
                                           object: nil)
    
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(windowDidChange),
                                           name: AXWindowServer.windowDidChangeNotification,
                                           object: nil)
    
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(permissionDidUpdate(_ :)),
                                           name: Accessibility.permissionDidUpdate,
                                           object: nil)
    
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(applicationWillTerminate),
                                           name: NSApplication.willTerminateNotification,
                                           object: nil)
    
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(autocompletePreferenceUpdated(_ :)),
                                           name: Defaults.autocompletePreferenceUpdated,
                                           object: nil)

  }
  
  enum ContextIndicator: String {
    case noContext = "☒"//"🟠"
    case hasContext = "☑"//"🟢"
    case processIsNotShell = "☐"//"🔵"
    
    func message(for bundleId: String? = nil) -> String {
      var message = "\(self.rawValue) fig"
      if (bundleId != "com.apple.Terminal") {
        message += " — "
      }
      return message
    }
  }
  
  @objc static func windowDidChange() {
    if SecureKeyboardInput.enabled {
      setContextIndicator(.noContext)
    }
  }
  
  @objc static func autocompletePreferenceUpdated(_ notification: Notification) {
    guard let enabled = notification.object as? Bool else { return }
    
    guard addIndicatorToTitlebar else { return }
    
    
    batchUpdate { (window, tty) -> AutocompleteContextNotifier.ContextIndicator in
      return enabled ? indicator(for: window, tty) : .noContext
    }
    
  }
  
  @objc static func applicationWillTerminate() {
    guard addIndicatorToTitlebar else { return }
    
    clearFigContext()
  }
  
  @objc static func permissionDidUpdate(_ notification: Notification) {
    guard let granted = notification.object as? Bool else { return }
    
    guard addIndicatorToTitlebar else { return }
    
    // set all to noContext if accesibility is disabled
    batchUpdate { (window, tty) -> AutocompleteContextNotifier.ContextIndicator in
      return granted ? indicator(for: window, tty) : .noContext
    }
    
  }
  
  static func indicator(for window: ExternalWindowHash, _ tty: TTY ) -> ContextIndicator {
    let keybuffer = KeypressProvider.shared.keyBuffer(for: window)
    
    if (!(tty.isShell ?? false)) {
      return .processIsNotShell
    } else if (keybuffer.buffer != nil && !keybuffer.writeOnly) {
      return .hasContext
    } else {
      return .noContext
    }
  }
  
  @objc static func contextLost() {
    setContextIndicator(.noContext)
  }
  
   @objc static func lineReset() {
      setContextIndicator(.hasContext)
  }
  
  @objc static func processUpdated() {
    setContextIndicator(.hasContext)
  }
  
  @objc static func windowTitleUpdated(_ notification: Notification) {
    guard let window = notification.object as? ExternalWindow else { return }
    
    print("window:", window.windowTitle ?? "")
    let title = window.windowTitle
    let firstCharacter = title?.first ?? " "
    
    
    // ignore if already contains context-indicator
    guard ContextIndicator(rawValue: String(firstCharacter)) != nil else { return }
    
    let keybuffer = KeypressProvider.shared.keyBuffer(for: window)
    
    if (keybuffer.buffer != nil && !keybuffer.writeOnly) {
      setContextIndicator(.hasContext)
    } else {
      setContextIndicator(.noContext)
    }
    
  }
  
  fileprivate static func windowTitleStrippedOfFigContext(using window: ExternalWindow) -> String {
    let title = window.windowTitle
    var update: String = ""
    if (window.bundleId == "com.apple.Terminal") {
      let settableTitle = title?.components(separatedBy: " — ").filter { ContextIndicator(rawValue: String($0.first ?? " ")) != nil } ?? []
      
      if settableTitle.count == 1 {
        update = settableTitle.first!
      }
      
    } else {
      update = title ?? ""
    }
    
    let firstCharacter = update.first
    
    if let previousIndicator = ContextIndicator(rawValue: String(firstCharacter ?? " ")) {
      update = String(update.dropFirst(previousIndicator.message(for: window.bundleId ?? "").count))
    }
    return update
  }
  
  static func setContextIndicator(_ indicatorUpdate: ContextIndicator, overwriteExistingTitle: Bool = true) {
    guard Defaults.loggedIn, Defaults.useAutocomplete, addIndicatorToTitlebar else { return }
    guard let window = AXWindowServer.shared.whitelistedWindow else {
        return
      }

    var indicator = indicatorUpdate
    if (!(window.tty?.isShell ?? false)) {
      indicator = .processIsNotShell
    }
    
    if (SecureKeyboardInput.enabled) {
      indicator = .noContext
    }
    
    let update = windowTitleStrippedOfFigContext(using: window)

    
    let message = indicator.message(for: window.bundleId ?? "")

    if (window.bundleId == "com.apple.Terminal") {
         window.tty?.setTitle(message + update)
    } else {
      window.tty?.setTitle(message + (overwriteExistingTitle ? (window.tty?.name ?? "") : update))
    }
  }
  
  static func batchUpdate(update: (ExternalWindowHash, TTY) -> ContextIndicator) {
    ShellHookManager.shared.ttys().forEach { (pair) in
      let (hash, tty) = pair
      let indicator = update(hash, tty)
      tty.setTitle(indicator.message() + (tty.name ?? ""))
    }
  }
  
  static func setFigContext() {
    batchUpdate { (hash, tty) -> ContextIndicator in
      return indicator(for: hash, tty)
    }
  }
  
  static func clearFigContext() {
    guard AutocompleteContextNotifier.addIndicatorToTitlebar else { return }
    ShellHookManager.shared.ttys().forEach { (pair) in
      let (_, tty) = pair
      tty.setTitle(tty.name ?? "")
    }
  }
  static var addIndicatorToTitlebar: Bool {
    get {
      if let status = Settings.shared.getValue(forKey: Settings.addStatusToTerminalTitle) as? Bool {
        return status
      }
      
      return UserDefaults.standard.bool(forKey: "addIndicatorToTitlebar")
    }

    set(flag) {
      if (!flag) {
        clearFigContext()
      } else {
        setFigContext()
      }
      
      UserDefaults.standard.set(flag, forKey: "addIndicatorToTitlebar")
      UserDefaults.standard.synchronize()
      
      Settings.shared.set(value: flag, forKey: Settings.addStatusToTerminalTitle)
      
    }
  }
  
}
