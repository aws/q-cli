//
//  VSCodeIntegration.swift
//  fig
//
//  Created by Matt Schrage on 2/4/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import Sentry

class VSCodeIntegration: TerminalIntegrationProvider {
  static let `default` = VSCodeIntegration(bundleIdentifier: Integrations.VSCode,
                                           configFolderName: ".vscode",
                                           applicationSupportFolderName: "Code",
                                           applicationName: "VSCode")
  static let insiders = VSCodeIntegration(bundleIdentifier: Integrations.VSCodeInsiders,
                                          configFolderName: ".vscode-insiders",
                                          applicationSupportFolderName: "Code - Insiders",
                                          applicationName: "VSCode Insiders")
  static let vscodium = VSCodeIntegration(bundleIdentifier: Integrations.VSCodium,
                                          configFolderName: ".vscode-oss",
                                          applicationSupportFolderName: "VSCodium",
                                          applicationName: "VSCodium",
                                          cliExecutableName: "codium")

  static let supportURL: URL = URL(string: "https://fig.io/docs/support/vscode-integration")!

  static let inheritEnvKey = "terminal.integrated.inheritEnv"
  static let accessibilitySupportKey = "editor.accessibilitySupport"

  static let extensionVersion = "0.0.6"

  fileprivate let configFolderName: String
  fileprivate let applicationSupportFolderName: String
  fileprivate let cliExecutableName: String

  init(bundleIdentifier: String,
       configFolderName: String,
       applicationSupportFolderName: String,
       applicationName: String,
       cliExecutableName: String = "code") {
    self.configFolderName = configFolderName
    self.applicationSupportFolderName = applicationSupportFolderName
    self.cliExecutableName = cliExecutableName
    super.init(bundleIdentifier: bundleIdentifier)

    self.applicationName = applicationName
    self.promptMessage =
      "Fig will add an extension to \(self.applicationName) that tracks which integrated terminal is" +
      " active.\n\n\(self.applicationName) will need to restart for changes to take effect.\n"
    self.promptButtonText = "Install Extension"
  }

  var settingsPath: String {
    let defaultPath =
      "\(NSHomeDirectory())/Library/Application Support/\(self.applicationSupportFolderName)/User/settings.json"
    return (try? FileManager.default.destinationOfSymbolicLink(atPath: defaultPath)) ?? defaultPath
  }

  // If the extension path changes make sure to update the uninstall script!
  var extensionsDir: String {
    return "\(NSHomeDirectory())/\(self.configFolderName)/extensions"
  }

  // If the extension path changes make sure to update the uninstall script!
  var extensionPath: String {
    return "\(self.extensionsDir)/withfig.fig-\(VSCodeIntegration.extensionVersion)/extension.js"
  }

  func uninstall() -> Bool {
    if let extensions = try? FileManager.default.contentsOfDirectory(atPath: self.extensionsDir) {
        for ext in extensions {
            if URL(fileURLWithPath: ext).lastPathComponent.hasPrefix("withfig.fig-") {
                try? FileManager.default.removeItem(atPath: ext)
            }
        }
    }
    return true
  }

  func install() -> InstallationStatus {
    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: self.bundleIdentifier) else {
      return .applicationNotInstalled
    }

    var successfullyUpdatedSettings = false
    var updatedSettings: String!
    do {
      if var json = try self.settings() {

        if let accessibilitySupport =
            json[VSCodeIntegration.accessibilitySupportKey] as? String?,
           accessibilitySupport != "on" {
          json[VSCodeIntegration.accessibilitySupportKey] = "off"
        }

        guard let data = try? JSONSerialization.data(withJSONObject: json, options: .prettyPrinted) else {
          return .failed(error: "Could not serialize VSCode settings")
        }
        updatedSettings = String(decoding: data, as: UTF8.self)

      } else {
        updatedSettings =
          """
        {
          "\(VSCodeIntegration.accessibilitySupportKey)": "off"
        }
        """
      }

      try updatedSettings.write(toFile: self.settingsPath, atomically: true, encoding: .utf8)
      successfullyUpdatedSettings = true
    } catch {
      // NSApp.appDelegate.dialogOKCancel(question: "Fig could not install the VSCode Integration",
      //                                 text: "An error occured when attempting to parse settings.json")

      Logger.log(message: "VSCode: An error occured when attempting to parse settings.json")

    }

    let cli = url.appendingPathComponent("Contents/Resources/app/bin/" + self.cliExecutableName)
    let vsix = Bundle.main.path(forResource: "fig-\(VSCodeIntegration.extensionVersion)", ofType: "vsix")!
    "\(cli.path.replacingOccurrences(of: " ", with: "\\ ")) --install-extension \(vsix)".runInBackground()

    guard successfullyUpdatedSettings else {
      // swiftlint:disable line_length
      return .failed(error:
                      "Fig could not parse VSCode's settings.json file.\nTo finish the installation, you will need to update a few preferences manually.",
                     supportURL: VSCodeIntegration.supportURL)

    }

    return .pending(event: .applicationRestart)
  }

  func verifyInstallation() -> InstallationStatus {
    guard self.applicationIsInstalled else {
      return .applicationNotInstalled
    }

    guard FileManager.default.fileExists(atPath: self.extensionPath) else {
      return .failed(error: "Extension is not installed.")
    }

    return .installed
  }

}

extension VSCodeIntegration {

  enum InstallationError: Error {
    case couldNotParseSettingsJSON
    case couldNotReadContentsOfSettingsFile
  }

  func settings() throws -> [String: Any]? {
    guard FileManager.default.fileExists(atPath: self.settingsPath) else {
      // file does not exist
      Logger.log(message: "VSCode: settings file does not exist")

      return nil
    }

    guard let settings = try? String(contentsOfFile: self.settingsPath) else {
      Logger.log(message: "VSCode: settings file is empty or could not be read")

      throw InstallationError.couldNotReadContentsOfSettingsFile
    }

    guard settings.count > 0 else {
      return nil
    }

    guard let json = settings.parseAsJSON() else {
      throw InstallationError.couldNotParseSettingsJSON
    }

    return json
  }

}

extension VSCodeIntegration {
  func getCursorRect(in window: ExternalWindow) -> NSRect? {
    return Accessibility.findXTermCursorInElectronWindow(window)
  }

  func terminalIsFocused(in window: ExternalWindow) -> Bool {
    return Accessibility.findXTermCursorInElectronWindow(window) != nil
  }

}
