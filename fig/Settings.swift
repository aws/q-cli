//
//  Settings.swift
//  fig
//
//  Created by Matt Schrage on 3/15/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa
import FigAPIBindings

// swiftlint:disable type_body_length
class Settings {
  static let ptyEnvKey = "pty.env"
  static let developerModeKey = "autocomplete.developerMode"
  static let developerModeNPMKey = "autocomplete.developerModeNPM"
  static let sshCommand = "ssh.commandPrefix"
  static let sshRemoteDirectoryScript = "ssh.remoteDirectoryScript"
  static let launchOnStartupKey = "app.launchOnStartup"
  static let legacyTelemetryDisabledKey = "app.disableTelemetry"
  static let telemetryDisabledKey = "telemetry.disabled"
  static let autocompleteWidth = "autocomplete.width"
  static let autocompleteHeight = "autocomplete.height"
  static let enterKeyBehavior = "autocomplete.enter"
  static let rightArrowKeyBehavior = "autocomplete.right-arrow"
  static let hyperDisabledKey = "integrations.hyper.disabled"
  static let vscodeDisabledKey = "integrations.vscode.disabled"
  static let iTermDisabledKey = "integrations.iterm.disabled"
  static let terminalDisabledKey = "integrations.terminal.disabled"
  static let additionalElectronTerminalsKey = "integrations.additionalElectronTerminals"
  static let additionalTerminalsKey = "integrations.additionalTerminals"
  static let hyperDelayKey = "integrations.hyper.delay"
  static let vscodeDelayKey = "integrations.vscode.delay"
  static let eventTapLocation = "developer.eventTapLocation"
  static let addStatusToTerminalTitle = "autocomplete.addStatusToTerminalTitle"
  static let disableAutocomplete = "autocomplete.disable"
  static let escapeKeyBehaviorKey = "autocomplete.escape"
  static let hideMenubarIcon = "app.hideMenubarIcon"
  static let debugModeKey = "developer.debugMode"
  static let onlyShowOnTabKey = "autocomplete.onlyShowOnTab"
  static let allowAlternateNavigationKeys = "autocomplete.allowAlternateNavigationKeys"
  static let disablePopoutDescriptions = "autocomplete.disablePopoutDescriptions"
  static let beta = "app.beta"
  static let shellIntegrationIsManagedByUser = "integrations.shell.managedByUser"
  static let theme = "autocomplete.theme"
  static let disableWebviewTransparency = "developer.disableWebviewTransparency"
  static let useControlRForHistoryBeta = "beta.history.ctrl-r"
  static let useControlRForHistory = "history.ctrl-r"
  static let shouldInterceptCommandI = "autocomplete.alwaysInterceptCommandI"
  static let inputMethodShouldPollForActivation = "integrations.input-method.shouldPollForActivation"
  static let ptyTranscript = "developer.pty.transcript"
  static let autocompleteURL = "developer.autocomplete.host"
  static let settingsURL = "developer.settings.host"
  static let missionControlURL = "developer.mission-control.host"
  static let experimentalIntegrations = "integrations.experimental"

  static let keyAliases = [
    "super": "command",
    "cmd": "command",
    "alt": "option",
    "opt": "option",
    "ctrl": "control",
    "shft": "shift",
    "return": "enter"
  ]

  static let filePath = NSHomeDirectory() + "/.fig/settings.json"
  static let defaultSettingsPath = Bundle
    .main.configURL
    .appendingPathComponent("tools", isDirectory: true)
    .appendingPathComponent("all-settings.json").path

  static let shared = Settings()
  // Note: app will crash if anything is logged before Settings.shared is initted
  static var canLogWithoutCrash = false

  // Unmodified settings read from/written to disk + updated by user.
  fileprivate var rawSettings: [String: Any]
  fileprivate var currentSettings: [String: Any]

  // Default settings, read from bundled all-settings.json list
  fileprivate var defaultSettings: [String: Any]

  // Mapping from standardized key strings like control+r to app actions,
  // e.g. { "control+r": {"autocomplete": "toggleHistory"} }
  fileprivate var keybindings: [String: [String: String]]

  func keys() -> [String] {
    return Array(currentSettings.keys)
  }

  func jsonRepresentation(ofDefaultSettings: Bool = false) -> String? {
    guard let data = try? JSONSerialization.data(
      withJSONObject: ofDefaultSettings ? defaultSettings : currentSettings,
      options: .prettyPrinted
    ) else {
      return nil
    }

    return String(decoding: data, as: UTF8.self)

  }

  static func log(_ message: String) {
    Logger.log(message: message, subsystem: .settings)
  }

  init() {
    defaultSettings = [:]
    rawSettings = [:]
    currentSettings = [:]
    keybindings = [:]

    if let settings = Settings.loadDefaultSettings() {
      defaultSettings = settings
    } else {
      print("Settings: could not load default settings!")
    }

    // load contents of file into memory
    if let settings = Settings.loadFromFile() {
      rawSettings = settings
    } else {
      print("Settings: could not load settings!")
      serialize()
    }

    recomputeSettingsFromRaw()
  }

  @objc func openUI() {
    MissionControl.openUI(.settings)
  }

  func update(_ keyValues: [String: Any]) {
    let prev = rawSettings
    rawSettings.merge(keyValues) { $1 }
    processDiffs(prev: prev, curr: rawSettings)
    recomputeSettingsFromRaw()
    serialize()
  }

  func set(value: Any?, forKey key: String) {
    let prev = rawSettings

    if let value = value {
      updateKey(key: key, value: value)
    } else {
      rawSettings.removeValue(forKey: key)
      if Settings.getKeybinding(settingKey: key) != nil {
        // If keybinding is removed we need to recompute everything to determine new binding.
        recomputeSettingsFromRaw()
      }
    }

    processDiffs(prev: prev, curr: rawSettings)
    serialize()
  }

  func getValue(forKey key: String) -> Any? {
    return rawSettings[key] ?? currentSettings[key] ?? defaultSettings[key]
  }

  func getKeybindings(forKey key: String) -> [String: String]? {
    return keybindings[key]
  }

  fileprivate func serialize() {
    do {
      let data = try JSONSerialization.data(withJSONObject: rawSettings, options: [.prettyPrinted, .sortedKeys])
      try data.write(to: URL(fileURLWithPath: Settings.filePath), options: .atomic)
    } catch {
      Settings.log("failed to serialize data")
    }
  }

  static func loadFileString(path: String) -> String? {
    guard FileManager.default.fileExists(atPath: path) else {
      Settings.log("file \(path) does not exist")
      return nil
    }

    guard let settings = try? String(contentsOfFile: path) else {
      Settings.log("file \(path) is empty")
      return nil
    }

    guard settings.count > 0 else {
      return nil
    }

    return settings
  }

  static func loadDefaultSettings() -> [String: Any]? {
    guard let fileString = loadFileString(path: Settings.defaultSettingsPath) else {
      return nil
    }

    if let jsonStream = fileString.data(using: .utf8) {
      do {
        guard let defaultSettings = try JSONSerialization.jsonObject(with: jsonStream,
                                                                     options: .fragmentsAllowed)
                                                                     as? [[String: Any]] else {
          return nil
        }
        // swiftlint:disable force_cast
        let keys = defaultSettings.map { $0["settingName"] as! String }
        let values = defaultSettings.map { $0["default"] as Any }
        return Dictionary(uniqueKeysWithValues: zip(keys, values))
      } catch {
        print(error.localizedDescription)
      }
    }
    return nil
  }

  func updateKey(key: String, value: Any) {
    rawSettings[key] = value

    if let (app, keyString) = Settings.getKeybinding(settingKey: key) {
      let standardizedKey = Settings.standardizeKeyString(keyString: keyString)
      let prefix = app == "global" ? "" : "\(app)."
      currentSettings["\(prefix)keybindings.\(standardizedKey)"] = value
      keybindings[standardizedKey, default: [:]][app] = value as? String
    } else {
      currentSettings[key] = value
    }
  }

  func recomputeSettingsFromRaw() {
    currentSettings = [:]
    keybindings = [:]

    for (setting, value) in defaultSettings {
      if let (app, keyString) = Settings.getKeybinding(settingKey: setting) {
        let key = Settings.standardizeKeyString(keyString: keyString)
        keybindings[key, default: [:]][app] = value as? String
      }
    }

    for (key, value) in rawSettings {
      updateKey(key: key, value: value)
    }
  }

  static func loadFromFile() -> [String: Any]? {
    guard let fileString = loadFileString(path: Settings.filePath) else {
      return nil
    }

    guard let settings = fileString.parseAsJSON() else {
      return nil
    }

    return settings
  }

  static func standardizeKeyString(keyString: String) -> String {
    let keys = keyString.components(separatedBy: "+").map { keyAliases[$0] ?? $0 }
    var standardKeys = keys.prefix(keys.count - 1).sorted { $0 < $1 }
    standardKeys.append(keys[keys.count - 1])
    return standardKeys.joined(separator: "+")
  }

  static func getKeybinding(settingKey: String) -> (String, String)? {
    // From a setting string like autocomplete.keybindings.control+r extract tuple of (app, keyString) if
    // setting is a keybinding.
    let components = settingKey.components(separatedBy: ".")
    if components.count > 2, components[1] == "keybindings" {
      let keyString = Settings.standardizeKeyString(keyString: components[2...].joined(separator: "."))
      return (components[0], keyString)
    } else if components.count > 1, components[0] == "keybindings" {
      let keyString = Settings.standardizeKeyString(keyString: components[1...].joined(separator: "."))
      return ("global", keyString)
    }
    return nil
  }

  func restartListener() {

    if let settings = Settings.loadDefaultSettings() {
      defaultSettings = settings
    } else {
      print("Settings: could not load default settings!")
    }

    self.settingsUpdated()
  }

  static var haveValidFormat: Bool {
    return Settings.loadFromFile() != nil
  }

  fileprivate func processSettingsUpdatesToLegacyDefaults() {
    if let disabled = currentSettings[Settings.disableAutocomplete] as? Bool {
      Defaults.shared.useAutocomplete = !disabled
    }

    if let debugMode = currentSettings[Settings.debugModeKey] as? Bool {
      Defaults.shared.debugAutocomplete = debugMode
    }
  }

  fileprivate func processDiffs(prev: [String: Any], curr: [String: Any]) {
    let priorTelemetryStatus = prev[Settings.legacyTelemetryDisabledKey] as? Bool ??
      prev[Settings.telemetryDisabledKey] as? Bool ?? false
    let currentTelemetryStatus = curr[Settings.legacyTelemetryDisabledKey] as? Bool ??
      curr[Settings.telemetryDisabledKey] as? Bool ?? false
    if priorTelemetryStatus != currentTelemetryStatus {
      TelemetryProvider.shared.identify(with:
                                  ["telemetry": currentTelemetryStatus ? "off" : "on"],
                                 shouldIgnoreTelemetryPreferences: true)
    }

    let priorLaunchAtLoginPreference = prev[Settings.launchOnStartupKey] as? Bool ?? true
    let currentLaunchAtLoginPreference = curr[Settings.launchOnStartupKey] as? Bool ?? true

    if priorLaunchAtLoginPreference != currentLaunchAtLoginPreference {

      if currentLaunchAtLoginPreference {
        LaunchAgent.launchOnStartup.addIfNotPresent()
      } else {
        LaunchAgent.launchOnStartup.remove()
      }
    }

    if prev[Settings.autocompleteURL] as? String != curr[Settings.autocompleteURL] as? String {
      WindowManager.shared.autocomplete?.webView?.loadAutocomplete(from:
                                                                    curr[Settings.autocompleteURL] as? String)
    }

  }

  static let settingsUpdatedNotification = Notification.Name("settingsUpdated")
  func settingsUpdated() {
    if let settings = Settings.loadFromFile() {
      processDiffs(prev: rawSettings, curr: settings)
      rawSettings = settings
      recomputeSettingsFromRaw()
      processSettingsUpdatesToLegacyDefaults()
      NotificationCenter.default.post(Notification(name: Settings.settingsUpdatedNotification))
    } else {

      // Don't show prompt if file is deleted, mainly because it is confusing in the uninstall flow
      guard FileManager.default.fileExists(atPath: Settings.filePath) else { return }
      DispatchQueue.main.async {
        _ = Alert.show(title: "Fig's settings can not be parsed.",
                       message: "An error occured while reading the Fig settings file stored at"
                              + "~/.fig/settings.json\n\nPlease make sure this file is valid JSON.",
                       icon: Alert.appIcon)

      }
    }
  }
}

extension Settings {
  func handleGetRequest(_ request: Fig_GetSettingsPropertyRequest) throws -> Fig_GetSettingsPropertyResponse {
    let value: Any = try {
      if request.hasKey {
        if let value = Settings.shared.getValue(forKey: request.key) {
          return value
        } else {
          throw APIError.generic(message: "No value for key")
        }
      } else {
        return Settings.shared.currentSettings
      }

    }()

    guard let data = try? JSONSerialization.data(withJSONObject: value,
                                                 options: [ .prettyPrinted,
                                                            .fragmentsAllowed]) else {
      throw APIError.generic(message: "Could not convert value for key to JSON")
    }

    return Fig_GetSettingsPropertyResponse.with {
      $0.jsonBlob = String(decoding: data, as: UTF8.self)
    }
  }

  func handleSetRequest(_ request: Fig_UpdateSettingsPropertyRequest) throws -> Bool {
    guard request.hasKey else {
      throw APIError.generic(message: "No key provided with request")
    }

    let value: Any? = {
      let valueString = request.hasValue ? request.value : nil
      guard let valueData = valueString?.data(using: .utf8) else {
        return nil
      }

      let value = try? JSONSerialization.jsonObject(with: valueData, options: .allowFragments)

      if value is NSNull {
        return nil
      }

      return value
    }()

    Settings.shared.set(value: value, forKey: request.key)

    return true

  }
}
