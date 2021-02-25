//
//  VSCodeIntegration.swift
//  fig
//
//  Created by Matt Schrage on 2/4/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import Sentry

class VSCodeIntegration {
  static var settingsPath: String {
    let defaultPath = "\(NSHomeDirectory())/Library/Application Support/Code/User/settings.json"
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
  static let extensionPath = "\(NSHomeDirectory())/.vscode/extensions/withfig.fig-0.0.1/extension.js"

  static func install(withRestart:Bool = true, completion: (() -> Void)? = nil) {
    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: "com.microsoft.VSCode") else {
        print("VSCode not installed")
        return
    }
    
    print(url)
    
    var updatedSettings: String!
    do {
      if var json = try VSCodeIntegration.settings() {
        
        json[inheritEnvKey] = false
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
          "\(inheritEnvKey)": false,
          "\(accessibilitySupportKey)": "off"
        }
        """
      }
    
    } catch {
      //NSApp.appDelegate.dialogOKCancel(question: "Fig could not install the VSCode Integration",
      //                                 text: "An error occured when attempting to parse settings.json")
      
      print("VSCode: An error occured when attempting to parse settings.json")
      SentrySDK.capture(message: "VSCode: An error occured when attempting to parse settings.json")
      
      completion?()
      return
    }
    
    try? updatedSettings.write(toFile: settingsPath, atomically: true, encoding: .utf8)

    let cli = url.appendingPathComponent("Contents/Resources/app/bin/code")
    let vsix = Bundle.main.path(forResource: "fig-0.0.1", ofType: "vsix")!
    print("\(cli.path.replacingOccurrences(of: " ", with: "\\ ")) --install-extension \(vsix)")
    "\(cli.path.replacingOccurrences(of: " ", with: "\\ ")) --install-extension \(vsix)".runInBackground { (out) in
      print(out)
      if (withRestart) {
        let VSCode = Restarter(with: "com.microsoft.VSCode")
        VSCode.restart(launchingIfInactive: false, completion: completion)
      } else {
        completion?()
      }
    }
  }
  
  static var isInstalled: Bool {
    guard let settings = try? VSCodeIntegration.settings(),
      let inheritEnv = settings[inheritEnvKey] as? Bool,
      settings[accessibilitySupportKey] != nil else {
        return false
    }
    
    return FileManager.default.fileExists(atPath: extensionPath) && inheritEnv == false
  }
    
  static func promptToInstall(completion: (()->Void)? = nil) {
    guard Defaults.loggedIn else {
      completion?()
      return
    }

    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: "com.microsoft.VSCode") else {
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
      VSCodeIntegration.install {
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
