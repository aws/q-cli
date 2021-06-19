//
//  VSCodeIntegration.swift
//  fig
//
//  Created by Matt Schrage on 2/4/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import Sentry

// TODO: refactor this so that VSCode and VSCode Insiders share this logic
class VSCodeInsidersIntegration: IntegrationProvider {
  static let supportURL: URL = URL(string: "https://fig.io/docs/support/vscode-integration")!
  static var settingsPath: String {
    let defaultPath = "\(NSHomeDirectory())/Library/Application Support/Code - Insiders/User/settings.json"
    return (try? FileManager.default.destinationOfSymbolicLink(atPath: defaultPath)) ?? defaultPath
  }

  enum InstallationError: Error {
      case couldNotParseSettingsJSON
      case couldNotReadContentsOfSettingsFile
  }
  
  static func settings() throws -> [String: Any]? {
    guard FileManager.default.fileExists(atPath: settingsPath) else {
      // file does not exist
      print("VSCode: settings file does not exist")
      SentrySDK.capture(message: "VSCode: settings file does not exist")

      return nil
    }
    
    guard let settings = try? String(contentsOfFile: settingsPath) else {
      print("VSCode: settings file is empty")
      SentrySDK.capture(message: "VSCode: settings file is empty or could not be read")

      throw InstallationError.couldNotReadContentsOfSettingsFile
    }
    
    guard settings.count > 0 else {
      return nil
    }
    
    guard let json = settings.jsonStringToDict() else {
      throw InstallationError.couldNotParseSettingsJSON
    }
    
    return json
  }
  
  static let inheritEnvKey = "terminal.integrated.inheritEnv"
  static let accessibilitySupportKey = "editor.accessibilitySupport"

  // If the extension path changes make sure to update the uninstall script!
  static let extensionVersion = "0.0.3"
  static let extensionPath = "\(NSHomeDirectory())/.vscode-insiders/extensions/withfig.fig-\(extensionVersion)/extension.js"

  static func install(withRestart:Bool = true, inBackground: Bool, completion: (() -> Void)? = nil) {
    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: Integrations.VSCodeInsiders) else {
        print("VSCode Insiders not installed")
        return
    }
    
    print(url)
    var successfullyUpdatedSettings = false
    var updatedSettings: String!
    do {
      if var json = try VSCodeInsidersIntegration.settings() {
        
        if let accessibilitySupport = json[accessibilitySupportKey] as? String?, accessibilitySupport != "on" {
          json[accessibilitySupportKey] = "off"
        }
        
        guard let data = try? JSONSerialization.data(withJSONObject: json, options: .prettyPrinted) else {
          print("Could not serialize VSCode settings")
          return
        }
        updatedSettings = String(decoding: data, as: UTF8.self)
        
      } else {
        updatedSettings =
        """
        {
          "\(accessibilitySupportKey)": "off"
        }
        """
      }
      
      try updatedSettings.write(toFile: settingsPath, atomically: true, encoding: .utf8)
      successfullyUpdatedSettings = true
    } catch {
      //NSApp.appDelegate.dialogOKCancel(question: "Fig could not install the VSCode Integration",
      //                                 text: "An error occured when attempting to parse settings.json")
      
      print("VSCode: An error occured when attempting to parse settings.json")
      SentrySDK.capture(message: "VSCode: An error occured when attempting to parse settings.json")
      
    }
    
    let cli = url.appendingPathComponent("Contents/Resources/app/bin/code")
    let vsix = Bundle.main.path(forResource: "fig-\(extensionVersion)", ofType: "vsix")!
    print("\(cli.path.replacingOccurrences(of: " ", with: "\\ ")) --install-extension \(vsix)")
    "\(cli.path.replacingOccurrences(of: " ", with: "\\ ")) --install-extension \(vsix)".runInBackground { (out) in
      print(out)
      guard successfullyUpdatedSettings else {
        if (!inBackground) {
          DispatchQueue.main.async {
            let openSupportPage = Alert.show(title: "Could not install VSCode integration automatically",
                                       message: "Fig could not parse VSCode's settings.json file.\nTo finish the installation, you will need to update a few preferences manually.",
                                       okText: "Learn more",
                                       icon: Alert.appIcon,
                                       hasSecondaryOption: true)
            if (openSupportPage) {
              NSWorkspace.shared.open(VSCodeInsidersIntegration.supportURL)
            }
          }
        }
        
        return
      }
      
      if (withRestart) {
        let VSCode = Restarter(with: Integrations.VSCodeInsiders)
        VSCode.restart(launchingIfInactive: false, completion: completion)
      } else {
        completion?()
      }
    }
  }
  
  static var isInstalled: Bool {
//    guard let settings = try? VSCodeIntegration.settings(),
//      settings[accessibilitySupportKey] != nil else {
//        return false
//    }
    
    return FileManager.default.fileExists(atPath: extensionPath)
  }
    
  static func promptToInstall(completion: (()->Void)? = nil) {
    guard Defaults.loggedIn else {
      completion?()
      return
    }

    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: Integrations.VSCodeInsiders) else {
      // application not installed
      completion?()
      return
    }
    
    let icon = NSImage(imageLiteralResourceName: "NSSecurity")

    let app = NSWorkspace.shared.icon(forFile: url.path)
    
    let alert = NSAlert()
    alert.icon = icon.overlayImage(app)
    alert.messageText = "Install VSCode Integration?"
    alert.informativeText = "Fig will add an extension to Visual Studio Code that tracks which integrated terminal is active.\n\nVSCode will need to restart for changes to take effect.\n"
    alert.alertStyle = .warning
    let button = alert.addButton(withTitle: "Install Extension")
    button.highlight(true)
    alert.addButton(withTitle: "Not now")
  
    
    let install = alert.runModal() == .alertFirstButtonReturn
    
    if (install) {
      VSCodeInsidersIntegration.install(inBackground: false) {
        print("Installation completed!")
        if let app = AXWindowServer.shared.topApplication, Integrations.VSCode == app.bundleIdentifier {
          Accessibility.triggerScreenReaderModeInChromiumApplication(app)
        }
        completion?()
      }
    } else {
      completion?()
    }
  }
  
}
