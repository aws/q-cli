//
//  SecureKeyboardInput.swift
//  fig
//
//  Created by Matt Schrage on 1/29/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import Carbon
class SecureKeyboardInput {
  static let statusChangedNotification = Notification.Name("SecureKeyboardInputStatusChangedNotification")
  fileprivate static let interval = 10.0
  static func listen() {
    Timer.scheduledTimer(timeInterval: interval, target: self, selector: #selector(checkStatus), userInfo: nil, repeats: true)
    NSWorkspace.shared.notificationCenter.addObserver(self,
                                                      selector: #selector(checkStatus),
                                                      name: NSWorkspace.didActivateApplicationNotification,
                                                      object: nil)
  }
  @objc static func checkStatus() {
    let status = SecureKeyboardInput.enabled
    if (status != previousStatus) {
        previousStatus = status
        NotificationCenter.default.post(Notification(name: SecureKeyboardInput.statusChangedNotification))
    }
  }
  
  fileprivate static var previousStatus: Bool = false
  static var wasEnabled: Bool {
    return previousStatus
  }
  static var enabled: Bool {
    return IsSecureEventInputEnabled()//CGSIsSecureEventInputSet()
  }
  
  static var responsibleProcessId: pid_t? {
    guard SecureKeyboardInput.enabled else { return nil }
    
    var pid: pid_t = 0;
    secure_keyboard_entry_process_info(&pid)
    return pid
  }
  
  static var responsibleApplication: NSRunningApplication? {
    guard let pid = SecureKeyboardInput.responsibleProcessId else { return nil }
    return NSRunningApplication(processIdentifier: pid)
  }
  
  // https://stackoverflow.com/a/35711624/926887
  @objc static func lockscreen() {
    let libHandle = dlopen("/System/Library/PrivateFrameworks/login.framework/Versions/Current/login", RTLD_LAZY)
    let sym = dlsym(libHandle, "SACLockScreenImmediate")
    typealias myFunction = @convention(c) () -> Void
    let SACLockScreenImmediate = unsafeBitCast(sym, to: myFunction.self)
    SACLockScreenImmediate()
  }
  
  fileprivate static let iTermKey = "Secure Input"
  fileprivate static let terminalKey = "SecureKeyboardEntry"

  static func enabled(by bundleIdentifier: String?) -> Bool {

    guard let bundleIdentifier = bundleIdentifier else { return false }


    let foreignDefaults = UserDefaults()
    foreignDefaults.addSuite(named: bundleIdentifier)
    switch (bundleIdentifier) {
      case Integrations.iTerm:
        return foreignDefaults.bool(forKey: iTermKey)
      case Integrations.Terminal:
        return foreignDefaults.bool(forKey: terminalKey)
      default:
       return false
    }

  }
  
  @objc static func openRelevantMenu() {
    openRelevantMenu(for: NSWorkspace.shared.frontmostApplication)
  }
  
  @objc static func openRelevantMenu(for app: NSRunningApplication? = nil) {
    guard let bundleId = app?.bundleIdentifier ?? AXWindowServer.shared.topmostWhitelistedWindow()?.bundleId else { return }
    
    if NSWorkspace.shared.menuBarOwningApplication?.bundleIdentifier == bundleId {
      Accessibility.openMenu(bundleId)
    } else if let app = app {
      
      app.activate(options: .activateIgnoringOtherApps)

      var kvo: NSKeyValueObservation? = nil
      kvo = NSWorkspace.shared.observe(\.menuBarOwningApplication, options: [.new]) { (workspace, delta) in
        if let app = delta.newValue, let bundleId = app?.bundleIdentifier, Integrations.nativeTerminals.contains(bundleId) {
          Accessibility.openMenu(bundleId)
          kvo?.invalidate()
        }
      }
    }
  }
  
  static let supportURL = URL(string:"https://fig.io/docs/support/secure-keyboard-input")!
  @objc class func openSupportPage() {
    NSWorkspace.shared.open(supportURL)
    DispatchQueue.global(qos: .background).async {
      TelemetryProvider.track(event: .openSecureInputSupportPage, with: [:])
    }
  }
  static func notifyIfEnabled() {
    guard SecureKeyboardInput.enabled else { return }
    
    let icon = NSImage(imageLiteralResourceName: "NSSecurity")
    let description: String = {
      if let responsibleApp = SecureKeyboardInput.responsibleApplication, let name = responsibleApp.localizedName  {
        return "Fig won't appear until it's disabled in '\(name)' (\(responsibleApp.processIdentifier))"
      } else {
        return "Fig won't appear until 'Secure Keyboard Input' is disabled."
      }
    }()
    
    let alert = NSAlert()
    alert.icon = icon.overlayAppIcon()
    alert.messageText = "'Secure Keyboard Input' is enabled"
    alert.informativeText = "This prevents Fig from processing keypress events.\n\n\(description)\n"
    alert.alertStyle = .warning
    
    let responsibleApp = SecureKeyboardInput.responsibleApplication

    let enabledInSettings = SecureKeyboardInput.enabled(by: responsibleApp?.bundleIdentifier)
    if (enabledInSettings) {
      let action = alert.addButton(withTitle: "Turn off")
      action.highlight(true)

    } else {
      let action = alert.addButton(withTitle: "Lock screen and log back in")
      action.highlight(true)
    }
//    button.highlight(true)
    
    alert.addButton(withTitle: "Learn more")
//    alert.addButton(withTitle: "Not now")
    
    let payload = [ "responsibleApp" : responsibleApp?.bundleIdentifier ?? "unknown"]
    DispatchQueue.global(qos: .background).async {
      TelemetryProvider.track(event: .showSecureInputEnabledAlert, with: payload)
    }
    
    let action = alert.runModal()
    let attemptFix = action == .alertFirstButtonReturn
    
    let openSupport = action == .alertSecondButtonReturn
    
    if (openSupport) {
      SecureKeyboardInput.openSupportPage()
    } else if (attemptFix) {
      if enabledInSettings {
        SecureKeyboardInput.openRelevantMenu(for: responsibleApp)
      } else {
        SecureKeyboardInput.lockscreen()
      }
    }
  }
}


