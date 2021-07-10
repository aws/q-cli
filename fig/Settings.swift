//
//  Settings.swift
//  fig
//
//  Created by Matt Schrage on 3/15/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa

class Settings {
  static let ptyInitFile = "pty.rc"
  static let ptyPathKey = "pty.path"
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
  static let logging = "developer.logging"
  static let loggingEnabledInternally = "developer.logging.internal"
  static let colorfulLogging = "developer.logging.color"
  static let beta = "app.beta"
  static let shellIntegrationIsManagedByUser = "integrations.shell.managedByUser"


  static let filePath = NSHomeDirectory() + "/.fig/settings.json"
  static let shared = Settings()
  //Note: app will crash if anything is logged before Settings.shared is initted
  static var canLogWithoutCrash = false
  fileprivate var currentSettings: [String: Any]
  
  func keys() -> [String] {
    return Array(currentSettings.keys)
  }
  
  func jsonRepresentation() -> String? {
    guard let data = try? JSONSerialization.data(withJSONObject: currentSettings, options: .prettyPrinted) else {
      return nil
    }
    
    return String(decoding: data, as: UTF8.self)

  }
  
  static func log(_ message: String) {
    guard canLogWithoutCrash else {
      print("Unable to log follow message since Settings.shared hasn't been inited yet.")
      print(message)
      return
    }
    Logger.log(message: message, subsystem: .settings)
  }
  
  init() {
    
    // load contents of file into memory
    if let settings = Settings.loadFromFile() {
      currentSettings = settings
    } else {
      print("Settings: could not load settings!")
      currentSettings = [:]
      serialize()
    }
    
    setUpFileSystemListeners()
    Settings.canLogWithoutCrash = true
  }
  
  fileprivate var settingsWindow: WebViewWindow?
  @objc class func openUI() {
    Settings.log("Open Settings UI")
    
    if let settingsWindow = Settings.shared.settingsWindow {
      
      if (settingsWindow.contentViewController != nil) {
        settingsWindow.makeKeyAndOrderFront(nil)
        settingsWindow.orderFrontRegardless()
        NSApp.activate(ignoringOtherApps: true)
        
        return
      } else {
        Settings.shared.settingsWindow?.contentViewController = nil
        Settings.shared.settingsWindow = nil
      }
    }
    
    let settingsViewController = WebViewController()
    settingsViewController.webView?.defaultURL = nil
    settingsViewController.webView?.loadBundleApp("settings/index")
    settingsViewController.webView?.dragShouldRepositionWindow = true

    let settings = WebViewWindow(viewController: settingsViewController, shouldQuitAppOnClose: false)
    settings.setFrame(NSRect(x: 0, y: 0, width: 670, height: 420), display: true, animate: false)
    settings.center()
    settings.makeKeyAndOrderFront(self)
    
    settings.delegate = settings
    settings.isReleasedWhenClosed = false
    settings.level = .normal
    
    Settings.shared.settingsWindow = settings
  }
  
  func update(_ keyValues: Dictionary<String, Any>) {
    let prev = currentSettings
    currentSettings.merge(keyValues) { $1 }
    processDiffs(prev: prev, curr: currentSettings)
    serialize()
  }
  
  func set(value: Any, forKey key: String) {
    let prev = currentSettings
    currentSettings.updateValue(value, forKey: key)
    processDiffs(prev: prev, curr: currentSettings)
    serialize()
  }
  
  func getValue(forKey key: String) -> Any? {
    return currentSettings[key]
  }
  
  fileprivate func serialize() {
    do {
      let data = try JSONSerialization.data(withJSONObject: currentSettings, options: [.prettyPrinted, .sortedKeys])
      try data.write(to: URL(fileURLWithPath: Settings.filePath), options: .atomic)
    } catch {
      Settings.log("failed to serialize data")
    }
  }
  
  static func loadFromFile() ->  [String: Any]? {
    guard FileManager.default.fileExists(atPath: Settings.filePath) else {
      Settings.log("settings file does not exist")
      return nil
    }
    
    guard let settings = try? String(contentsOfFile: Settings.filePath) else {
      Settings.log("settings file is empty")
      return nil
    }
    
    guard settings.count > 0 else {
      return nil
    }
    
    guard let json = settings.jsonStringToDict() else {
      return nil
    }
    
    return json
  }

  func restartListener() {
    self.eventSource?.cancel()
    self.setUpFileSystemListeners()
    self.settingsUpdated()
  }
  
  static var haveValidFormat: Bool {
    return Settings.loadFromFile() != nil
  }
  
  fileprivate func processSettingsUpdatesToLegacyDefaults() {
    if let disabled = currentSettings[Settings.disableAutocomplete] as? Bool {
      Defaults.useAutocomplete = !disabled
    }
    
    if let debugMode = currentSettings[Settings.debugModeKey] as? Bool {
      Defaults.debugAutocomplete = debugMode
    }
  }
  
  fileprivate func processDiffs(prev: [String: Any], curr: [String: Any]) {
    let priorTelemetryStatus = prev[Settings.legacyTelemetryDisabledKey] as? Bool ??
                               prev[Settings.telemetryDisabledKey] as? Bool ?? false
    let currentTelemetryStatus = curr[Settings.legacyTelemetryDisabledKey] as? Bool ??
                                 curr[Settings.telemetryDisabledKey] as? Bool ?? false
    if priorTelemetryStatus != currentTelemetryStatus {
      TelemetryProvider.identify(with:
                                  ["telemetry": currentTelemetryStatus ? "off" : "on"],
                                 shouldIgnoreTelemetryPreferences: true)
    }
  }
  
  static let settingsUpdatedNotification = Notification.Name("settingsUpdated")
  func settingsUpdated() {
    if let settings = Settings.loadFromFile() {
       processDiffs(prev: currentSettings, curr: settings)
       currentSettings = settings
       processSettingsUpdatesToLegacyDefaults()
       NotificationCenter.default.post(Notification(name: Settings.settingsUpdatedNotification))
    } else {
      
      // Don't show prompt if file is deleted, mainly because it is confusing in the uninstall flow
      guard FileManager.default.fileExists(atPath: Settings.filePath) else { return }
      DispatchQueue.main.async {
          let _ = Alert.show(title: "Fig's settings can not be parsed.",
                             message: "An error occured while reading the Fig settings file stored at ~/.fig/settings.json\n\nPlease make sure this file is valid JSON.",
                             icon: Alert.appIcon)

      }
    }
  }
  
  var eventSource: DispatchSourceFileSystemObject?
  fileprivate func setUpFileSystemListeners() {
    // set up file observers
    guard FileManager.default.fileExists(atPath: Settings.filePath) else {
      Settings.log("file does not exist. Not setting up listeners")
      return
    }

    let descriptor = open(Settings.filePath, O_EVTONLY)
    if descriptor == -1 {
      return
    }

    self.eventSource = DispatchSource.makeFileSystemObjectSource(fileDescriptor: descriptor,
                                                                 eventMask: [.all],
                                                                 queue: DispatchQueue.main)
    self.eventSource?.setEventHandler {
      [weak self] in
      Settings.log(String(describing: self?.eventSource?.dataStrings ?? []))
      if ( [.write, .attrib].contains(self?.eventSource?.data) ) {
        self?.settingsUpdated()
      }
      
      
      if (self?.eventSource?.data.contains(.delete) ?? false) {
        self?.eventSource?.cancel()
        self?.setUpFileSystemListeners()
        self?.settingsUpdated()
      }
    }
    
    self.eventSource?.setCancelHandler() {
      close(descriptor)
    }
    
    self.eventSource?.resume()

  }
  
  deinit {
    self.eventSource?.cancel()
  }
  
}

extension DispatchSourceFileSystemObject {
    var dataStrings: [String] {
        var s = [String]()
        if data.contains(.all)      { s.append("all") }
        if data.contains(.attrib)   { s.append("attrib") }
        if data.contains(.delete)   { s.append("delete") }
        if data.contains(.extend)   { s.append("extend") }
        if data.contains(.funlock)  { s.append("funlock") }
        if data.contains(.link)     { s.append("link") }
        if data.contains(.rename)   { s.append("rename") }
        if data.contains(.revoke)   { s.append("revoke") }
        if data.contains(.write)    { s.append("write") }
        return s
    }
}
