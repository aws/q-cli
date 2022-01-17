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

  // https://github.com/kovidgoyal/kitty/blob/04807453ecf4a15ef2d49485d619410b2c25151c/kitty/constants.py#L57
  static let configLocation: URL = URL(fileURLWithPath: NSHomeDirectory() + "/.config/kitty/kitty.conf")
  // https://sw.kovidgoyal.net/kitty/faq/#how-do-i-specify-command-line-options-for-kitty-on-macos
  static let pythonScriptPathInBundle = Bundle.main.path(forResource: "kitty-integration", ofType: "py")!
  static func pythonScriptPath(usingKittyVariable: Bool = false) -> String {
    return (usingKittyVariable ? "${HOME}" : NSHomeDirectory())
          + "/.fig/tools/kitty-integration.py"
  }

  static let configKey = "\n# Fig Kitty Integration: Enabled\nwatcher \(pythonScriptPath(usingKittyVariable: true))\n# End of Fig Kitty Integration\n"
  fileprivate static let minimumSupportedVersion = SemanticVersion(version: "0.24.0")!

}

extension KittyIntegration: IntegrationProvider {
  func verifyInstallation() -> InstallationStatus {

    guard self.applicationIsInstalled else {
      return .applicationNotInstalled
    }

    if let failed = self.currentVersionIsSupported(minimumVersion: KittyIntegration.minimumSupportedVersion) {
      return failed
    }

    guard let kittyConfig = try? String(contentsOf: KittyIntegration.configLocation) else {
      return .failed(error: "Could not read '\(KittyIntegration.configLocation.path)'")
    }

    guard kittyConfig.contains(KittyIntegration.configKey) else {
      return .failed(error: "watcher is not included in kitty.conf")
    }

    let inputMethodStatus = InputMethod.default.verifyInstallation()
    guard inputMethodStatus == .installed else {
      return .pending(event: .inputMethodActivation)
    }

    // If the application is already running,
    // it must be restarted for the new input method to work
    if self.status == .pending(event: .inputMethodActivation) {
      return .pending(event: .applicationRestart)
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

    try? FileManager.default.removeItem(atPath: KittyIntegration.pythonScriptPath())

    do {
      try FileManager.default.createSymbolicLink(
        atPath: KittyIntegration.pythonScriptPath(),
        withDestinationPath: KittyIntegration.pythonScriptPathInBundle
      )
    } catch {
      return .failed(
        error: "Could not create symlink at \(KittyIntegration.pythonScriptPath()): \(error.localizedDescription)"
      )
    }

    if FileManager.default.fileExists(atPath: KittyIntegration.configLocation.path) {

      guard let kittyConfig = try? String(contentsOf: KittyIntegration.configLocation) else {
        return .failed(error: "Could not read '\(KittyIntegration.configLocation.path)'")
      }

      if !kittyConfig.contains(KittyIntegration.configKey) {
        if let file = try? FileHandle(forUpdating: KittyIntegration.configLocation) {
          file.seekToEndOfFile()

          file.write(KittyIntegration.configKey.data(using: .utf8)!)
          file.closeFile()
        } else {
          let config = KittyIntegration.configKey.trimmingCharacters(in: .whitespacesAndNewlines)
          return .failed(error: "Could not append '\(config)' to \(KittyIntegration.configLocation.path)")
        }
      }

    } else {
      do {
        try KittyIntegration.configKey.write(toFile: KittyIntegration.configLocation.path,
                                             atomically: true,
                                             encoding: .utf8)
      } catch {
        return .failed(error: "Could not write to \(KittyIntegration.configLocation.path) (\(error.localizedDescription))")
      }
    }

    if !InputMethod.default.isInstalled {
      let status = InputMethod.default.install()
      guard status == .installed else {
        return .pending(event: .inputMethodActivation)
      }

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
