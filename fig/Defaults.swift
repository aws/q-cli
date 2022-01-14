//
//  Defaults.swift
//  fig
//
//  Created by Matt Schrage on 7/8/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
import Sentry
import FigAPIBindings

enum Build: String {
  case production = "prod"
  case staging = "staging"
  case dev = "dev"
}

class Defaults {
  static let shared = Defaults(UserDefaults.standard)

  private var defaults: UserDefaults!

  var isProduction: Bool {
    return build == .production
  }

  var isStaging: Bool {
    return defaults.string(forKey: "build") == "staging"
  }

  var build: Build {
    get {
      return Build(rawValue: defaults.string(forKey: "build") ?? "") ?? .production
    }
    set(value) {
      defaults.set(value.rawValue, forKey: "build")
      defaults.synchronize()
      WindowManager.shared.createAutocomplete()
      (NSApp.delegate as? AppDelegate)?.configureStatusBarItem()

    }
  }

  var uuid: String {
    guard let uuid = defaults.string(forKey: "uuid") else {
      let uuid = UUID().uuidString
      defaults.set(uuid, forKey: "uuid")
      defaults.synchronize()
      return uuid
    }

    return uuid
  }

  var showSidebar: Bool {
    get {
      return defaults.string(forKey: "sidebar") != "hidden"
    }

    set(value) {
      defaults.set(value ? "visible" : "hidden", forKey: "sidebar")
      defaults.synchronize()
    }
  }

  var email: String? {
    get {
      return defaults.string(forKey: "userEmail")
    }

    set(email) {
      let user = User()
      user.email = email
      SentrySDK.setUser(user)
      defaults.set(email, forKey: "userEmail")
      defaults.synchronize()
    }
  }

  var version: String {
    return Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "<unknown>"
  }

  var automaticallyLaunchWebAppsInDetachedWindow: Bool {
    get {
      return defaults.string(forKey: "undockWebApps") == "true"
    }

    set(flag) {
      defaults.set(flag ? "true" : "false", forKey: "undockWebApps")
      defaults.synchronize()
    }
  }

  var loggedIn: Bool {
    get {
      return UserDefaults(suiteName: "com.mschrage.fig.shared")?.bool(forKey: "loggedIn") ?? false
    }

    set(loggedIn) {
      UserDefaults(suiteName: "com.mschrage.fig.shared")?.set(loggedIn, forKey: "loggedIn")
      UserDefaults(suiteName: "com.mschrage.fig.shared")?.synchronize()
      if let delegate = NSApp.delegate as? AppDelegate {
        delegate.configureStatusBarItem()
      }
    }
  }
  var domainToken: String? {
    get {
      return defaults.string(forKey: "domainToken")
    }

    set(token) {
      defaults.set(token, forKey: "domainToken")
      defaults.synchronize()
    }
  }

  var defaultActivePosition: CompanionWindow.OverlayPositioning {
    get {

      return  defaults.bool(forKey: "updatedDefaultActivePosition")
        ? CompanionWindow.OverlayPositioning(rawValue: defaults.integer(forKey: "defaultActivePosition"))
        ?? .outsideRight
        : .outsideRight
    }

    set(id) {
      defaults.set(id.rawValue, forKey: "defaultActivePosition")
      defaults.synchronize()
    }
  }

  var shouldTrackTargetWindow: Bool {
    get {
      return
        defaults.bool(forKey: "shouldTrackTargetWindow")
    }

    set(token) {
      defaults.set(token, forKey: "shouldTrackTargetWindow")
      defaults.synchronize()
    }
  }

  var clearExistingLineOnTerminalInsert: Bool {
    get {
      return
        defaults.bool(forKey: "clearExistingLineOnTerminalInsert")
    }

    set(token) {
      defaults.set(token, forKey: "clearExistingLineOnTerminalInsert")
      defaults.synchronize()
    }
  }

  var triggerSidebarWithMouse: Bool {
    get {
      return
        defaults.bool(forKey: "triggerSidebarWithMouse")
    }

    set(token) {
      defaults.set(token, forKey: "triggerSidebarWithMouse")
      defaults.synchronize()
    }
  }

  let autocompletePreferenceUpdated = Notification.Name("autocompletePreferenceUpdated")
  fileprivate var _useAutcomplete: Bool?
  var useAutocomplete: Bool {
    get {
      if let flag = _useAutcomplete {
        return flag
      } else {
        let flag = defaults.bool(forKey: "useAutocomplete")
        _useAutcomplete = flag
        return flag
      }
    }

    set(flag) {
      guard _useAutcomplete != flag else { return }

      _useAutcomplete = flag
      NotificationCenter.default.post(name: autocompletePreferenceUpdated, object: flag)
      defaults.set(flag, forKey: "useAutocomplete")
      defaults.synchronize()

      Settings.shared.set(value: !flag, forKey: Settings.disableAutocomplete)

      NSApp.appDelegate.configureStatusBarItem()
    }

  }

  var playSoundWhenContextIsLost: Bool {
    get {
      return
        defaults.bool(forKey: "playSoundWhenContextIsLost")
    }

    set(flag) {
      defaults.set(flag, forKey: "playSoundWhenContextIsLost")
      defaults.synchronize()
    }

  }

  var versionAtPreviousLaunch: String? {
    get {
      return  defaults.string(forKey: "versionAtPreviousLaunch")
    }

    set(version) {
      defaults.set(version, forKey: "versionAtPreviousLaunch")
      defaults.synchronize()
    }
  }

  var debugAutocomplete: Bool {
    get {
      return
        defaults.bool(forKey: "debugAutocomplete")
    }

    set(flag) {
      guard debugAutocomplete != flag else {
        return
      }

      defaults.set(flag, forKey: "debugAutocomplete")
      defaults.synchronize()

      Settings.shared.set(value: flag, forKey: Settings.debugModeKey)

      WindowManager.shared.autocomplete?.backgroundColor = .clear

    }

  }

  var globalAccessibilityTimeout: Float {
    get {
      return defaults.float(forKey: "globalAccessibilityTimeout")
    }

    set(value) {
      defaults.set(value, forKey: "globalAccessibilityTimeout")
      defaults.synchronize()
    }
  }

  var broadcastLogs: Bool {
    get {
      return
        defaults.bool(forKey: "broadcastLogs")
    }

    set(flag) {
      defaults.set(flag, forKey: "broadcastLogs")
      defaults.synchronize()
    }

  }

  var broadcastLogsForSubsystem: Logger.Subsystem {
    get {
      return Logger.Subsystem(rawValue: defaults.string(forKey: "broadcastLogsForSubsystem") ?? "") ?? .global
    }

    set(subsystem) {
      defaults.set(subsystem.rawValue, forKey: "broadcastLogsForSubsystem")
      defaults.synchronize()
    }

  }

  var autocompleteVersion: String? {
    get {
      return  defaults.string(forKey: "autocompleteVersion")
    }

    set(version) {
      defaults.set(version, forKey: "autocompleteVersion")
      defaults.synchronize()
    }
  }

  var autocompleteWidth: CGFloat? {
    get {
      let string = defaults.string(forKey: "autocompleteWidth")
      guard let str = string, let n = NumberFormatter().number(from: str) else { return nil }
      return n as? CGFloat
    }

    set(width) {
      guard let width = width else { return }
      let str = NumberFormatter().string(from: NSNumber(floatLiteral: Double(width) ))
      defaults.set(str, forKey: "autocompleteWidth")
      defaults.synchronize()
    }
  }

  var processWhitelist: [String] {
    get {
      let string = defaults.string(forKey: "processWhitelist")
      return string?.split(separator: ",").map { String($0) } ?? []
    }

    set(whitelist) {
      defaults.set(whitelist.joined(separator: ","), forKey: "processWhitelist")
      defaults.synchronize()
    }

  }

  var ignoreProcessList: [String] {
    get {
      let string = defaults.string(forKey: "ignoreProcessList")
      return string?.split(separator: ",").map { String($0) } ?? []
    }

    set(whitelist) {
      defaults.set(whitelist.joined(separator: ","), forKey: "ignoreProcessList")
      defaults.synchronize()
    }

  }

  var launchedFollowingCrash: Bool {
    get {
      return
        defaults.bool(forKey: "launchedFollowingCrash")
    }

    set(flag) {
      defaults.set(flag, forKey: "launchedFollowingCrash")
      defaults.synchronize()
    }

  }

  var onlyInsertOnTab: Bool {
    get {
      if let behavior = Settings.shared.getValue(forKey: Settings.enterKeyBehavior) as? String {
        switch behavior {
        case "ignore":
          return true
        case "insert":
          return false
        default:
          return false
        }
      }

      return defaults.bool(forKey: "onlyInsertOnTab")
    }

    set(flag) {
      defaults.set(flag, forKey: "onlyInsertOnTab")
      defaults.synchronize()

      Settings.shared.set(value: flag ? "ignore" : "insert", forKey: Settings.enterKeyBehavior)
    }

  }

  // determined by running `dscl . -read ~/ UserShell`
  // output: "UserShell: /bin/zsh"
  var userShell: String {
    get {
      let shell = defaults.string(forKey: "userShell")
      return shell?.replacingOccurrences(of: "UserShell: ", with: "") ?? "/bin/sh"
    }

    set(shell) {
      var val: String?
      if shell.starts(with: "UserShell: ") {
        val = shell
      } else {
        val = "UserShell: \(shell)"
      }

      defaults.set(val!, forKey: "userShell")
      defaults.synchronize()
    }
  }

  var SSHIntegrationEnabled: Bool {
    get {
      return defaults.bool(forKey: "SSHIntegrationEnabled")
    }

    set(flag) {
      defaults.set(flag, forKey: "SSHIntegrationEnabled")
      defaults.synchronize()
    }
  }

  var promptedToRestartDueToXtermBug: Bool {
    get {
      return defaults.bool(forKey: "promptedToRestartDueToXtermBug")
    }

    set(flag) {
      defaults.set(flag, forKey: "promptedToRestartDueToXtermBug")
      defaults.synchronize()
    }
  }

  var hasShownAutocompletePopover: Bool {
    get {
      return defaults.bool(forKey: "hasShownAutocompletePopover")
    }

    set(flag) {
      defaults.set(flag, forKey: "hasShownAutocompletePopover")
      defaults.synchronize()
    }
  }

  var port: Int {
    get {
      return UserDefaults(suiteName: "com.mschrage.fig.shared")?.integer(forKey: "port") ?? 8765
    }

    set (port) {
      UserDefaults(suiteName: "com.mschrage.fig.shared")?.set(port, forKey: "port")
      UserDefaults(suiteName: "com.mschrage.fig.shared")?.synchronize()
    }

  }

  var developerModeEnabled: Bool {
    get {
      if let mode = Settings.shared.getValue(forKey: Settings.developerModeKey) as? Bool, mode {
        return mode
      }

      if let mode = Settings.shared.getValue(forKey: Settings.developerModeNPMKey) as? Bool, mode {
        return mode
      }

      return false
    }

    set (enabled) {
      var delta: [String: Any] = [:]
      if Settings.shared.getValue(forKey: Settings.developerModeKey) as? Bool != nil {
        delta[Settings.developerModeKey] = enabled
      }

      if Settings.shared.getValue(forKey: Settings.developerModeNPMKey) as? Bool != nil {
        delta[Settings.developerModeNPMKey] = enabled
      }

      Settings.shared.update(delta)
    }
  }

  @objc func toggleDeveloperMode() {
    developerModeEnabled.toggle()
  }

  var beta: Bool {
    get {
      return Settings.shared.getValue(forKey: Settings.beta) as? Bool ?? false
    }

    set (enabled) {
      Settings.shared.set(value: enabled, forKey: Settings.beta)
    }
  }

  var telemetryDisabled: Bool {
    get {

      if let mode = Settings.shared.getValue(forKey: Settings.legacyTelemetryDisabledKey) as? Bool, mode {
        return mode
      }

      if let mode = Settings.shared.getValue(forKey: Settings.telemetryDisabledKey) as? Bool, mode {
        return mode
      }

      return false
    }

    set (enabled) {
      var delta: [String: Any] = [:]
      if Settings.shared.getValue(forKey: Settings.legacyTelemetryDisabledKey) as? Bool != nil {
        delta[Settings.legacyTelemetryDisabledKey] = enabled
      }

      if Settings.shared.getValue(forKey: Settings.telemetryDisabledKey) as? Bool != nil {
        delta[Settings.telemetryDisabledKey] = enabled
      }

      Settings.shared.update(delta)
    }
  }

  var accessibilityEnabledOnPreviousLaunch: Bool? {
    get {
      return  defaults.bool(forKey: "accessibilityEnabledOnPreviousLaunch")
    }

    set(version) {
      defaults.set(version, forKey: "accessibilityEnabledOnPreviousLaunch")
      defaults.synchronize()
    }
  }

  var insertUsingRightArrow: Bool {
    get {
      if let behavior = Settings.shared.getValue(forKey: Settings.rightArrowKeyBehavior) as? String {
        switch behavior {
        case "insert":
          return true
        case "ignore":
          return false
        default:
          return false
        }
      }

      return false
    }

    set(flag) {
      Settings.shared.set(value: flag ? "insert" :  "ignore", forKey: Settings.rightArrowKeyBehavior)
    }

  }

  init(_ defaults: UserDefaults) {
    self.defaults = defaults
  }
}

extension Defaults {
  func handleGetRequest(_ request: Fig_GetDefaultsPropertyRequest) throws -> Fig_GetDefaultsPropertyResponse {
    guard request.hasKey else {
      throw APIError.generic(message: "No key provided.")
    }

    let value = defaults.object(forKey: request.key)

    var fig_value: Fig_DefaultsValue

    switch value {
    case nil:
      fig_value = Fig_DefaultsValue.with { $0.null = true }
    case let number as NSNumber:
      if number === kCFBooleanTrue || number === kCFBooleanFalse {
        fig_value = Fig_DefaultsValue.with({ $0.boolean = number.boolValue })
      } else {
        fig_value = Fig_DefaultsValue.with { $0.integer = number.int64Value }
      }
    case let string as String:
      fig_value = Fig_DefaultsValue.with { $0.string = string }
    default:
      throw APIError.generic(message: "Value is an unsupport type.")

    }

    return Fig_GetDefaultsPropertyResponse.with { response in
      response.key = request.key
      response.value = fig_value
    }

  }

  @discardableResult
  func handleSetRequest(_ request: Fig_UpdateDefaultsPropertyRequest) throws -> Bool {
    guard request.hasKey else {
      throw APIError.generic(message: "No key provided.")
    }

    guard request.hasValue && request.value.type != .null(true) else {
      defaults.removeObject(forKey: request.key)
      defaults.synchronize()
      return true
    }

    switch request.value.type {
    case .boolean(let bool):
      defaults.set(bool, forKey: request.key)
    case .integer(let integer):
      defaults.set(integer, forKey: request.key)
    case .string(let string):
      defaults.set(string, forKey: request.key)
    default:
      throw APIError.generic(message: "Value is an unsupport type.")
    }

    defaults.synchronize()
    return true
  }

}
