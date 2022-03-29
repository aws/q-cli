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
  static let updateAvailableNotification = Notification.Name("updateAvailableNotification")
  static let appcastURL = URL(string: "https://versions.withfig.com/appcast.xml")!
  static let betaAppcastURL = URL(string: "https://beta.withfig.com/appcast.xml")!

  static let provider = UpdateService(sparkle: SUUpdater.shared())
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
        TelemetryProvider.shared.identify(with: ["beta": (beta ? "true" : "false")])
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
    (self.sparkle as SparkleDeprecatedAPI).installUpdatesIfAvailable()
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
      // update config file
      self.notifyShellOfUpdateStatus()

      if self.update != nil {
        NotificationCenter.default.post(name: UpdateService.updateAvailableNotification,
                                        object: [
                                          "build": self.updateBuild,
                                          "version": self.updateVersion,
                                          "published": self.updatePublishedOn
                                        ])
      }
    }
  }

  fileprivate let configUpdateAvailableKey = "NEW_VERSION_AVAILABLE"
  fileprivate func notifyShellOfUpdateStatus() {
    let value = self.updateIsAvailable ? self.updateVersion ?? "???" : nil
    LocalState.shared.set(value: value, forKey: configUpdateAvailableKey)
  }

  func resetShellConfig() {
    LocalState.shared.set(value: nil, forKey: configUpdateAvailableKey)
  }

  func installUpdateIfAvailible() {
    if self.updateIsAvailable {
      // This updates the status in the shell config and js
      self.update = nil
      // since the update is already downloaded, quitting the app will apply it.
      // todo(mschrage): restart app automatically. (currently it must be relaunched manually or using `fig launch`
      NSApp.terminate(nil)

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

  func updaterWillRelaunchApplication(_ updater: SUUpdater) {
    UpdateService.log("will relaunch application")
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

  func versionComparator(for updater: SUUpdater) -> SUVersionComparison? {
    #if DEBUG
    return DebugVersionComparison()
    #else
    return nil
    #endif
  }

}

private protocol SparkleDeprecatedAPI {
  func installUpdatesIfAvailable()
}
extension SUUpdater: SparkleDeprecatedAPI {}

class DebugVersionComparison: SUVersionComparison {
  func compareVersion(_ versionA: String, toVersion versionB: String) -> ComparisonResult {
    return .orderedSame
  }
}

class ForceUpdateComparison: SUVersionComparison {
  func compareVersion(_ versionA: String, toVersion versionB: String) -> ComparisonResult {
    return .orderedAscending
  }
}

extension UpdateService: Logging {
  static func log(_ message: String) {
    Logger.log(message: message, subsystem: .updater)
  }
}

import FigAPIBindings
extension UpdateService {
  func applicationUpdateStatusRequest(_ request: Fig_ApplicationUpdateStatusRequest)
    throws -> Fig_ApplicationUpdateStatusResponse {
    guard self.updateIsAvailable else {
      return Fig_ApplicationUpdateStatusResponse.with { response in
        response.available = false
      }
    }

    return Fig_ApplicationUpdateStatusResponse.with { status in
      status.available = true

      if let build = self.updateBuild {
        status.build = build
      }

      if let version = self.updateVersion {
        status.version = version
      }

      if let published = self.updatePublishedOn {
        status.published = published
      }
    }
  }
}
