//
//  UpdateService.swift
//  fig
//
//  Created by Matt Schrage on 6/30/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import Sparkle

class UpdateService: NSObject {
  static let appcastURL = URL(string: "https://versions.withfig.com/appcast.xml")!
  static let betaAppcastURL = URL(string: "https://beta.withfig.com/appcast.xml")!

  static let provider = UpdateService(sparkle: SUUpdater.shared())
  static func log(_ message: String) {
    Logger.log(message: message, subsystem: .updater)
  }
  fileprivate let sparkle: SUUpdater
  init(sparkle: SUUpdater) {
    
    self.sparkle = sparkle
    super.init()
    
    self.sparkle.delegate = self

    // https://github.com/sparkle-project/Sparkle/issues/1047
    UserDefaults.standard.set(true, forKey: "SUAtomaticallyUpdate")
    self.sparkle.automaticallyDownloadsUpdates = true
    self.sparkle.automaticallyChecksForUpdates = true
    
    self.sparkle.checkForUpdateInformation()
    
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(settingsDidChange),
                                           name: Settings.settingsUpdatedNotification,
                                           object: nil)
  

  }
  
  @objc func settingsDidChange() {
    if let beta = Settings.shared.getValue(forKey: Settings.beta) as? Bool {
      let current = self.sparkle.feedURL
      let update = beta ? UpdateService.betaAppcastURL : UpdateService.appcastURL
      
      if current != update {
        self.sparkle.feedURL = update
        TelemetryProvider.identify(with: ["beta" : (beta ? "true" : "false")])
        self.sparkle.checkForUpdates(nil)
      }
    }
  }
  
  // Show updates with UI
  func checkForUpdates(_ sender: Any!) {
    UpdateService.log("checking for updates")
    self.sparkle.checkForUpdates(sender)
  }
  
  func installUpdatesIfAvailable() {
    self.sparkle.installUpdatesIfAvailable()
  }
  
  var updateIsAvailable: Bool {
    return self.update != nil
  }
  
  var updateVersion: String? {
    guard let update = self.update else {
      return nil
    }
    
    return update.displayVersionString
  }
  
  var updateBuild: String? {
    guard let update = self.update else {
      return nil
    }
    
    return update.versionString
  }
  
  var updatePublishedOn: String? {
    guard let update = self.update else {
      return nil
    }
    
    return update.dateString
  }
  
  
  fileprivate var update: SUAppcastItem? {
    didSet {
      // Update autocomplete webview
      self.notifyAutocompleteOfUpdateStatus()
      // update config file
      self.notifyShellOfUpdateStatus()
    }
  }
  
  fileprivate func notifyAutocompleteOfUpdateStatus(withNotification: Bool = true) {
    DispatchQueue.main.async {
      Autocomplete.runJavascript("fig.updateAvailable = \(self.updateIsAvailable)")

      if self.updateIsAvailable {
        Autocomplete.runJavascript(
          """
          fig.updateMetadate =
          {
            version: "\(self.updateVersion ?? "")",
            build: "\(self.updateBuild ?? "")",
            published: "\(self.updatePublishedOn ?? "")"
          }
          """)
      } else {
        Autocomplete.runJavascript(
          """
          fig.updateMetadate = null
          """)
      }
      
      // TODO: Replace with dispatchEvent API - mschrage
      if withNotification {
        Autocomplete.runJavascript("fig.updateAvailabilityChanged()")

      }
    }
  }
  
  fileprivate let userConfigPath: URL = URL(fileURLWithPath: "\(NSHomeDirectory())/.fig/user/config")
  fileprivate let configUpdateAvailableKey = "NEW_VERSION_AVAILABLE"
  fileprivate func notifyShellOfUpdateStatus() {
    if let config = try? String(contentsOf: userConfigPath, encoding: .utf8) {
      var lines = config.split(separator: "\n").filter { !$0.trimmingCharacters(in: .whitespaces).starts(with: configUpdateAvailableKey) }
      
      if self.updateIsAvailable {
        lines.append("\(configUpdateAvailableKey)=\(self.updateVersion ?? "???")")
      }
      
      let newConfig = lines.joined(separator: "\n")
      
      do {
        try newConfig.write(to: userConfigPath,
                        atomically: true,
                        encoding: .utf8)
      } catch {
        UpdateService.log("could not write updated config file")
      }

      
      
    } else {
      UpdateService.log("could not read config file at \(userConfigPath.path)")
    }
    
  }

  func installUpdateIfAvailible() {
    if self.updateIsAvailable {
      // This updates the status in the shell config and js
      self.update = nil
      // since the update is already downloaded, restarting the app should apply it.
      NSApp.appDelegate.restart()
    }
  }
  
}

extension UpdateService: SUUpdaterDelegate {

  func updater(_ updater: SUUpdater, didDownloadUpdate item: SUAppcastItem) {
    UpdateService.log("did download update (\(item.displayVersionString ?? "?" ))")
  }
  
  func updaterDidNotFindUpdate(_ updater: SUUpdater) {
    self.update = nil
    UpdateService.log("did not find update")
  }
  
  func updater(_ updater: SUUpdater, willInstallUpdate item: SUAppcastItem) {
    UpdateService.log("will install update")
  }
  
  func updater(_ updater: SUUpdater, didFinishLoading appcast: SUAppcast) {
    UpdateService.log("loaded appcast.xml from \(updater.feedURL.absoluteString)")
  }
  
  func updater(_ updater: SUUpdater, didFindValidUpdate item: SUAppcastItem) {
    self.update = item
    UpdateService.log("found valid update (\(self.updateVersion ?? "?" ))")
  }
}
