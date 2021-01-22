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
    NotificationCenter.default.addObserver(self, selector: #selector(contextLost), name: KeystrokeBuffer.contextLostInKeystrokeBufferNotification, object: nil)
    NotificationCenter.default.addObserver(self, selector: #selector(lineReset), name: KeystrokeBuffer.lineResetInKeyStrokeBufferNotification, object: nil)
    NotificationCenter.default.addObserver(self, selector: #selector(lineReset), name: KeystrokeBuffer.contextRestoredInKeystrokeBufferNotification, object: nil)
    NotificationCenter.default.addObserver(self, selector: #selector(processUpdated), name: TTY.processUpdated, object: nil)

  }
  
  enum ContextIndicator: String {
    case noContext = "☒"//"🟠"
    case hasContext = "☑"//"🟢"
    case processIsNotShell = "☐"//"🔵"
    case none = "none"
    
    func message(for bundleId: String) -> String {
      guard self != .none else { return "" }
      
      var message = "\(self.rawValue) fig"
      if (bundleId == "com.googlecode.iterm2") {
        message += " — "
      }
      return message
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
  
  static func setContextIndicator(_ indicatorUpdate: ContextIndicator) {
    guard Defaults.loggedIn, Defaults.useAutocomplete else { return }
    guard let window = AXWindowServer.shared.whitelistedWindow else {
        return
      }

    var indicator = indicatorUpdate
    if (!(window.tty?.isShell ?? false)) {
      indicator = .processIsNotShell
    }
    
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
    
    let message = indicator.message(for: window.bundleId ?? "")

    if (indicator == .none) {
      window.tty?.setTitle(update)
      
    } else if (window.bundleId == "com.apple.Terminal") {
         window.tty?.setTitle(message)
    } else {
      window.tty?.setTitle(message + (window.tty?.name ?? ""))
    }
  }
  
}
