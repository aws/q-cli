//
//  Diagnostic.swift
//  fig
//
//  Created by Matt Schrage on 1/28/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import FigAPIBindings

class Diagnostic {
  static var accessibility: Bool {
    return Accessibility.enabled
  }

  static var secureKeyboardInput: Bool {
    return SecureKeyboardInput.enabled
  }

  static var blockingProcess: String? {
    guard SecureKeyboardInput.enabled else { return nil }

    if let app = SecureKeyboardInput.responsibleApplication {
      // swiftlint:disable line_length
      return "\(app.localizedName ?? "") - \(app.bundleIdentifier ?? "") \(SecureKeyboardInput.enabled(by: app.bundleIdentifier) ? "(via Settings)" : "")"
    } else {
      return "no app for pid '\(SecureKeyboardInput.responsibleProcessId ?? -1)'"
    }
  }

  static var userConfig: String? {
    return try? String(contentsOfFile: "\(NSHomeDirectory())/.fig/user/config", encoding: String.Encoding.utf8)
  }

  static var installedCLI: Bool {
    guard let path = Diagnostic.pathOfCLI,
          let symlink = try? FileManager.default.destinationOfSymbolicLink(atPath: path) else {
      return false
    }

    return FileManager.default.fileExists(atPath: path) && FileManager.default.fileExists(atPath: symlink)
  }

  static var pathOfCLI: String? {
    var location: String?

    if FileManager.default.fileExists(atPath: "\(NSHomeDirectory())/.fig/bin/fig") {
      location = "\(NSHomeDirectory())/.fig/bin/fig"
    } else if FileManager.default.fileExists(atPath: "/usr/local/bin/fig") {
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

    let target = "fig init"

    return dotfiles.allSatisfy { (file) -> Bool in
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
    let integrations = [ "ssh" ]

    let settings = [ "settings.json" ]

    let filesAndFolders = integrations +
      settings

    return filesAndFolders.allSatisfy { (path) -> Bool in
      var isDir: ObjCBool = false
      return FileManager.default.fileExists(atPath: "\(dotfig)/\(path)", isDirectory: &isDir)

    }
  }

  static var installationScriptRan: Bool {

    let folderContainsExpectedFiles = Diagnostic.dotfigFolderIsSetupCorrectly

    let shellIntegrationIsManagedByUser = Settings.shared.getValue(forKey:
                                                                    Settings.shellIntegrationIsManagedByUser) as? Bool ?? false

    if shellIntegrationIsManagedByUser {
      return folderContainsExpectedFiles
    }

    return folderContainsExpectedFiles && shellIntegrationAddedToDotfiles
  }

  static var pathToBundle: String {
    return Bundle.main.bundlePath
  }

  static var numberOfCompletionSpecs: Int {
    return (try? FileManager.default.contentsOfDirectory(atPath: "\(NSHomeDirectory())/.fig/autocomplete").count) ?? 0
  }

  static var processForTopmostWindow: String {
    guard Integrations.frontmostApplicationIsValidTerminal(),
          let window = AXWindowServer.shared.allowlistedWindow,
          let context = window.associatedShellContext
    else {
      return "???"
    }

    return context.executablePath
  }

  static var processIdForTopmostWindow: String {
    guard Integrations.frontmostApplicationIsValidTerminal(),
          let window = AXWindowServer.shared.allowlistedWindow,
          let context = window.associatedShellContext
    else {
      return "???"
    }

    return "\(context.processId)"
  }

  static var workingDirectoryForTopmostWindow: String {
    guard Integrations.frontmostApplicationIsValidTerminal(),
          let window = AXWindowServer.shared.allowlistedWindow,
          let context = window.associatedShellContext
    else {
      return "???"
    }

    return context.workingDirectory
  }

  static var processIsShellInTopmostWindow: Bool {
    guard Integrations.frontmostApplicationIsValidTerminal(),
          let window = AXWindowServer.shared.allowlistedWindow,
          let context = window.associatedShellContext
    else {
      return false
    }

    return context.isShell()
  }

  static var ttyDescriptorForTopmostWindow: String {
    guard Integrations.frontmostApplicationIsValidTerminal(),
          let window = AXWindowServer.shared.allowlistedWindow,
          let context = window.associatedShellContext
    else {
      return "???"
    }

    return context.ttyDescriptor
  }

  static var descriptionOfTopmostWindow: String {
    guard Integrations.frontmostApplicationIsValidTerminal(),
          let window = AXWindowServer.shared.allowlistedWindow
    else {
      return "???"
    }

    return "\(window.hash) (\(window.bundleId ?? "???"))"
  }

  static var sessionForTopmostWindow: String {
    guard Integrations.frontmostApplicationIsValidTerminal(),
          let window = AXWindowServer.shared.allowlistedWindow
    else {
      return "???"
    }

    return window.session ?? "???"
  }

  static var keybufferHasContextForTopmostWindow: Bool {
    guard Integrations.frontmostApplicationIsValidTerminal() else {
      return false
    }

    return AXWindowServer.shared.allowlistedWindow != nil
  }

  static var version: String {
    return Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "-1"
  }

  static var build: String {
    return Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? ""
  }

  static var distribution: String {
    // swiftlint:disable line_length
    return "Version \(Diagnostic.version) (B\(Diagnostic.build))\(Defaults.shared.isProduction ? "" : " [\(Defaults.shared.build.rawValue)]")"
  }

  static var pseudoTerminalPath: String? {
    return LocalState.shared.getValue(forKey: LocalState.ptyPathKey) as? String
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
    let dotfiles = [
      ".profile",
      ".bashrc",
      ".bash_profile",
      ".zshrc",
      ".zprofile",
      ".config/fish/config.fish",
      ".tmux.conf",
      ".ssh/config"
    ]

    return dotfiles.contains { (path) -> Bool in
      return (try? FileManager.default.destinationOfSymbolicLink(atPath: "\(NSHomeDirectory())/\(path)")) != nil
    }
  }

  //https://github.com/sparkle-project/Sparkle/blob/3a5c620b60f483b71f8c28573ac29bf85fda6193/Sparkle/SUHost.m#L178-L183

  // Check if app is translocated
  static var isRunningOnReadOnlyVolume: Bool {
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

  static var unixSocketServerExists: Bool {
    let path = IPC.unixSocket.path
    return FileManager.default.fileExists(atPath: path)
  }

  static func setDebuggerStatus(_ request: Fig_DebuggerUpdateRequest) throws -> Bool {
    var color = NSColor.clear
    if request.hasColor, let hex = NSColor(hex: request.color) {
      color = hex
    }

    if request.layout.count == 0 {
      debuggerStatusFromWeb = nil
    } else {
      debuggerStatusFromWeb = (color, request.layout)
    }

    return true
  }

  static var debuggerStatusFromWeb: (NSColor, [String])?

  // swiftlint:disable line_length
  static var summary: String {
    """

    \(Diagnostic.distribution) \(Defaults.shared.beta ? "[Beta] " : "")\(Defaults.shared.debugAutocomplete ? "[Debug] " : "")\(Defaults.shared.developerModeEnabled ? "[Dev] " : "")[\(KeyboardLayout.shared.currentLayoutName() ?? "?")] \(Diagnostic.isRunningOnReadOnlyVolume ? "TRANSLOCATED!!!" : "")
    UserShell: \(Defaults.shared.userShell)
    Bundle path: \(Diagnostic.pathToBundle)
    Autocomplete: \(Defaults.shared.useAutocomplete)
    Settings.json: \(Diagnostic.settingsExistAndHaveValidFormat)
    CLI installed: \(Diagnostic.installedCLI)
    CLI tool path: \(Diagnostic.pathOfCLI ?? "<none>")
    Accessibility: \(Accessibility.enabled)
    SSH Integration: \(Defaults.shared.SSHIntegrationEnabled)
    Tmux Integration: \(TmuxIntegration.isInstalled)
    iTerm Integration: \(iTermIntegration.default.isInstalled) \(iTermIntegration.default.isConnectedToAPI ? "[Authenticated]": "")
    Hyper Integration: \(HyperIntegration.default.isInstalled)
    VSCode Integration: \(VSCodeIntegration.default.isInstalled)
    Docker Integration: \(DockerEventStream.shared.socket.isConnected)
    Symlinked dotfiles: \(Diagnostic.dotfilesAreSymlinked)
    Only insert on tab: \(Defaults.shared.onlyInsertOnTab)
    UNIX Socket Exists: \(Diagnostic.unixSocketServerExists)
    Installation Script: \(Diagnostic.installationScriptRan)
    PseudoTerminal Path: \(Diagnostic.pseudoTerminalPath ?? "<generated dynamically>")
    SecureKeyboardInput: \(Diagnostic.secureKeyboardInput)
    SecureKeyboardProcess: \(Diagnostic.blockingProcess ?? "<none>")
    Current active process: \(Diagnostic.processForTopmostWindow) (\(Diagnostic.processIdForTopmostWindow)) - \(Diagnostic.ttyDescriptorForTopmostWindow)
    Current terminal session: \(Diagnostic.sessionForTopmostWindow)
    Current working directory: \(Diagnostic.workingDirectoryForTopmostWindow)
    Current window identifier: \(Diagnostic.descriptionOfTopmostWindow)

    """
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
