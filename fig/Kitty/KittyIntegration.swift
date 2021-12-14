//
//  KittyIntegration.swift
//  fig
//
//  Created by Matt Schrage on 9/13/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class KittyIntegration: InputMethodDependentTerminalIntegrationProvider {
  static let `default` = KittyIntegration(bundleIdentifier: Integrations.Kitty)

  static let configDirectory: URL = URL(fileURLWithPath: NSHomeDirectory() + "/.config/kitty")
  // https://sw.kovidgoyal.net/kitty/faq/#how-do-i-specify-command-line-options-for-kitty-on-macos
  static let cmdlineFilename = "macos-launch-services-cmdline"
  static let cmdlineFilepath = configDirectory.appendingPathComponent(cmdlineFilename)
  static let pythonScriptPathInBundle = Bundle.main.path(forResource: "kitty-integration", ofType: "py")!
  static let pythonScriptPath = NSHomeDirectory() + "/.fig/tools/kitty-integration.py"

  static let commandLineArguments = "--watcher \(pythonScriptPath)"
  fileprivate static let minimumSupportedVersion = SemanticVersion(version: "0.20.0")!

}

extension KittyIntegration: IntegrationProvider {
  func verifyInstallation() -> InstallationStatus {

    guard self.applicationIsInstalled else {
      return .applicationNotInstalled
    }

    if let failed = self.currentVersionIsSupported(minimumVersion: KittyIntegration.minimumSupportedVersion) {
      return failed
    }

    guard FileManager.default.fileExists(atPath: KittyIntegration.cmdlineFilepath.path) else {
      return .failed(error: "'\(KittyIntegration.cmdlineFilepath.path)' file does not exist")
    }

    guard let kittyCommandLine = try? String(contentsOf: KittyIntegration.cmdlineFilepath) else {
      return .failed(error: "Could not read '\(KittyIntegration.cmdlineFilepath.path)'")
    }

    guard kittyCommandLine.contains(KittyIntegration.commandLineArguments) else {
      return .failed(error: "\(KittyIntegration.cmdlineFilename) does not contains --watcher")
    }

    let inputMethodStatus = InputMethod.default.verifyInstallation()
    guard inputMethodStatus == .installed else {
      return .pending(event: .inputMethodActivation)
    }

    return .installed
  }

  func install() -> InstallationStatus {
    guard self.applicationIsInstalled else {
      return .applicationNotInstalled
    }

    if let failed = self.currentVersionIsSupported(minimumVersion: KittyIntegration.minimumSupportedVersion) {
      return failed
    }

    try? FileManager.default.removeItem(atPath: KittyIntegration.pythonScriptPath)

    do {
      try FileManager.default.createSymbolicLink(
        atPath: KittyIntegration.pythonScriptPath,
        withDestinationPath: KittyIntegration.pythonScriptPathInBundle
      )
    } catch {
      return .failed(
        error: "Could not create symlink at \(KittyIntegration.pythonScriptPath): \(error.localizedDescription)"
      )
    }

    if FileManager.default.fileExists(atPath: KittyIntegration.cmdlineFilepath.path) {

      guard let kittyCommandLine = try? String(contentsOf: KittyIntegration.cmdlineFilepath) else {
        return .failed(error: "Could not read '\(KittyIntegration.cmdlineFilepath.path)'")
      }

      guard kittyCommandLine.contains(KittyIntegration.commandLineArguments) else {
        return .failed(error: "\(KittyIntegration.cmdlineFilename) already exists and contains user-specified configuration.",
                       supportURL: URL(string: "https://fig.io/support/terminals/kitty"))
      }

    } else {
      do {
        try KittyIntegration.commandLineArguments.write(toFile: KittyIntegration.cmdlineFilepath.path,
                                                        atomically: true,
                                                        encoding: .utf8)
      } catch {
        return .failed(error: "Could not write to \(KittyIntegration.cmdlineFilename)")
      }
    }

    if !InputMethod.default.isInstalled {
      _ = InputMethod.default.install()
    }

    return .pending(event: .applicationRestart)
  }
}

extension KittyIntegration: TerminalIntegration {
  func getCursorRect(in window: ExternalWindow) -> NSRect? {
    return InputMethod.getCursorRect()
  }

  func terminalIsFocused(in window: ExternalWindow) -> Bool {
    return true
  }

}
