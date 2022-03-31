//
//  TabbyIntegration.swift
//  fig
//
//  Created by Matt Schrage on 12/20/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class TabbyIntegration: TerminalIntegrationProvider {
  static let `default` = TabbyIntegration(bundleIdentifier: Integrations.Tabby)

  static let pluginsDirectory = URL(fileURLWithPath: NSSearchPathForDirectoriesInDomains(.applicationSupportDirectory,
                                                                    .userDomainMask,
                                                                    true).first!)
                                .appendingPathComponent("tabby", isDirectory: true)
                                .appendingPathComponent("plugins", isDirectory: true)
                                .appendingPathComponent("node_modules", isDirectory: true)

  static let pluginName = "tabby-plugin-fig-integration"
  static let pluginFolderPath = TabbyIntegration.pluginsDirectory.appendingPathComponent(TabbyIntegration.pluginName,
                                                                                     isDirectory: true)
  static let pluginPath = TabbyIntegration.pluginFolderPath.appendingPathComponent(
                          TabbyIntegration.pluginPathInBundle.lastPathComponent)

  static let pluginVersion =  "0.1.1"
  static let pluginPathInBundle = Bundle.main.url(forResource: "tabby-integration", withExtension: "js")!

  static let packageJSON =
  """
  {
    "name": "\(pluginName)",
    "version": "\(pluginVersion)",
    "description": "Fig integration for Tabby",
    "keywords": [
      "tabby-plugin"
    ],
    "author": "Fig",
    "main": "\(pluginPathInBundle.lastPathComponent)"
  }
  """

  func verifyInstallation() -> InstallationStatus {
    guard self.applicationIsInstalled else {
      return .applicationNotInstalled
    }

    guard let destination = try? FileManager.default.destinationOfSymbolicLink(atPath:
                                                                                TabbyIntegration.pluginPath.path),
          destination == TabbyIntegration.pluginPathInBundle.path else {
      return .failed(error: "Symlinked js file '\(TabbyIntegration.pluginPath.path)'" +
                            " does not point to expected destination")
    }

    return .installed
  }

  func uninstall() -> Bool {
    if (try? FileManager.default.removeItem(at: TabbyIntegration.pluginFolderPath)) == nil {
      return false
    }
    if (try? FileManager.default.removeItem(at: TabbyIntegration.pluginPathInBundle)) == nil {
      return false
    }

    return true
  }

  func install() -> InstallationStatus {
    guard self.applicationIsInstalled else {
      return .applicationNotInstalled
    }

    do {
      let figPluginDirectory = TabbyIntegration.pluginFolderPath
      try? FileManager.default.createDirectory(at: figPluginDirectory,
                                              withIntermediateDirectories: true,
                                              attributes: nil)

      try TabbyIntegration.packageJSON.write(toFile: figPluginDirectory.appendingPathComponent("package.json").path,
                                         atomically: true,
                                         encoding: .utf8)

      try? FileManager.default.removeItem(at: TabbyIntegration.pluginPath)

      try FileManager.default.createSymbolicLink(
        at: TabbyIntegration.pluginPath,
        withDestinationURL: TabbyIntegration.pluginPathInBundle
      )
    } catch {
      return .failed(error: "Could not install Tabby plugin")
    }

    return .pending(event: .applicationRestart)
  }
}

extension TabbyIntegration {
  func getCursorRect(in window: ExternalWindow) -> NSRect? {
    return Accessibility.findXTermCursorInElectronWindow(window)
  }

  func terminalIsFocused(in window: ExternalWindow) -> Bool {
    return true
  }
}
