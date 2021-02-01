//
//  Diagnostic.swift
//  fig
//
//  Created by Matt Schrage on 1/28/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Diagnostic {
  static var accessibility: Bool {
    get {
      return Accessibility.enabled
    }
  }
  
  static var secureKeyboardInput: Bool {
    get {
      return SecureKeyboardInput.enabled
    }
  }
  
  static var blockingProcess: String? {
    get {
      guard SecureKeyboardInput.enabled else { return nil }

      if let app = SecureKeyboardInput.responsibleApplication {
        return "\(app.localizedName ?? "") - \(app.bundleIdentifier ?? "")"
      } else {
        return "no app for pid '\(SecureKeyboardInput.responsibleProcessId ?? -1)'"
      }
    }
  }
  
  static var userConfig: String? {
    get {
      return try? String(contentsOfFile: "\(NSHomeDirectory())/.fig/user/config", encoding: String.Encoding.utf8)
    }
  }
  
  static var installedCLI: Bool {
    get {
      guard let path = Diagnostic.pathOfCLI, let symlink = try? FileManager.default.destinationOfSymbolicLink(atPath: path) else { return false }
      
      return FileManager.default.fileExists(atPath: path) && FileManager.default.fileExists(atPath: symlink)
    }
  }
  
  static var pathOfCLI: String? {
    var location: String? = nil
    
    if (FileManager.default.fileExists(atPath: "\(NSHomeDirectory())/.fig/bin/fig")) {
      location = "\(NSHomeDirectory())/.fig/bin/fig"
    } else if (FileManager.default.fileExists(atPath: "/usr/local/bin/fig")) {
      location = "/usr/local/bin/fig"
    }
    
    return location
  }
    
  static var pathToBundle: String {
      return Bundle.main.bundlePath
  }
  
  static var numberOfCompletionSpecs: Int {
    get {
      return (try? FileManager.default.contentsOfDirectory(atPath: "\(NSHomeDirectory())/.fig/autocomplete").count) ?? 0
    }
  }
  
  static var processForTopmostWindow: String {
    get {
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.nativeTerminals.contains(app.bundleIdentifier ?? ""),
            let window = AXWindowServer.shared.whitelistedWindow,
            let tty = window.tty
      else {
        return "???"
      }

      return tty.cmd != nil ? "\(tty.cmd ?? "")" : "<Unknown Process>"
    }
  }
  
  static var workingDirectoryForTopmostWindow: String {
    get {
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.nativeTerminals.contains(app.bundleIdentifier ?? ""),
            let window = AXWindowServer.shared.whitelistedWindow,
            let tty = window.tty
      else {
        return "???"
      }

      return tty.cwd ?? "<Unknown Working Directory>"
    }
  }
  
  static var processIsShellInTopmostWindow: Bool {
    get {
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.nativeTerminals.contains(app.bundleIdentifier ?? ""),
            let window = AXWindowServer.shared.whitelistedWindow,
            let tty = window.tty
      else {
        return false
      }

      return tty.isShell ?? false
    }
  }
  
  static var ttyDescriptorForTopmostWindow: String {
    get {
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.nativeTerminals.contains(app.bundleIdentifier ?? ""),
            let window = AXWindowServer.shared.whitelistedWindow,
            let tty = window.tty
      else {
        return "???"
      }

      return tty.descriptor
    }
  }
  
  static var descriptionOfTopmostWindow: String {
    get {
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.nativeTerminals.contains(app.bundleIdentifier ?? ""),
            let window = AXWindowServer.shared.whitelistedWindow
      else {
        return "???"
      }

      return "\(window.hash) (\(window.bundleId ?? "???"))"
    }
  }
  
  static var keybufferHasContextForTopmostWindow: Bool {
    get {
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.nativeTerminals.contains(app.bundleIdentifier ?? ""),
            let window = AXWindowServer.shared.whitelistedWindow
      else {
        return false
      }
      
      let keybuffer = KeypressProvider.shared.keyBuffer(for: window)
      return keybuffer.buffer != nil
    }
  }
  
  static var keybufferRepresentationForTopmostWindow: String {
    get {
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.nativeTerminals.contains(app.bundleIdentifier ?? ""),
            let window = AXWindowServer.shared.whitelistedWindow
      else {
        return "<no context>"
      }
      
      let keybuffer = KeypressProvider.shared.keyBuffer(for: window)
      return keybuffer.representation
    }
  }
  
  static var version: String {
    return Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "-1"
  }
  
  static var build: String {
    return Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? ""
  }
  
  static var distribution: String {
    return "Version \(Diagnostic.version) (B\(Diagnostic.build))"
  }
  
  static var summary: String {
    get {
      """
      
      \(Diagnostic.distribution)
      UserShell: \(Defaults.userShell)
      Bundle path: \(Diagnostic.pathToBundle)
      Autocomplete: \(Defaults.useAutocomplete)
      CLI installed: \(Diagnostic.installedCLI)
      CLI tool path: \(Diagnostic.pathOfCLI ?? "<none>")
      Accessibility: \(Accessibility.enabled)
      Number of specs: \(Diagnostic.numberOfCompletionSpecs)
      SSH Integration: \(Defaults.SSHIntegrationEnabled)
      Only insert on tab: \(Defaults.onlyInsertOnTab)
      SecureKeyboardInput: \(Diagnostic.secureKeyboardInput)
      SecureKeyboardProcess: \(Diagnostic.blockingProcess ?? "<none>")
      iTerm Tab Integration: \(iTermTabIntegration.isInstalled())
      Current active process: \(Diagnostic.processForTopmostWindow)
      Current working directory: \(Diagnostic.workingDirectoryForTopmostWindow)
      Current window identifier: \(Diagnostic.descriptionOfTopmostWindow)

      """
    }
  }
}
