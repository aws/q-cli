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
      return CGSIsSecureEventInputSet()
    }
  }
  
  static var blockingProcess: String? {
    get {
      guard Diagnostic.secureKeyboardInput else { return nil }
      
      var pid: pid_t = 0;
      secure_keyboard_entry_process_info(&pid)
      if let app = NSRunningApplication(processIdentifier: pid) {
        return "\(app.localizedName ?? "") - \(app.bundleIdentifier ?? "")"
      } else {
        return "no app for pid"
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
      return FileManager.default.fileExists(atPath: "\(NSHomeDirectory())/.fig/bin/fig")
    }
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
      Usershell: \(Defaults.userShell)
      Autocomplete: \(Defaults.useAutocomplete)
      CLI installed: \(Diagnostic.installedCLI)
      Accessibility: \(Accessibility.enabled)
      Number of specs: \(Diagnostic.numberOfCompletionSpecs)
      SSH Integration: \(Defaults.SSHIntegrationEnabled)
      Only insert on tab: \(Defaults.onlyInsertOnTab)
      SecureKeyboardInput: \(Diagnostic.secureKeyboardInput)
      SecureKeyboardProcess: \(Diagnostic.blockingProcess ?? "<none>")
      iTerm Tab Integration: \(iTermTabIntegration.isInstalled())
      Current active process: \(processForTopmostWindow)
      Current working directory: \(workingDirectoryForTopmostWindow)
      
      """
    }
  }
}
