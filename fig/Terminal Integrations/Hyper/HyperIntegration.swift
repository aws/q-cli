//
//  HyperIntegration.swift
//  fig
//
//  Created by Matt Schrage on 2/8/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import Sentry

class HyperIntegration: TerminalIntegrationProvider {
  static let `default` = HyperIntegration(bundleIdentifier: Integrations.Hyper)
  static let settingsPath = "\(NSHomeDirectory())/.hyper.js"

  static var settings: [String: Any]? {
    guard let settings = try? String(contentsOfFile: settingsPath),
          let json = settings.parseAsJSON() else {
      return nil
    }

    return json
  }

  // If the extension path changes make sure to update the uninstall script!
  static let pluginPath: URL = URL(
    fileURLWithPath: "\(NSHomeDirectory())/.hyper_plugins/local/fig-hyper-integration/index.js"
  )
  static let pluginPathInBundle = Bundle.main.url(forResource: "hyper-integration", withExtension: "js")!

  func uninstall() -> Bool {
    guard let settings = try? String(contentsOfFile: HyperIntegration.settingsPath) else {
      return false
    }
    let updatedSettings = settings
      .replacingOccurrences(of: "\"fig-hyper-integration\"", with: "")
      .replacingOccurrences(of: "\"fig-hyper-integration\",", with: "")

    do {
      try updatedSettings.write(toFile: HyperIntegration.settingsPath, atomically: true, encoding: .utf8)
    } catch {
      return false
    }

    return true
  }

  func install() -> InstallationStatus {
    guard NSWorkspace.shared.applicationIsInstalled(self.bundleIdentifier) else {
      return .applicationNotInstalled
    }

    do {
      try FileManager.default.createDirectory(
        atPath: HyperIntegration.pluginPath.deletingLastPathComponent().path,
        withIntermediateDirectories: true
      )
      try? FileManager.default.removeItem(at: HyperIntegration.pluginPath)
      try FileManager.default.createSymbolicLink(
        at: HyperIntegration.pluginPath,
        withDestinationURL: HyperIntegration.pluginPathInBundle
      )
    } catch {

      guard let destination =
              try? FileManager.default.destinationOfSymbolicLink(atPath: HyperIntegration.pluginPath.path),
            destination == HyperIntegration.pluginPathInBundle.path else {
        return .failed(error: "Could not create symbolic link to plugin in \(HyperIntegration.pluginPath)")
      }
    }

    var updatedSettings: String!

    guard let settings = try? String(contentsOfFile: HyperIntegration.settingsPath) else {
      return .failed(error: "Could not read Hyper settings")
    }

    let noExistingPlugins =
      """
    localPlugins: [
      "fig-hyper-integration"
    ]
    """

    let existingPlugins =
      """
    localPlugins: [
      "fig-hyper-integration",
    """

    if settings.contains("fig-hyper-integration") {
      print("~/hyper.js already contains 'fig-hyper-integration'")
      updatedSettings = settings

    } else if
      settings.contains("localPlugins: []") {
      updatedSettings = settings.replacingOccurrences(of: "localPlugins: []", with: noExistingPlugins)
    } else if settings.contains("localPlugins: [") {
      updatedSettings = settings.replacingOccurrences(of: "localPlugins: [\n", with: existingPlugins)
    } else {
      // swiftlint:disable line_length
      return .failed(error: "In order for Fig to work with multiple tabs/panes in Hyper, you will need to setup the integration manually by editing ~/hyper.js.",
                     supportURL: URL(string: "https://fig.io/support/terminals/hyper"))
    }

    do {
      try updatedSettings.write(toFile: HyperIntegration.settingsPath, atomically: true, encoding: .utf8)
    } catch {
      return .failed(error: "Could not write to '\(HyperIntegration.settingsPath)' to update localPlugins.")
    }

    return .pending(event: .applicationRestart)

  }

  func verifyInstallation() -> InstallationStatus {
    guard self.applicationIsInstalled else {
      return .applicationNotInstalled
    }

    guard let settings = try? String(contentsOfFile: HyperIntegration.settingsPath) else {
      return .failed(error: "Could not read Hyper settings file (\(HyperIntegration.settingsPath))")
    }

    guard settings.contains("fig-hyper-integration") else {
      return .failed(error: "hyper.js must include `fig-hyper-integration` in localPlugins.")
    }

    guard FileManager.default.fileExists(atPath: HyperIntegration.pluginPath.path) else {
      return .failed(error: "Hyper plugin does not exists at \(HyperIntegration.pluginPath.path)")
    }

    return .installed

  }

}

extension HyperIntegration {
  func getCursorRect(in window: ExternalWindow) -> NSRect? {
    return Accessibility.findXTermCursorInElectronWindow(window)
  }

  func terminalIsFocused(in window: ExternalWindow) -> Bool {
    return true
  }
}
