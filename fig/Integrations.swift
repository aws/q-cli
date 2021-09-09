//
//  Integrations.swift
//  fig
//
//  Created by Matt Schrage on 3/1/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Integrations {
    static let iTerm = "com.googlecode.iterm2"
    static let Terminal = "com.apple.Terminal"
    static let Hyper = "co.zeit.hyper"
    static let VSCode = "com.microsoft.VSCode"
    static let VSCodeInsiders = "com.microsoft.VSCodeInsiders"
  
    static let terminals: Set = ["com.googlecode.iterm2",
                                 "com.apple.Terminal",
                                 "io.alacritty",
                                 "co.zeit.hyper",
                                "net.kovidgoyal.kitty"]
    static let browsers:  Set = ["com.google.Chrome"]
    static let editors:   Set = ["com.apple.dt.Xcode",
                                 "com.sublimetext.3",
                                 "com.microsoft.VSCode"]
    static let nativeTerminals: Set = ["com.googlecode.iterm2",
                                       "com.apple.Terminal" ]
    static let searchBarApps: Set = ["com.apple.Spotlight",
                                     "com.runningwithcrayons.Alfred",
                                     "com.raycast.macos"]
  
  static let electronIDEs: Set = [VSCode, VSCodeInsiders]
  static var electronTerminals: Set<String> {
    get {
      let additions = Set(Settings.shared.getValue(forKey: Settings.additionalElectronTerminalsKey) as? [String] ?? [])
      
      return additions
        .union(Integrations.electronIDEs)
        .union([Integrations.Hyper])
    }
  }
  
  
    static var terminalsWhereAutocompleteShouldAppear: Set<String> {
      get {
        let additions = Set(Settings.shared.getValue(forKey: Settings.additionalTerminalsKey) as? [String] ?? [])
        return Integrations.nativeTerminals
        .union(Integrations.electronTerminals)
        .union(additions)
  .subtracting(Integrations.autocompleteBlocklist)

      }
    }
  
  static var autocompleteBlocklist: Set<String> {
      get {
          var blocklist: Set<String> = []
          if let hyperDisabled = Settings.shared.getValue(forKey: Settings.hyperDisabledKey) as? Bool, hyperDisabled {
              blocklist.insert(Integrations.Hyper)
          }
          
          if let vscodeDisabled = Settings.shared.getValue(forKey: Settings.vscodeDisabledKey) as? Bool, vscodeDisabled {
              blocklist.insert(Integrations.VSCode)
              blocklist.insert(Integrations.VSCodeInsiders)
          }
          
          if let itermDisabled = Settings.shared.getValue(forKey: Settings.iTermDisabledKey) as? Bool, itermDisabled {
              blocklist.insert(Integrations.iTerm)
          }
          
          if let terminalDisabled  = Settings.shared.getValue(forKey: Settings.terminalDisabledKey) as? Bool, terminalDisabled {
              blocklist.insert(Integrations.Terminal)
          }
          return blocklist
      }
  }
  
    static var allowed: Set<String> {
        
        get {
            if let allowed = UserDefaults.standard.string(forKey: "allowedApps") {
                return Set(allowed.split(separator: ",").map({ String($0)}))
            } else {
                return []
            }
        }
    }
    
    static var blocked: Set<String> {
        get {
           if let allowed = UserDefaults.standard.string(forKey: "blockedApps") {
               return Set(allowed.split(separator: ",").map({ String($0)}))
           } else {
               return []
           }
       }
    }
    static var whitelist: Set<String> {
        get {
            return Integrations.terminals
            .union(Integrations.allowed)
      .subtracting(Integrations.blocked)
        }
    }
  static let providers: [String: TerminalIntegrationProvider ] =
                        [ Integrations.iTerm : iTermIntegration.default,
                          Integrations.Hyper : HyperIntegration.default,
                          Integrations.VSCode : VSCodeIntegration.default,
                          Integrations.VSCodeInsiders : VSCodeIntegration.insiders]
}


enum InstallationStatus: Equatable {
    case unattempted    // we have not tried to install the integration
    case pendingRestart // waiting for the host app to restart for the integration to be active
    case installed      // integration has been successfully installed
    
    case appNotPresent  // target app not installed,
    case deniedByUser   // when prompted, the user rejected the integration prompt
    case failed(error: String, supportURL: URL? = nil)
    
    func encoded() -> Data? {
        let encoder = JSONEncoder()
        return try? encoder.encode(self)
    }
    
    init?(data: Data?) {
        guard let data = data else {
            return nil
        }
        
        let decoder = JSONDecoder()

        guard let status = try? decoder.decode(InstallationStatus.self, from: data) else {
            return nil
        }
        
        self = status
    }
}

extension InstallationStatus: Codable {
    enum CodingKeys: CodingKey {
        case unattempted, pendingRestart, installed, appNotPresent, deniedByUser, failed
    }
    
    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let key = container.allKeys.first
        
        switch key {
        case .failed:
            var nestedContainer = try container.nestedUnkeyedContainer(forKey: .failed)
            let error = try nestedContainer.decode(String.self)
            let supportURL = try nestedContainer.decode(URL?.self)
            self = .failed(error: error,
                           supportURL: supportURL)
        case .unattempted:
            self = .unattempted
        case .pendingRestart:
            self = .pendingRestart
        case .installed:
            self = .installed
        case .appNotPresent:
            self = .appNotPresent
        case .deniedByUser:
            self = .deniedByUser
        default:
            throw DecodingError.dataCorrupted(
                DecodingError.Context(
                    codingPath: container.codingPath,
                    debugDescription: "Unabled to decode enum."
                )
            )
        }
    }

    
    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        
        switch self {
        case .unattempted:
            try container.encode(true, forKey: .unattempted)
        case .pendingRestart:
            try container.encode(true, forKey: .pendingRestart)
        case .installed:
            try container.encode(true, forKey: .installed)
        case .appNotPresent:
            try container.encode(true, forKey: .appNotPresent)
        case .deniedByUser:
            try container.encode(true, forKey: .deniedByUser)
        case .failed(let error, let supportURL):
            var nestedContainer = container.nestedUnkeyedContainer(forKey: .failed)
            try nestedContainer.encode(error)
            try nestedContainer.encode(supportURL)
        }
    }
    
}

protocol IntegrationProvider {
  static func install(withRestart: Bool, inBackground: Bool, completion: (() -> Void)?)
  static var isInstalled: Bool { get }
  static func promptToInstall(completion: (() -> Void)?)
}

protocol TerminalIntegrationProvider {
    var bundleIdentifier: String { get }
    var applicationName: String? { get }
    var status: InstallationStatus { get }
    var shouldAttemptToInstall: Bool { get }
    
    // Must be implemented!
    var isInstalled: Bool { get }
    func install() -> InstallationStatus
    
    func install(withRestart: Bool, inBackground: Bool, completion: ((InstallationStatus) -> Void)?)
    func promptToInstall(completion: ((InstallationStatus) -> Void)?)

}


class GenericTerminalIntegrationProvider: TerminalIntegrationProvider {
    var shouldAttemptToInstall: Bool {
        get {
            return Defaults.loggedIn && status == .unattempted
        }
    }
    
    let bundleIdentifier: String
    var applicationName: String?
    var promptMessage: String?
    var promptButtonText: String?
    private let defaultsKey: String
    var status: InstallationStatus {
        didSet {
            UserDefaults.standard.set(status.encoded(), forKey: defaultsKey)
            UserDefaults.standard.synchronize()
        }
    }
    
    init(bundleIdentifier: String) {
        self.bundleIdentifier = bundleIdentifier
        self.defaultsKey =  self.bundleIdentifier + ".integration"
        
        if NSWorkspace.shared.applicationIsInstalled(self.bundleIdentifier) {
            let data = UserDefaults.standard.data(forKey: self.defaultsKey)
            self.status = InstallationStatus(data: data) ?? .unattempted
            self.applicationName = NSWorkspace.shared.urlForApplication(withBundleIdentifier: self.bundleIdentifier)?.deletingPathExtension().lastPathComponent

        } else {
            self.status = .appNotPresent
        }
        
        NSWorkspace.shared.notificationCenter.addObserver(self,
                                                          selector: #selector(didLaunchApplicationNotification(notification:)),
                                                          name: NSWorkspace.didLaunchApplicationNotification,
                                                          object: nil)
    }
    
    deinit {
        NSWorkspace.shared.notificationCenter.removeObserver(self)
    }
    
    @objc func didLaunchApplicationNotification(notification: Notification) {
        guard let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication else {
            return
        }
        
        if app.bundleIdentifier == self.bundleIdentifier && self.status == .appNotPresent {
            self.status = .unattempted
        }
    }
    
    func install() -> InstallationStatus {
        fatalError("GenericTerminalIntegrationProvider.install() is unimplemented" )
    }
    
    var isInstalled: Bool {
        fatalError("GenericTerminalIntegrationProvider.isInstalled is unimplemented" )
    }
    
    func install(withRestart: Bool, inBackground: Bool, completion: ((InstallationStatus) -> Void)? = nil) {
        let status = self.install()
        let name = self.applicationName ?? self.bundleIdentifier
        let title = "Could not install \(name) integration"
        
        if !inBackground {
            switch status {
            case .appNotPresent:
                Alert.show(title: title,
                           message: "\(name) is not installed.")
            case .failed(let error, let supportURL):
                
                if let supportURL = supportURL {
                    let openSupportPage = Alert.show(title: title,
                                               message: error,
                                               okText: "Learn more",
                                               icon: Alert.appIcon,
                                               hasSecondaryOption: true)
                    if (openSupportPage) {
                      NSWorkspace.shared.open(supportURL)
                    }
                    
                } else {
                    Alert.show(title: title,
                               message: error)
                }
            case .deniedByUser:
                // todo(mschrage): disable autocomplete in target terminal, if integration is denied
                break
            default:
                break
            }
        }
        
        if withRestart && status == .pendingRestart {
            let targetTerminal = Restarter(with: self.bundleIdentifier)
            targetTerminal.restart(launchingIfInactive: false) {
                self.status = .installed
                completion?(self.status)
            }
        } else {
            self.status = status
            completion?(self.status)
        }
    }

    
    func promptToInstall(completion: ((InstallationStatus) -> Void)? = nil) {
        guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: self.bundleIdentifier) else {
          self.status = .appNotPresent
          completion?(self.status)
          return
        }
        
        let icon = NSImage(imageLiteralResourceName: "NSSecurity")
        let name = self.applicationName ?? self.bundleIdentifier

        let app = NSWorkspace.shared.icon(forFile: url.path)
        let shouldInstall = Alert.show(title: "Install \(name) Integration?",
                                     message: promptMessage ?? "Fig will add a plugin to \(name) that tracks which terminal session is active.\n\n",
                                     okText: promptButtonText ?? "Install plugin",
                                     icon: icon.overlayImage(app),
                                     hasSecondaryOption: true)
        
        if shouldInstall {
          install(withRestart: true,
                  inBackground: false) { status in
            
            // Trigger accessibility if target terminal is built using electron
            if Integrations.electronTerminals.contains(self.bundleIdentifier),
               let app = AXWindowServer.shared.topApplication,
               self.bundleIdentifier == app.bundleIdentifier {
              Accessibility.triggerScreenReaderModeInChromiumApplication(app)
            }
          }
        } else {
            self.status = .deniedByUser
            completion?(self.status)
        }
        
    }
}
