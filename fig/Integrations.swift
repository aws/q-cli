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
    static let Kitty = "net.kovidgoyal.kitty"
    static let Alacritty = "io.alacritty"

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
  
  static let otherTerminals = [ Kitty, Alacritty ]
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
        .union(Integrations.otherTerminals)
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
  static let providers: [String: GenericTerminalIntegrationProvider ] =
                        [ Integrations.iTerm : iTermIntegration.default,
                          Integrations.Hyper : HyperIntegration.default,
                          Integrations.VSCode : VSCodeIntegration.default,
                          Integrations.VSCodeInsiders : VSCodeIntegration.insiders,
                          Integrations.Alacritty : AlacrittyIntegration.default
                        ]
}

enum InstallationDependency: String, Codable {
    case applicationRestart
    case inputMethodActivation
}

enum InstallationStatus: Equatable {
    case applicationNotInstalled  // target app not installed,
    case unattempted    // we have not tried to install the integration
    case pending(event: InstallationDependency)
    case installed      // integration has been successfully installed
    
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
    
    //
    func staticallyVerifiable() -> Bool {
        return ![InstallationStatus.pending(event: .applicationRestart)].contains(self)
    }
}

extension InstallationStatus: Codable {
    enum CodingKeys: CodingKey {
        case unattempted, pending, installed, failed, applicationNotInstalled
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
        case .pending:
            var nestedContainer = try container.nestedUnkeyedContainer(forKey: .pending)
            let dependency = try nestedContainer.decode(InstallationDependency.self)
            self = .pending(event: dependency)
        case .unattempted:
            self = .unattempted
        case .installed:
            self = .installed
        case .applicationNotInstalled:
            self = .applicationNotInstalled
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
        case .installed:
            try container.encode(true, forKey: .installed)
        case .applicationNotInstalled:
            try container.encode(true, forKey: .applicationNotInstalled)
        case .pending(let dependency):
            var nestedContainer = container.nestedUnkeyedContainer(forKey: .pending)
            try nestedContainer.encode(dependency)

        case .failed(let error, let supportURL):
            var nestedContainer = container.nestedUnkeyedContainer(forKey: .failed)
            try nestedContainer.encode(error)
            try nestedContainer.encode(supportURL)
        }
    }
    
}

protocol IntegrationProvider {
    func verifyInstallation() -> InstallationStatus
    func install() -> InstallationStatus
}

// https://stackoverflow.com/a/51333906
// Create typealias so we can inherit from superclass while also requiring certain methods to be implemented
typealias TerminalIntegrationProvider = GenericTerminalIntegrationProvider & IntegrationProvider


extension Integrations {
    static let statusDidChange = Notification.Name("integrationStatusDidChange")
    static let integrationKey = "integrationKey"
}

class GenericTerminalIntegrationProvider {
    
    let bundleIdentifier: String
    var applicationName: String
    var applicationIsInstalled: Bool {
        
        didSet {
            if applicationIsInstalled, applicationName == bundleIdentifier {
                self.applicationName = NSWorkspace.shared.urlForApplication(withBundleIdentifier: self.bundleIdentifier)?
                    .deletingPathExtension()
                    .lastPathComponent ?? bundleIdentifier
            }
            
            if applicationIsInstalled && self.status == .applicationNotInstalled {
                self.status = .unattempted
            }
            
            if !applicationIsInstalled && self.status != .applicationNotInstalled {
                self.status = .applicationNotInstalled
            }
        }
    }

    var promptMessage: String?
    var promptButtonText: String?
    private let defaultsKey: String
    
    var status: InstallationStatus {
        didSet {
            UserDefaults.standard.set(status.encoded(), forKey: defaultsKey)
            UserDefaults.standard.synchronize()
            
            let notification = Notification(name: Integrations.statusDidChange,
                                            object: nil,
                                            userInfo: [ Integrations.integrationKey: self ])
            NotificationCenter.default.post(notification)
        }
    }
    
    init(bundleIdentifier: String) {
        self.bundleIdentifier = bundleIdentifier
        self.defaultsKey =  self.bundleIdentifier + ".integration"
        self.applicationName = NSWorkspace.shared.urlForApplication(withBundleIdentifier: self.bundleIdentifier)?
                                                 .deletingPathExtension()
                                                 .lastPathComponent ?? bundleIdentifier
        
        self.applicationIsInstalled = NSWorkspace.shared.applicationIsInstalled(self.bundleIdentifier)
        
        if self.applicationIsInstalled {
            let data = UserDefaults.standard.data(forKey: self.defaultsKey)
            self.status = InstallationStatus(data: data) ?? .unattempted
            
            if self.status.staticallyVerifiable() {
                self.verifyAndUpdateInstallationStatus()
            }
            
        } else {
            self.status = .applicationNotInstalled
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
        
        guard app.bundleIdentifier == self.bundleIdentifier else {
            return
        }
        
        if !self.applicationIsInstalled {
            self.applicationIsInstalled = true
        }
        
        if self.status == .pending(event: .applicationRestart) {
            self.verifyAndUpdateInstallationStatus()
        }

    }
    
    func _install() -> InstallationStatus {
        guard let provider = self as? TerminalIntegrationProvider else {
            return .failed(error: "TerminalIntegrationProvider does not conform to protocol.")
        }
        
        return provider.install()
    }
    
    func _verifyInstallation() -> InstallationStatus {
        guard let provider = self as? TerminalIntegrationProvider else {
            return .failed(error: "TerminalIntegrationProvider does not conform to protocol.")
        }
        
        return provider.verifyInstallation()
    }
    
    var isInstalled: Bool {
        return self._verifyInstallation() == .installed
    }
    
    func verifyAndUpdateInstallationStatus() {
        let status = _verifyInstallation()
        if self.status != status {
            self.status = status
        }
    }
    
    var shouldAttemptToInstall: Bool {
        get {
            return Defaults.loggedIn && status == .unattempted
        }
    }
    
    func install(withRestart: Bool, inBackground: Bool, completion: ((InstallationStatus) -> Void)? = nil) {
        let name = self.applicationName
        let title = "Could not install \(name) integration"

        let status = self._install()
        
        if !inBackground {
            switch status {
            case .applicationNotInstalled:
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
            default:
                break
            }
        }
        
        if withRestart && status == .pending(event: .applicationRestart) {
            let targetTerminal = Restarter(with: self.bundleIdentifier)
            targetTerminal.restart(launchingIfInactive: false) {
                self.verifyAndUpdateInstallationStatus()
                completion?(self.status)
            }
        } else {
            self.status = status
            completion?(self.status)
        }
    }

    @objc func promptToInstall() {
        promptToInstall(completion: nil)
    }
    
    @objc func openSupportPage() {
        
        switch self.status {
        case .failed(_, let supportURL):
            if let supportURL = supportURL {
                NSWorkspace.shared.open(supportURL)
            }
        default:
            break
        }
           
    }
    
    @objc func restart() {
        let targetTerminal = Restarter(with: self.bundleIdentifier)
        targetTerminal.restart(launchingIfInactive: false) {
            
            if self.status == .pending(event: .applicationRestart) {
                self.verifyAndUpdateInstallationStatus()
            }
            
        }
    }
    
    func promptToInstall(completion: ((InstallationStatus) -> Void)? = nil) {
        guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: self.bundleIdentifier) else {
          self.status = .applicationNotInstalled
          completion?(self.status)
          return
        }
        
        let icon = NSImage(imageLiteralResourceName: "NSSecurity")
        let name = self.applicationName

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
            self.status = .unattempted
            completion?(self.status)
        }
        
    }
}


class InputMethodDependentTerminalIntegrationProvider: GenericTerminalIntegrationProvider {
    override init(bundleIdentifier: String) {
        super.init(bundleIdentifier: bundleIdentifier)
        
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(inputMethodStatusDidChange),
                                               name: InputMethod.statusDidChange,
                                               object: nil)
    }
    
    deinit {
        NotificationCenter.default.removeObserver(self)
    }
    
    @objc func inputMethodStatusDidChange() {
        self.status = self._verifyInstallation()
    }
}
