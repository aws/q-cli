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
        return "\(app.localizedName ?? "") - \(app.bundleIdentifier ?? "") \(SecureKeyboardInput.enabled(by: app.bundleIdentifier) ? "(via Settings)" : "")"
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
  
  static var keybindingsPath: String? {
    get {
      let path = KeyBindingsManager.keymapFilePath.path
      
      return FileManager.default.fileExists(atPath: path) ? path : nil
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
  
  static var shellIntegrationAddedToDotfiles: Bool {
    let dotfiles = [
      ".bashrc",
      ".bash_profile",
      ".zshrc",
      ".zprofile",
      ".profile"
    ]
    
    let target = "source ~/.fig/fig.sh"
    
    return dotfiles.reduce(true) { (result, file) -> Bool in
      guard result else {
        return false
      }
      let filepath = "\(NSHomeDirectory())/\(file)"
      guard FileManager.default.fileExists(atPath: filepath) else {
        return true
      }
      
      guard let contents = try? String(contentsOfFile: filepath) else {
        return true
      }
      
      return contents.contains(target)
    }
  }
  
  static var dotfigFolderIsSetupCorrectly: Bool {
    let dotfig = "\(NSHomeDirectory())/.fig"
    
    // Integration setup files
    let integrations = ["tmux", "ssh"]
    
    
    let settings = [ "settings.json" ]
    
    // Shell Hooks
    let shellHooks = [
      "fig.sh",
      "shell/bash-preexec.sh",
      "shell/post.bash",
      "shell/post.fish",
      "shell/post.sh",
      "shell/post.zsh",
      "shell/pre.fish",
      "shell/pre.sh",
      "shell/zle.zsh",
      "zle" // make sure folder exists
    ]
    
    let onboarding = [
      "tools",
      "tools/drip",
      "tools/drip/fig_onboarding.sh",
      "user/config"
    ]
    
    let filesAndFolders = integrations +
                              settings +
                            shellHooks +
                            onboarding + [ "autocomplete" ]
      
    
    return filesAndFolders.reduce(true) { (exists, path) -> Bool in
      var isDir : ObjCBool = false
      return exists && FileManager.default.fileExists(atPath: "\(dotfig)/\(path)", isDirectory:&isDir)

    }
  }
  
  static var installationScriptRan: Bool {
    
    let folderContainsExpectedFiles = Diagnostic.dotfigFolderIsSetupCorrectly
    
    let shellIntegrationIsManagedByUser = Settings.shared.getValue(forKey: Settings.shellIntegrationIsManagedByUser) as? Bool ?? false
    
    if shellIntegrationIsManagedByUser {
      return folderContainsExpectedFiles
    }
    
    return folderContainsExpectedFiles && shellIntegrationAddedToDotfiles
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
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.terminalsWhereAutocompleteShouldAppear.contains(app.bundleIdentifier ?? ""),
            let window = AXWindowServer.shared.whitelistedWindow,
            let tty = window.tty
      else {
        return "???"
      }

      return tty.cmd != nil ? "\(tty.cmd ?? "")" : "<Unknown Process>"
    }
  }
  
  static var processIdForTopmostWindow: String {
    get {
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.terminalsWhereAutocompleteShouldAppear.contains(app.bundleIdentifier ?? ""),
            let window = AXWindowServer.shared.whitelistedWindow,
            let tty = window.tty
      else {
        return "???"
      }

      return tty.pid != nil ? "\(tty.pid ?? -1)" : "???"
    }
  }
  
  static var workingDirectoryForTopmostWindow: String {
    get {
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.terminalsWhereAutocompleteShouldAppear.contains(app.bundleIdentifier ?? ""),
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
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.terminalsWhereAutocompleteShouldAppear.contains(app.bundleIdentifier ?? ""),
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
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.terminalsWhereAutocompleteShouldAppear.contains(app.bundleIdentifier ?? ""),
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
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.terminalsWhereAutocompleteShouldAppear.contains(app.bundleIdentifier ?? ""),
            let window = AXWindowServer.shared.whitelistedWindow
      else {
        return "???"
      }

      return "\(window.hash) (\(window.bundleId ?? "???"))"
    }
  }
  
  static var keybufferHasContextForTopmostWindow: Bool {
    get {
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.terminalsWhereAutocompleteShouldAppear.contains(app.bundleIdentifier ?? ""),
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
      guard let app = NSWorkspace.shared.frontmostApplication, Integrations.terminalsWhereAutocompleteShouldAppear.contains(app.bundleIdentifier ?? ""),
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
    return "Version \(Diagnostic.version) (B\(Diagnostic.build))\(Defaults.isProduction ? "" : " [\(Defaults.build.rawValue)]")"
  }
  
  static var pseudoTerminalPath: String? {
    return Settings.shared.getValue(forKey: Settings.ptyPathKey) as? String
  }
  
  static var pseudoTerminalPathAppearsValid: Bool? {
    guard let path = Diagnostic.pseudoTerminalPath else {
      return nil
    }
    
    return path.contains("/usr/bin")
  }
  
  static var settingsExistAndHaveValidFormat: Bool {
    return Settings.haveValidFormat
  }
  
  static var dotfilesAreSymlinked: Bool {
    let dotfiles = [".profile", ".bashrc", ".bash_profile", ".zshrc", ".zprofile", ".config/fish/config.fish", ".tmux.conf", ".ssh/config"]
    
    return dotfiles.reduce(false) { (existingSymlink, path) -> Bool in
      guard !existingSymlink else {
        return existingSymlink
      }
      
      return (try? FileManager.default.destinationOfSymbolicLink(atPath: "\(NSHomeDirectory())/\(path)")) != nil
    }
  }
  
  //https://github.com/sparkle-project/Sparkle/blob/3a5c620b60f483b71f8c28573ac29bf85fda6193/Sparkle/SUHost.m#L178-L183
  
  // Check if app is translocated
  static var isRunningOnReadOnlyVolume: Bool {
    get {
      let url = Bundle.main.bundleURL as NSURL
      var resourceValue: AnyObject?

      do {
        try url.getResourceValue(&resourceValue, forKey: URLResourceKey.volumeIsReadOnlyKey)
      } catch {
        return false
      }
      
      if let isReadOnly = resourceValue as? NSNumber {
        return isReadOnly.boolValue
      } else {
        return false
      }
    }
  }

  static var summary: String {
    get {
      """
      
      \(Diagnostic.distribution) \(Defaults.beta ? "[Beta] " : "")\(Defaults.debugAutocomplete ? "[Debug] " : "")\(Defaults.developerModeEnabled ? "[Dev] " : "")[\(KeyboardLayout.shared.currentLayoutName() ?? "?")] \(Diagnostic.isRunningOnReadOnlyVolume ? "TRANSLOCATED!!!" : "")
      UserShell: \(Defaults.userShell)
      Bundle path: \(Diagnostic.pathToBundle)
      Autocomplete: \(Defaults.useAutocomplete)
      Settings.json: \(Diagnostic.settingsExistAndHaveValidFormat)
      CLI installed: \(Diagnostic.installedCLI)
      CLI tool path: \(Diagnostic.pathOfCLI ?? "<none>")
      Accessibility: \(Accessibility.enabled)
      Number of specs: \(Diagnostic.numberOfCompletionSpecs)
      SSH Integration: \(Defaults.SSHIntegrationEnabled)
      Tmux Integration: \(TmuxIntegration.isInstalled)
      Keybindings path: \(Diagnostic.keybindingsPath ?? "<none>")
      iTerm Integration: \(iTermIntegration.isInstalled) \(iTermIntegration.shared.isConnectedToAPI ? "[Authenticated]": "")
      Hyper Integration: \(HyperIntegration.isInstalled)
      VSCode Integration: \(VSCodeIntegration.isInstalled)
      Docker Integration: \(DockerEventStream.shared.socket.isConnected)
      Symlinked dotfiles: \(Diagnostic.dotfilesAreSymlinked)
      Only insert on tab: \(Defaults.onlyInsertOnTab)
      Installation Script: \(Diagnostic.installationScriptRan)
      PseudoTerminal Path: \(Diagnostic.pseudoTerminalPath ?? "<generated dynamically>")
      SecureKeyboardInput: \(Diagnostic.secureKeyboardInput)
      SecureKeyboardProcess: \(Diagnostic.blockingProcess ?? "<none>")
      Current active process: \(Diagnostic.processForTopmostWindow) (\(Diagnostic.processIdForTopmostWindow)) - \(Diagnostic.ttyDescriptorForTopmostWindow)
      Current working directory: \(Diagnostic.workingDirectoryForTopmostWindow)
      Current window identifier: \(Diagnostic.descriptionOfTopmostWindow)

      """
    }
  }
  
  static func summaryWithEnvironment(_ env: [String: Any]) -> String {
    let relevantEnvironmentVariables =
    """
    PATH: \(env["PATH"] as? String ?? "???")
    FIG_INTEGRATION_VERSION: \(env["FIG_INTEGRATION_VERSION"] as? String ?? "???")
    """
    return Diagnostic.summary + relevantEnvironmentVariables
    
  }
}
