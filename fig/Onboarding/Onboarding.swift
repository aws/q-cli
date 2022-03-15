//
//  Onboarding.swift
//  fig
//
//  Created by Matt Schrage on 8/19/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Sentry

extension FileManager {

  func isDirectory(at path: URL) -> Bool {
    var isDir: ObjCBool = false
    let fileExistsAtDestination = self.fileExists(atPath: path.path, isDirectory: &isDir)

    return fileExistsAtDestination && isDir.boolValue
  }

  func recursivelyCopyContentsOfDirectory(at source: URL, to destination: URL) throws {

    var isDir: ObjCBool = false
    let fileExistsAtDestination = self.fileExists(atPath: destination.path, isDirectory: &isDir)

    switch (fileExistsAtDestination, isDir.boolValue) {
    case (true, true):
      break
    case (true, false):
      try self.removeItem(at: destination)
    case (false, _):
      try self.createDirectory(at: destination, withIntermediateDirectories: true, attributes: nil)
    }

    let filepaths = try self.contentsOfDirectory(at: source,
                                             includingPropertiesForKeys: nil,
                                             options: [])
    for file in filepaths {

      if self.isDirectory(at: file) {
        try recursivelyCopyContentsOfDirectory(at: file, to: destination.appendingPathComponent(file.lastPathComponent,
                                                                                             isDirectory: true))
      } else {
        let fileDestination = destination.appendingPathComponent(file.lastPathComponent,
                                                        isDirectory: false)

        if self.fileExists(atPath: fileDestination.path) {
          try self.removeItem(at: fileDestination)
        }
        try copyItem(at: file, to: fileDestination)
      }
    }
  }
}

class Onboarding {

  fileprivate static let configDirectoryInBundle: URL = (Bundle.main.resourceURL?
                                                          .appendingPathComponent("config", isDirectory: true))!

  fileprivate static let figDirectory: URL = URL(fileURLWithPath: NSHomeDirectory() + "/.fig", isDirectory: true)
  fileprivate static let appsDirectory: URL = figDirectory.appendingPathComponent("apps", isDirectory: true)
  fileprivate static let binDirectory: URL = figDirectory.appendingPathComponent("bin", isDirectory: true)
  fileprivate static let dotfilesDirectory: URL = figDirectory
                                                .appendingPathComponent("user",
                                                                        isDirectory: true)
                                                .appendingPathComponent("dotfiles",
                                                                        isDirectory: true)

  static let loginURL: URL = Remote.baseURL.appendingPathComponent("login", isDirectory: true)

  static func setUpEnviroment(completion:( () -> Void)? = nil) {

    if Diagnostic.isRunningOnReadOnlyVolume {
      Logger.log(message: "Currently running on read only volume! App is translocated!")
    }

    guard let figcliPath = Bundle.main.path(forAuxiliaryExecutable: "fig-darwin-universal") else {
      return Logger.log(message: "Could not locate install script!")
    }

    guard let figtermPath = Bundle.main.path(forAuxiliaryExecutable: "figterm") else {
      return Logger.log(message: "Could not locate figterm binary!")
    }

    do {

      // swiftlint:disable identifier_name
      let fs = FileManager.default

      try? fs.createDirectory(at: appsDirectory,
                              withIntermediateDirectories: true,
                              attributes: nil)

      try? fs.createDirectory(at: binDirectory,
                              withIntermediateDirectories: true,
                              attributes: nil)

      try? fs.createDirectory(at: dotfilesDirectory,
                              withIntermediateDirectories: true,
                                              attributes: nil)

      let binaries = try fs.contentsOfDirectory(at: binDirectory,
                                                 includingPropertiesForKeys: nil,
                                                 options: [])
      // delete binary artifacts to ensure ad-hoc code signature works for arm64 binaries on M1
      for binary in binaries {
        try fs.removeItem(at: binary)
      }

      try fs.recursivelyCopyContentsOfDirectory(at: configDirectoryInBundle, to: figDirectory)

      // rename figterm binaries to mirror supported shell
      // copy binaries on install to avoid issues with file permissions at runtime
      let supportedShells = ["zsh", "bash", "fish"]
      for shell in supportedShells {
        try fs.copyItem(atPath: figtermPath,
                        toPath: binDirectory.appendingPathComponent("\(shell) (figterm)").path)
      }

      // Create settings.json file, if it doesn't already exist.
      let settingsFile = figDirectory.appendingPathComponent("settings.json")
      if !fs.fileExists(atPath: settingsFile.path) {
        fs.createFile(atPath: settingsFile.path,
                      contents: "{}".data(using: .utf8),
                      attributes: nil)

      }

    } catch {
      Logger.log(message: "An error occured when attempting to install Fig! " + error.localizedDescription)
      SentrySDK.capture(message: "Installation: " + error.localizedDescription)
      Defaults.shared.lastInstallationError = error.localizedDescription
    }

    // Determine user's login shell by explicitly reading from "/Users/$(whoami)"
    // rather than ~ to handle rare cases where these are different.
    let response = Process.run(command: "/usr/bin/dscl",
                               args: [".", "-read", "/Users/\(NSUserName())", "UserShell"])

    if response.exitCode == 0 {
      Defaults.shared.userShell = response.output.joined(separator: "")
      // read from Defaults to reuse parsing logic
      LocalState.shared.set(value: Defaults.shared.userShell,
                            forKey: LocalState.userShell)
    } else {
      Logger.log(message: "Could not determine user shell. Error \(response.exitCode):" +
                 response.error.joined(separator: "\n"))
    }

    // Create config file and add default values if they do not exist
    Config.shared.addIfNotPresent(key: "FIG_LOGGED_IN", value: "0")
    Config.shared.addIfNotPresent(key: "FIG_ONBOARDING", value: "0")
    LocalState.shared.addIfNotPresent(key: LocalState.hasSeenOnboarding, value: false)

    // Install binaries in the appropriate location222
    symlinkBundleExecutable("figterm", to: binDirectory.appendingPathComponent("figterm").path)
    symlinkBundleExecutable("fig_get_shell", to: binDirectory.appendingPathComponent("fig_get_shell").path)
    copyFigCLIExecutable(to: "~/.local/bin/fig")
    copyFigCLIExecutable(to: "~/.fig/bin/fig")

    // Install launch agent that watches for Fig.app being trashed
    LaunchAgent.uninstallWatcher.addIfNotPresent()

    "\(figcliPath) install --no-confirm".runInBackground(
                                                completion: { _ in
                                                  completion?()
                                                })
  }

  static func symlinkBundleExecutable(_ executable: String, to path: String) {
    let fullPath = NSString(string: path).expandingTildeInPath
    let existingSymlink = try? FileManager.default.destinationOfSymbolicLink(atPath: fullPath)

    if let cliPath = Bundle.main.path(forAuxiliaryExecutable: executable), existingSymlink != cliPath {
      do {
        try? FileManager.default.removeItem(atPath: fullPath)
        let fullURL = URL(fileURLWithPath: fullPath)
        try? FileManager.default.createDirectory(
          at: fullURL.deletingLastPathComponent(),
          withIntermediateDirectories: true,
          attributes: [:]
        )
        try FileManager.default.createSymbolicLink(at: fullURL, withDestinationURL: URL(fileURLWithPath: cliPath))
      } catch {
        Logger.log(message: "Could not symlink executable '\(executable)' to '\(path)'")
        SentrySDK.capture(message: "Could not symlink executable '\(executable)' to '\(path)'")
      }
    }

  }

  static func copyFigCLIExecutable(to path: String) {
    symlinkBundleExecutable("fig-darwin-universal", to: path)
  }

  static func setupTerminalsForShellOnboarding(completion: (() -> Void)? = nil) {
    WindowManager.shared.newNativeTerminalSession(completion: completion)
  }
}

import FigAPIBindings
import WebKit
extension Onboarding {
  static func handleRequest(
    _ request: Fig_OnboardingRequest,
    in webView: WKWebView,
    callback: @escaping ((Bool) -> Void)
  ) {

    switch request.action {
    case .installationScript:
      Onboarding.setUpEnviroment {
        callback(true)
      }
    case .promptForAccessibilityPermission:
      Accessibility.promptForPermission { _ in
        callback(true)
      }
    case .closeAccessibilityPromptWindow:
      Accessibility.closeUI()
    case .launchShellOnboarding:
      callback(true)
      webView.window?.close()
      Defaults.shared.loggedIn = true

      Onboarding.setupTerminalsForShellOnboarding {
        SecureKeyboardInput.notifyIfEnabled()
      }

      NSApp.appDelegate.setupCompanionWindow()
    case .uninstall:
      NSApp.appDelegate.uninstall(showDialog: true)
    case .UNRECOGNIZED:
      Logger.log(message: "Unrecognized Onboarding Action!", subsystem: .api)
      callback(false)
    }
  }
}
