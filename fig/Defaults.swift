//
//  Defaults.swift
//  fig
//
//  Created by Matt Schrage on 7/8/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
import Sentry

enum Build: String {
    case production = "prod"
    case staging = "staging"
    case dev = "dev"
}

class Defaults {
    static var isProduction: Bool {
        return Defaults.build == .production
    }
    
    static var isStaging: Bool {
        return UserDefaults.standard.string(forKey: "build") == "staging"
    }
    
    static var build: Build {
        get {
            return Build(rawValue: UserDefaults.standard.string(forKey: "build") ?? "") ?? .production
        }
        set(value) {
            UserDefaults.standard.set(value.rawValue, forKey: "build")
            UserDefaults.standard.synchronize()
            WindowManager.shared.createSidebar()
            WindowManager.shared.createAutocomplete()
            (NSApp.delegate as? AppDelegate)?.configureStatusBarItem()

        }
    }
    
    static var uuid: String {
        guard let uuid = UserDefaults.standard.string(forKey: "uuid") else {
            let uuid = UUID().uuidString
            UserDefaults.standard.set(uuid, forKey: "uuid")
            UserDefaults.standard.synchronize()
            return uuid
        }
        
        return uuid
    }
    
    static var showSidebar:Bool {
        get {
            return UserDefaults.standard.string(forKey: "sidebar") != "hidden"
        }
        
        set(value) {
            UserDefaults.standard.set(value ? "visible" : "hidden", forKey: "sidebar")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var email: String? {
        get {
            return UserDefaults.standard.string(forKey: "userEmail")
        }
        
        set(email) {
            let user = User()
            user.email = email
            SentrySDK.setUser(user)
            UserDefaults.standard.set(email, forKey: "userEmail")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var version: String {
         return Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "<unknown>"
    }
    
    static var automaticallyLaunchWebAppsInDetachedWindow: Bool {
        get {
            return UserDefaults.standard.string(forKey: "undockWebApps") == "true"
        }

        set(flag) {
            UserDefaults.standard.set(flag ? "true" : "false", forKey: "undockWebApps")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var loggedIn: Bool {
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
    static var domainToken: String? {
        get {
            return UserDefaults.standard.string(forKey: "domainToken")
        }
        
        set(token) {
            UserDefaults.standard.set(token, forKey: "domainToken")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var defaultActivePosition: CompanionWindow.OverlayPositioning {
        get {
             
            return  UserDefaults.standard.bool(forKey: "updatedDefaultActivePosition") ? CompanionWindow.OverlayPositioning(rawValue: UserDefaults.standard.integer(forKey: "defaultActivePosition")) ?? .outsideRight :  .outsideRight
        }
        
        set(id) {
            UserDefaults.standard.set(id.rawValue, forKey: "defaultActivePosition")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var shouldTrackTargetWindow: Bool {
        get {
            return
                UserDefaults.standard.bool(forKey: "shouldTrackTargetWindow")
        }
        
        set(token) {
            UserDefaults.standard.set(token, forKey: "shouldTrackTargetWindow")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var clearExistingLineOnTerminalInsert: Bool {
        get {
            return
                UserDefaults.standard.bool(forKey: "clearExistingLineOnTerminalInsert")
        }
        
        set(token) {
            UserDefaults.standard.set(token, forKey: "clearExistingLineOnTerminalInsert")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var triggerSidebarWithMouse: Bool {
        get {
            return
                UserDefaults.standard.bool(forKey: "triggerSidebarWithMouse")
        }
        
        set(token) {
            UserDefaults.standard.set(token, forKey: "triggerSidebarWithMouse")
            UserDefaults.standard.synchronize()
        }
    }
    
    static let autocompletePreferenceUpdated = Notification.Name("autocompletePreferenceUpdated")
    fileprivate static var _useAutcomplete: Bool? = nil
    static var useAutocomplete: Bool {
        get {
            if let flag = _useAutcomplete {
                return flag
            } else {
                let flag = UserDefaults.standard.bool(forKey: "useAutocomplete")
                _useAutcomplete = flag
                return flag
            }
        }
        
        set(flag) {
            guard _useAutcomplete != flag else { return }
          
            _useAutcomplete = flag
            NotificationCenter.default.post(name: Defaults.autocompletePreferenceUpdated, object: flag)
            UserDefaults.standard.set(flag, forKey: "useAutocomplete")
            UserDefaults.standard.synchronize()
            
            Settings.shared.set(value: !flag, forKey: Settings.disableAutocomplete)
            
            NSApp.appDelegate.configureStatusBarItem()
        }

    }
    
    static var playSoundWhenContextIsLost: Bool {
           get {
               return
                   UserDefaults.standard.bool(forKey: "playSoundWhenContextIsLost")
           }
           
           set(flag) {
               UserDefaults.standard.set(flag, forKey: "playSoundWhenContextIsLost")
               UserDefaults.standard.synchronize()
           }

       }
    
    static var deferToShellAutosuggestions: Bool {
           get {
               return
                   UserDefaults.standard.bool(forKey: "zshAutosuggestionPlugin")
           }
           
           set(flag) {
               UserDefaults.standard.set(flag, forKey: "zshAutosuggestionPlugin")
               UserDefaults.standard.synchronize()
           }

       }
    
    static var versionAtPreviousLaunch: String? {
        get {
            return  UserDefaults.standard.string(forKey: "versionAtPreviousLaunch")
        }
        
        set(version){
            UserDefaults.standard.set(version, forKey: "versionAtPreviousLaunch")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var debugAutocomplete: Bool {
        get {
            return
                UserDefaults.standard.bool(forKey: "debugAutocomplete")
        }
        
        set(flag) {
            guard debugAutocomplete != flag else {
              return
            }
          
            UserDefaults.standard.set(flag, forKey: "debugAutocomplete")
            UserDefaults.standard.synchronize()
          
            Settings.shared.set(value: flag, forKey: Settings.debugModeKey)

        }

    }
    
    static var broadcastLogs: Bool {
        get {
            return
                UserDefaults.standard.bool(forKey: "broadcastLogs")
        }
        
        set(flag) {
            UserDefaults.standard.set(flag, forKey: "broadcastLogs")
            UserDefaults.standard.synchronize()
        }

    }
    
    static var broadcastLogsForSubsystem: Logger.Subsystem {
        get {
            return Logger.Subsystem(rawValue: UserDefaults.standard.string(forKey: "broadcastLogsForSubsystem") ?? "") ?? .global
        }
        
        set(subsystem) {
            UserDefaults.standard.set(subsystem.rawValue, forKey: "broadcastLogsForSubsystem")
            UserDefaults.standard.synchronize()
        }

    }
    
    static var autocompleteVersion: String? {
        get {
            return  UserDefaults.standard.string(forKey: "autocompleteVersion")
        }
        
        set(version){
            UserDefaults.standard.set(version, forKey: "autocompleteVersion")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var autocompleteWidth: CGFloat? {
        get {
            let string = UserDefaults.standard.string(forKey: "autocompleteWidth")
            guard let str = string, let n = NumberFormatter().number(from: str) else { return nil }
            return n as? CGFloat
        }
        
        set(width){
            guard let width = width else { return }
            let str = NumberFormatter().string(from: NSNumber(floatLiteral: Double(width) ))
            UserDefaults.standard.set(str, forKey: "autocompleteWidth")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var processWhitelist: [String] {
        get {
            let string = UserDefaults.standard.string(forKey: "processWhitelist")
            return string?.split(separator: ",").map { String($0) } ?? []
        }
        
        set(whitelist){
            UserDefaults.standard.set(whitelist.joined(separator: ","), forKey: "processWhitelist")
            UserDefaults.standard.synchronize()
        }
        
    }
    
    static var ignoreProcessList: [String] {
        get {
            let string = UserDefaults.standard.string(forKey: "ignoreProcessList")
            return string?.split(separator: ",").map { String($0) } ?? []
        }
        
        set(whitelist){
            UserDefaults.standard.set(whitelist.joined(separator: ","), forKey: "ignoreProcessList")
            UserDefaults.standard.synchronize()
        }
        
    }

    static var launchedFollowingCrash: Bool {
        get {
            return
                UserDefaults.standard.bool(forKey: "launchedFollowingCrash")
        }
        
        set(flag) {
            UserDefaults.standard.set(flag, forKey: "launchedFollowingCrash")
            UserDefaults.standard.synchronize()
        }

    }
    
    static var onlyInsertOnTab: Bool {
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
          
            return UserDefaults.standard.bool(forKey: "onlyInsertOnTab")
        }
        
        set(flag) {
            UserDefaults.standard.set(flag, forKey: "onlyInsertOnTab")
            UserDefaults.standard.synchronize()
          
            Settings.shared.set(value: flag ? "ignore" : "insert", forKey: Settings.enterKeyBehavior)
        }

    }
    
    // determined by running `dscl . -read ~/ UserShell`
    // output: "UserShell: /bin/zsh"
    static var userShell: String {
        get {
            let shell = UserDefaults.standard.string(forKey: "userShell")
            return shell?.replacingOccurrences(of: "UserShell: ", with: "") ?? "/bin/sh"
        }
        
        set(shell) {
            var val: String?
            if (shell.starts(with: "UserShell: ")) {
                val = shell
            } else {
                val = "UserShell: \(shell)"
            }
            
            UserDefaults.standard.set(val!, forKey: "userShell")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var SSHIntegrationEnabled: Bool {
        get {
              return UserDefaults.standard.bool(forKey: "SSHIntegrationEnabled")
          }
              
          set(flag) {
              UserDefaults.standard.set(flag, forKey: "SSHIntegrationEnabled")
              UserDefaults.standard.synchronize()
          }
    }
    
    static var hasShownAutocompletePopover: Bool {
        get {
              return UserDefaults.standard.bool(forKey: "hasShownAutocompletePopover")
          }
              
          set(flag) {
              UserDefaults.standard.set(flag, forKey: "hasShownAutocompletePopover")
              UserDefaults.standard.synchronize()
          }
    }
  
    static var port: Int {
      get {
        return UserDefaults(suiteName: "com.mschrage.fig.shared")?.integer(forKey: "port") ?? 8765
      }
      
      set (port) {
        UserDefaults(suiteName: "com.mschrage.fig.shared")?.set(port, forKey: "port")
        UserDefaults(suiteName: "com.mschrage.fig.shared")?.synchronize()
      }
      
    }
  
    static var developerModeEnabled: Bool {
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
        var delta: [String:Any] = [:]
        if Settings.shared.getValue(forKey: Settings.developerModeKey) as? Bool != nil {
          delta[Settings.developerModeKey] = enabled
        }
        
        if Settings.shared.getValue(forKey: Settings.developerModeNPMKey) as? Bool != nil {
          delta[Settings.developerModeNPMKey] = enabled
        }
        
        Settings.shared.update(delta)
      }
    }
  
    @objc static func toggleDeveloperMode() {
      Defaults.developerModeEnabled = !Defaults.developerModeEnabled
    }
  
    static var beta: Bool {
      get {
        return Settings.shared.getValue(forKey: Settings.beta) as? Bool ?? false
      }
      
      set (enabled) {
        Settings.shared.set(value: enabled, forKey: Settings.beta)
      }
    }
  
    static var telemetryDisabled: Bool {
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
      var delta: [String:Any] = [:]
      if Settings.shared.getValue(forKey: Settings.legacyTelemetryDisabledKey) as? Bool != nil {
        delta[Settings.legacyTelemetryDisabledKey] = enabled
      }
      
      if Settings.shared.getValue(forKey: Settings.telemetryDisabledKey) as? Bool != nil {
        delta[Settings.telemetryDisabledKey] = enabled
      }
      
      Settings.shared.update(delta)
    }
  }
  
    static var accessibilityEnabledOnPreviousLaunch: Bool? {
        get {
            return  UserDefaults.standard.bool(forKey: "accessibilityEnabledOnPreviousLaunch")
        }
        
        set(version){
            UserDefaults.standard.set(version, forKey: "accessibilityEnabledOnPreviousLaunch")
            UserDefaults.standard.synchronize()
        }
    }
  
    static var insertUsingRightArrow: Bool {
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
            Settings.shared.set(value: flag ? "insert" :  "ignore" , forKey: Settings.rightArrowKeyBehavior)
        }

    }
}
