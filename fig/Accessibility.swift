//
//  Accessibility.swift
//  fig
//
//  Created by Matt Schrage on 1/25/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Accessibility {
  static let permissionDidUpdate = Notification.Name("accessibilityPermissionDidUpdate")
  static var enabled: Bool {
    return AXIsProcessTrusted()
  }
  
  static func listen() {
    let center = DistributedNotificationCenter.default()
    let accessibilityChangedNotification = NSNotification.Name("com.apple.accessibility.api")
    center.addObserver(forName: accessibilityChangedNotification, object: nil, queue: nil) { _ in

      DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
        NotificationCenter.default.post(name: Accessibility.permissionDidUpdate, object: Accessibility.enabled)
      }
    }
  }
  
  static func openAccessibilityPermissionsInSystemPreferences() {
    NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!)
  }
  
  fileprivate static var pendingPermission: Bool = false
  
  static func promptForPermission(completion: ((Bool) -> Void)? = nil) {
    guard !Accessibility.enabled else {
      print("Accessibility Permission Granted!")
      completion?(true)
      return
    }
    
    openAccessibilityPermissionsInSystemPreferences()
    
    guard !pendingPermission else { return }
    pendingPermission = true
    
    DispatchQueue.global(qos: .background).async {
        TelemetryProvider.track(event: .promptedForAXPermission, with: [:])
    }
    
    Accessibility.waitForNextUpdate { (granted) in
      DispatchQueue.global(qos: .background).async {
        TelemetryProvider.track(event: .grantedAXPermission, with: [:])
      }
      print("Accessibility Permission Granted!!!")
      completion?(granted)
      Accessibility.pendingPermission = false
    }
    
  }
  
  static func waitForNextUpdate(whereGranted: Bool = true, completion: @escaping (Bool) -> Void) {
    let center = DistributedNotificationCenter.default()
    let accessibilityChangedNotification = NSNotification.Name("com.apple.accessibility.api")
    var observer: NSObjectProtocol?
    observer = center.addObserver(forName: accessibilityChangedNotification, object: nil, queue: nil) { _ in

    DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
        // only stop observing only when value is true
        let granted = Accessibility.enabled
        

        if (granted) {
          completion(enabled)
        } else if (!whereGranted) {
           completion(enabled)
        }
        
        // remove observer if we just wanted
        // whatever the next update was (on or off)
        // or remove if granted
        if (!whereGranted || granted) {
          center.removeObserver(observer!)
        }
  
      }
    }
  }
  
}
