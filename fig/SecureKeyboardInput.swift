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
  
  static let supportURL = URL(string:"https://withfig.com/docs/support/secure-keyboard-input")!
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
    let button = alert.addButton(withTitle: "Learn more")
    button.highlight(true)
    alert.addButton(withTitle: "Not now")
    
    let payload = [ "responsibleApp" : SecureKeyboardInput.responsibleApplication?.bundleIdentifier ?? "unknown"]
    DispatchQueue.global(qos: .background).async {
      TelemetryProvider.track(event: .showSecureInputEnabledAlert, with: payload)
    }
    
    let openSupport = alert.runModal() == .alertFirstButtonReturn
    
    if (openSupport) {
      NSWorkspace.shared.open(supportURL)
      DispatchQueue.global(qos: .background).async {
        TelemetryProvider.track(event: .openSecureInputSupportPage, with: payload)
      }
    }
  }
}


