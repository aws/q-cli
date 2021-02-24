//
//  HyperIntegration.swift
//  fig
//
//  Created by Matt Schrage on 2/8/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class HyperIntegration {
  static let settingsPath = "\(NSHomeDirectory())/.hyper.js"

  static var settings: [String: Any]? {
    guard let settings = try? String(contentsOfFile: settingsPath),
    let json = settings.jsonStringToDict() else {
      return nil
    }
    
    return json
  }
  
  // If the extension path changes make sure to update the uninstall script!
  static let pluginPath: URL = URL(fileURLWithPath:"\(NSHomeDirectory())/.hyper_plugins/local/fig-hyper-integration/index.js")

  static func install(withRestart:Bool = false, completion: (() -> Void)? = nil) {
    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: Integrations.Hyper) else {
        print("Hyper not installed")
        return
    }
    
    print(url)
    
    var updatedSettings: String!
    
    guard let settings = try? String(contentsOfFile: settingsPath) else {
      print("Could not read Hyper settings")
      return
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
      print("User will need to update the file manually")
    }
    
    do {
      try FileManager.default.createDirectory(atPath: pluginPath.deletingLastPathComponent().path, withIntermediateDirectories: true)
      try FileManager.default.createSymbolicLink(at: pluginPath, withDestinationURL: Bundle.main.url(forResource: "hyper-integration", withExtension: "js")!)
    } catch {
      print("Could not add Hyper plugin")
    }
    
    try? updatedSettings.write(toFile: settingsPath, atomically: true, encoding: .utf8)
    
    if (withRestart) {
      let Hyper = Restarter(with: Integrations.Hyper)
      Hyper.restart(launchingIfInactive: false, completion: completion)
    } else {
      completion?()
    }
  }
  
  static var isInstalled: Bool {
    guard let settings = try? String(contentsOfFile: settingsPath) else {
        return false
    }
    
    return FileManager.default.fileExists(atPath: pluginPath.path) && settings.contains("fig-hyper-integration")
  }
    
  static func promptToInstall(completion: (()->Void)? = nil) {
    guard Defaults.loggedIn else {
      completion?()
      return
    }
    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: Integrations.Hyper) else {
      // application not installed
      completion?()
      return
    }
    
    let icon = NSImage(imageLiteralResourceName: "NSSecurity")

    let app = NSWorkspace.shared.icon(forFile: url.path)
    
    let alert = NSAlert()
    alert.icon = icon.overlayImage(app)
    alert.messageText = "Install Hyper Integration?"
    alert.informativeText = "Fig will add a plugin to Hyper that tracks which terminal session is active.\n\n"
    alert.alertStyle = .warning
    let button = alert.addButton(withTitle: "Install Plugin")
    button.highlight(true)
    alert.addButton(withTitle: "Not now")
  
    
    let install = alert.runModal() == .alertFirstButtonReturn
    
    if (install) {
      HyperIntegration.install(withRestart: true) {
        print("Installation completed!")
        if let app = AXWindowServer.shared.topApplication, Integrations.Hyper == app.bundleIdentifier {
          Accessibility.triggerScreenReaderModeInChromiumApplication(app)
        }
        completion?()
      }
    } else {
      completion?()
    }
  }
  
}
