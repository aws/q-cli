//
//  VSCodeIntegration.swift
//  fig
//
//  Created by Matt Schrage on 2/4/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class VSCodeIntegration {
  static let settingsPath = "\(NSHomeDirectory())/Library/Application Support/Code/User/settings.json"

  static var settings: [String: Any]? {
    guard let settings = try? String(contentsOfFile: settingsPath),
    let json = settings.jsonStringToDict() else {
      return nil
    }
    
    return json
  }
  
  static let inheritEnvKey = "terminal.integrated.inheritEnv"
  // If the extension path changes make sure to update the uninstall script!
  static let extensionPath = "\(NSHomeDirectory())/.vscode/extensions/withfig.fig-0.0.1/extension.js"

  static func install(withRestart:Bool = true, completion: (() -> Void)? = nil) {
    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: "com.microsoft.VSCode") else {
        print("VSCode not installed")
        return
    }
    
    print(url)
    
    var updatedSettings: String!
    if var json = VSCodeIntegration.settings {
      
      json[inheritEnvKey] = false
      guard let data = try? JSONSerialization.data(withJSONObject: json, options: .prettyPrinted) else {
        print("Could not serialize VSCode settings")
        return
      }
      updatedSettings = String(decoding: data, as: UTF8.self)
      
    } else {
      updatedSettings =
      """
      {
        "\(inheritEnvKey)": false
      }
      """
      
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
    guard let settings = VSCodeIntegration.settings,
      let inheritEnv = settings[inheritEnvKey] as? Bool else {
        return false
    }
    
    return FileManager.default.fileExists(atPath: extensionPath) && inheritEnv == false
  }
    
  static func promptToInstall(completion: (()->Void)? = nil) {
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
        completion?()
      }
    } else {
      completion?()
    }
  }
  
}
