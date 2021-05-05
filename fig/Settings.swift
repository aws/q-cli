//
//  Settings.swift
//  fig
//
//  Created by Matt Schrage on 3/15/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class Settings {
  static let ptyInitFile = "pty.rc"
  static let ptyPathKey = "pty.path"
  static let developerModeKey = "autocomplete.developerMode"
  static let developerModeNPMKey = "autocomplete.developerModeNPM"
  static let sshCommand = "ssh.commandPrefix"
  static let sshRemoteDirectoryScript = "ssh.remoteDirectoryScript"
  static let launchOnStartupKey = "app.launchOnStartup"
  static let telemetryDisabledKey = "app.disableTelemetry"
  static let autocompleteWidth = "autocomplete.width"
  static let autocompleteHeight = "autocomplete.height"
  static let enterKeyBehavior = "autocomplete.enter"
  static let hyperDisabledKey = "integrations.hyper.disabled"
  static let vscodeDisabledKey = "integrations.vscode.disabled"
  static let iTermDisabledKey = "integrations.iterm.disabled"
  static let terminalDisabledKey = "integrations.terminal.disabled"
  static let hyperDelayKey = "integrations.hyper.delay"
  static let vscodeDelayKey = "integrations.vscode.delay"
  static let eventTapLocation = "developer.eventTapLocation"
  static let addStatusToTerminalTitle = "autocomplete.addStatusToTerminalTitle"
  static let disableAutocomplete = "autocomplete.disable"
  static let escapeKeyBehaviorKey = "autocomplete.escape"
  static let hideMenubarIcon = "app.hideMenubarIcon"
  static let debugModeKey = "developer.debugMode"

  static let filePath = NSHomeDirectory() + "/.fig/settings.json"
  static let shared = Settings()
  
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
  }
  
  func update(_ keyValues: Dictionary<String, Any>) {
    currentSettings.merge(keyValues) { $1 }
    serialize()
  }
  
  func set(value: Any, forKey key: String) {
    currentSettings.updateValue(value, forKey: key)
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
      print("Settings: failed to serialize data")
    }
  }
  
  static func loadFromFile() ->  [String: Any]? {
    guard FileManager.default.fileExists(atPath: Settings.filePath) else {
      print("Settings: settings file does not exist")
      return nil
    }
    
    guard let settings = try? String(contentsOfFile: Settings.filePath) else {
      print("Settings: settings file is empty")
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
  
  static let settingsUpdatedNotification = Notification.Name("settingsUpdated")
  func settingsUpdated() {
    if let settings = Settings.loadFromFile() {
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
      print("Settings: file does not exist. Not setting up listeners")
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
      print("Settings:", self?.eventSource?.dataStrings ?? [])
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
