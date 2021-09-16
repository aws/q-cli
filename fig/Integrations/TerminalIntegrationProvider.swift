//
//  TerminalIntegrationProvider.swift
//  fig
//
//  Created by Matt Schrage on 9/14/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

//
protocol TerminalIntegration {
    func getCursorRect(in window: ExternalWindow) -> NSRect?
    func terminalIsFocused(in window: ExternalWindow) -> Bool
}

protocol TerminalIntegrationUI {
    var bundleIdentifier: String { get }
    var applicationName: String { get }
    var applicationIsInstalled: Bool { get }
    
    func install(withRestart: Bool, inBackground: Bool, completion: ((InstallationStatus) -> Void)?)
    func promptToInstall(completion: ((InstallationStatus) -> Void)?)
    func restart()
    func promptToInstall()
    func openSupportPage()
}
// https://stackoverflow.com/a/51333906
// Create typealias so we can inherit from superclass while also requiring certain methods to be implemented
typealias TerminalIntegrationProvider = GenericTerminalIntegrationProvider & TerminalIntegration & IntegrationProvider

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
        guard let provider = self as? IntegrationProvider else {
            return .failed(error: "TerminalIntegrationProvider does not conform to protocol.")
        }
        
        return provider.install()
    }
    
    func _verifyInstallation() -> InstallationStatus {
        guard let provider = self as? IntegrationProvider else {
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
    
    func currentVersionIsSupported(minimumVersion: SemanticVersion) -> InstallationStatus? {
        
        guard let bundleURL =  NSWorkspace.shared.urlForApplication(withBundleIdentifier: Integrations.Kitty) else {
            return .failed(error: "Could not determine bundle URL")
        }
        guard let bundle = Bundle(url: bundleURL) else {
            return .failed(error: "Could not load info.plist ")
        }
        
        guard let versionString = bundle.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String else {
            return .failed(error: "Could not determine application version ")
        }
        
        guard let version = SemanticVersion(version: versionString) else {
            return .failed(error: "Could not parse version string (\(versionString))")
        }
        
        guard version >= minimumVersion else {
            return .failed(error: "\(self.applicationName) version \(version.string) is not supported. Must be \(minimumVersion.string) or above")
        }
        
        return nil
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
