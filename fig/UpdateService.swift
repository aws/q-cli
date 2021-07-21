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
    UserDefaults.standard.set(true, forKey: "SUAutomaticallyUpdate")
    UserDefaults.standard.synchronize()
    self.sparkle.automaticallyDownloadsUpdates = true
    self.sparkle.automaticallyChecksForUpdates = true
    self.sparkle.checkForUpdatesInBackground()
    
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
  
  fileprivate let configUpdateAvailableKey = "NEW_VERSION_AVAILABLE"
  fileprivate func notifyShellOfUpdateStatus() {
    let value = self.updateIsAvailable ? self.updateVersion ?? "???" : nil
    Config.set(value: value, forKey: configUpdateAvailableKey)
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
  
  func updater(_ updater: SUUpdater, willDownloadUpdate item: SUAppcastItem, with request: NSMutableURLRequest) {
    UpdateService.log("will download update (\(item.displayVersionString ?? "?" ))")

  }
  
  func updater(_ updater: SUUpdater, failedToDownloadUpdate item: SUAppcastItem, error: Error) {
    UpdateService.log("failed to download update: \(error.localizedDescription)")
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
    UpdateService.log("found valid update (\(item.displayVersionString ?? "?" ))")
    self.sparkle.checkForUpdatesInBackground()

  }
  
  func updater(_ updater: SUUpdater, didAbortWithError error: Error) {
    UpdateService.log("did abort with error: \(error.localizedDescription)")
  }
  
  func updater(_ updater: SUUpdater, willExtractUpdate item: SUAppcastItem) {
    UpdateService.log("will extract update (\(item.displayVersionString ?? "?" ))")
  }
      
  func updater(_ updater: SUUpdater,
               willInstallUpdateOnQuit item: SUAppcastItem,
               immediateInstallationBlock installationBlock: @escaping () -> Void) {
    self.update = item
    UpdateService.log("ready to apply update (\(self.updateVersion ?? "?" ))")

  }
  
}
