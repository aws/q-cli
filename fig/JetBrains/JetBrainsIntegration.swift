//
//  JetBrainsIntegration.swift
//  fig
//
//  Created by Matt Schrage on 12/22/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import ZIPFoundation

class JetBrainsIntegration: InputMethodDependentTerminalIntegrationProvider & IntegrationProvider {
  // com.jetbrains.intellij.ce
  static let ideaCE   = JetBrainsIntegration(bundleIdentifier: Integrations.IntellijCE)
  static let WebStorm = JetBrainsIntegration(bundleIdentifier: Integrations.WebStorm)
  static let GoLand   = JetBrainsIntegration(bundleIdentifier: Integrations.GoLand)
  static let PhpStorm = JetBrainsIntegration(bundleIdentifier: Integrations.PhpStorm)
  static let PyCharm  = JetBrainsIntegration(bundleIdentifier: Integrations.PyCharm)
  static let AppCode  = JetBrainsIntegration(bundleIdentifier: Integrations.AppCode)

  static let plugin = Plugin(name: "jetbrains-extension",
                             version: "2.0.0",
                             fileExtension: "zip")
  func pluginsPath(for productVersion: String) -> URL {

    // swiftlint:disable line_length
    // https://www.jetbrains.com/help/idea/directories-used-by-the-ide-to-store-settings-caches-plugins-and-logs.html#plugins-directory
    // ~/Library/Application Support/JetBrains/<product><version>/plugins
    return URL.applicationSupport
              .appendingPathComponent("JetBrains", isDirectory: true)
              .appendingPathComponent(productVersion, isDirectory: true)
              .appendingPathComponent("plugins", isDirectory: true)

  }

  fileprivate func getJVMProperties() throws ->  [String: AnyObject] {
    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: self.bundleIdentifier) else {
      throw InstallationStatus.failed(error: "Could not locate URL for application")
    }

    let infoPlistURL = url.appendingPathComponent("Contents", isDirectory: true)
                           .appendingPathComponent("Info.plist")

    guard FileManager.default.fileExists(atPath: infoPlistURL.path) else {
      throw InstallationStatus.failed(error: "Could not locate Info.plist for application")
    }

    guard let info = PropertyList.read(from: infoPlistURL) else {
      throw InstallationStatus.failed(error: "Could not read plist at \(infoPlistURL.path)")
    }

    guard let jvmOption = info["JVMOptions"] as? [String: AnyObject] else {
      throw InstallationStatus.failed(error: "Could not read JVMOptions from plist")
    }

    guard let properties = jvmOption["Properties"] as? [String: AnyObject] else {
      throw InstallationStatus.failed(error: "Could not read `Properties` from JVMOptions")
    }

    return properties
  }
  func pluginsPath() throws -> URL {

    let properties = try getJVMProperties()

    guard let pathSelector = properties["idea.paths.selector"] as? String else {
      throw InstallationStatus.failed(error: "Could not read 'idea.paths.selector' from JVMOptions.properties")
    }

    return pluginsPath(for: pathSelector)
  }

  fileprivate var _id: String?
  fileprivate func getId() -> String {
    // See `PlatformUtils` in `com.intellij.util.PlatformUtils;`
    let defaultId = "idea"

    if let id = self._id {
      return id
    }

    guard let properties = try? getJVMProperties() else {
      return defaultId
    }

    if let id = properties["idea.platform.prefix"] as? String {
      return id
    }

    return defaultId
  }

  override var id: String {
    return _id ?? getId()
  }

  func verifyInstallation() -> InstallationStatus {
    guard self.applicationIsInstalled else {
      return .applicationNotInstalled
    }

    let pluginsPathURL: URL!
    do {
      pluginsPathURL = try self.pluginsPath()
    } catch let error as InstallationStatus {
      return error
    } catch {
      return .failed(error: "An unknown error occured determining the plugins folder")
    }

    let destinationURL = pluginsPathURL.appendingPathComponent(JetBrainsIntegration.plugin.slug,
                                                               isDirectory: true)

    guard FileManager.default.fileExists(atPath: destinationURL.path) else {
      return .failed(error: "Plugin is not installed")
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

    let pluginsPathURL: URL!
    do {
      pluginsPathURL = try self.pluginsPath()
    } catch let error as InstallationStatus {
      return error
    } catch {
      return .failed(error: "An unknown error occured determining the plugins folder")
    }

    let applicationFolder = pluginsPathURL.deletingLastPathComponent()

    guard FileManager.default.fileExists(atPath: applicationFolder.path) else {
      return .failed(error: "\(applicationFolder) does not exist.")
    }

    let destinationURL = pluginsPathURL.appendingPathComponent(JetBrainsIntegration.plugin.slug,
                                                               isDirectory: true)

    // Remove old versions of plugin
    do {
      let plugins = try FileManager.default.contentsOfDirectory(atPath: pluginsPathURL.path)

      try plugins.forEach { plugin in
        if plugin.starts(with: JetBrainsIntegration.plugin.name) {
          try FileManager.default.removeItem(atPath: plugin)
        }
      }

    } catch {
      Logger.log(message: "An error occured when removing previous version of plugin: \(error.localizedDescription)")
    }

    do {
      try FileManager.default.createDirectory(at: destinationURL, withIntermediateDirectories: true, attributes: nil)
      try FileManager.default.unzipItem(at: JetBrainsIntegration.plugin.resourceInBundle, to: destinationURL)
    } catch {
      return .failed(error: "Error unzipping plugin: \(error.localizedDescription)")
    }

    if !InputMethod.default.isInstalled {
      let status = InputMethod.default.install()
      guard status == .installed else {
        return .pending(event: .inputMethodActivation)
      }

    }

    return .installed
  }
}

extension JetBrainsIntegration: TerminalIntegration {
  func getCursorRect(in window: ExternalWindow) -> NSRect? {
    return InputMethod.getCursorRect()
  }

  func terminalIsFocused(in window: ExternalWindow) -> Bool {
    return true
  }

}
