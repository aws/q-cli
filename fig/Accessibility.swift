//
//  Accessibility.swift
//  fig
//
//  Created by Matt Schrage on 1/25/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import AXSwift
import Sentry

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
  
  static func checkIfPermissionRevoked() {
    let previousPermissions = Defaults.accessibilityEnabledOnPreviousLaunch
    let currentPermissions = Accessibility.enabled
    Defaults.accessibilityEnabledOnPreviousLaunch = currentPermissions

    switch (previous: previousPermissions, current: currentPermissions) {
    case (previous: true, current: true):
      print("Accessibility: permission remains enabled.")
    case (previous: false, current: true):
      print("Accessibility: permission was granted during previous session.")
    case (previous: true, current: false):
      print("Accessibility: permission was LOST since previous session.")
      SentrySDK.capture(message: "Accessibility: permission was LOST since previous session.")
    case (previous: false, current: false):
      print("Accessibility: permission has not been granted.")
    case (previous: nil, current: _):
      print("Accessibility: previous permission status not recorded.")
    default:
      print("Accessibility: unexpected state")
      

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
  
  static func listAttributes(for element: AXUIElement) {
    var names: CFArray?
    AXUIElementCopyAttributeNames(element, &names)
    print(names as Any)

    var parametrizedNames: CFArray?
    AXUIElementCopyParameterizedAttributeNames(element, &parametrizedNames)
    print(parametrizedNames as Any)
  }
  
  // https://github.com/chromium/chromium/blob/99314be8152e688bafbbf9a615536bdbb289ea87/chrome/browser/chrome_browser_application_mac.mm
  // https://github.com/electron/electron/blob/462de5f97a302987dc5fa5c222781ceed040f390/docs/tutorial/accessibility.md
  static let kAXManualAccessibility = "AXManualAccessibility" as CFString
  static func triggerScreenReaderModeInChromiumApplication(_ app: ExternalApplication) {
//    var one: NSInteger = 1;
//    let cfOne: CFNumber = CFNumberCreate(kCFAllocatorDefault, .nsIntegerType, &one);
//    AXUIElementSetAttributeValue(app.axAppRef, "AXEnhancedUserInterface" as CFString, cfOne)
//
//    var role : AnyObject?
//    let roleError = AXUIElementCopyAttributeValue(app.axAppRef, kAXRoleAttribute as CFString, &role)
//    print(roleError)
    
//    CFBooleanRef value = enable ? kCFBooleanTrue : kCFBooleanFalse;
    AXUIElementSetAttributeValue(app.axAppRef, kAXManualAccessibility, kCFBooleanTrue);
  }
  
  static func triggerScreenReaderModeInFrontmostApplication() {
    if let app = AXWindowServer.shared.topApplication {
      triggerScreenReaderModeInChromiumApplication(app)
    }
  }
  
  fileprivate static var cursorCache: [ExternalWindowHash: [UIElement]] = [:]
  static func findXTermCursorInElectronWindow(_ window: ExternalWindow, skipCache: Bool = false) -> CGRect? {
    guard let axElement = window.accesibilityElement else { return nil }
    
    var cursor: UIElement? = cursorCache[window.hash]?.filter { cursorIsActive($0) }.reduce(nil, { (existing, cache) -> UIElement? in
      guard existing == nil else {
        return existing
      }
      
//      print("cursor: elementIsCursor", elementIsCursor(cache))
      return cache //cursorIsActive(cache)// ? cache : nil //findXTermCursor(cache)
    })
    
    if skipCache {
      print("cursor: skip cache")
      cursor = nil
    }
    
    if !skipCache && cursor == nil && (cursorCache[window.hash]?.count ?? 0) > 0 {
      print("xterm-cursor: exists but is disabled (\(cursorCache[window.hash]?.count ?? 0))")
    } else if cursor == nil {
      let root = UIElement(axElement)
      cursor = findXTermCursor(root)
    } else {
      print("xterm-cursor: Cursor Cache hit!")
    }
    
    guard let currentCursor = cursor else {
      return nil
    }
    
    // create cache if it doesn't exist
    if cursorCache[window.hash] == nil {
      cursorCache[window.hash] = []
    }
    
    // Add cursor to cache if not there
    if !cursorCache[window.hash]!.contains(currentCursor) {
      cursorCache[window.hash]!.append(currentCursor)
    }

    
   
    guard let frame: CGRect = try? currentCursor.attribute(.frame) else {
      return nil
    }
    
    return  NSRect(x: frame.origin.x, y: NSMaxY(NSScreen.screens[0].frame) - frame.origin.y, width:  frame.width, height: frame.height)
  }
  
  fileprivate static func cursorIsActive(_ elm: UIElement?) -> Bool {
    if let elm = elm, let role = try? elm.role(),
      role == .textField,
      let hasKeyboardFocus: Bool = try? elm.attribute(.focused),
      hasKeyboardFocus == true {
      
      return true
    } else {
      return false
    }
  }
  
  fileprivate static func findXTermCursor(_ root: UIElement) -> UIElement? {
    if let role = try? root.role(), role == .textField, let hasKeyboardFocus: Bool = try? root.attribute(.focused), hasKeyboardFocus == true {
      return root
    }
    
    
    let children: [UIElement] = (try? root.arrayAttribute(.children)) ?? []
    
    let roles: Set<Role> = [.scrollArea, .group, .textField, .application, .browser]
    let candidates = children.map { (element) -> UIElement? in
      // optimize which elements are checked
      if let role = try? element.role(), !roles.contains(role) {
        print("role: ", role)
        return nil
      }
      return findXTermCursor(element)
    }.filter { $0 != nil }
    
    guard let candidate = candidates.first else {
      print("xterm-cursor: no candidates")
      return nil
    }
    
    if (candidates.count != 1) {
      print("xterm-cursor: There were two candidates!")
    }
    
    return candidate

  }
  
  static func openMenu(_ bundleId: String) {
    guard let elm = Application.allForBundleID(bundleId).first else { return }
    guard let menuBar = try? elm.attribute(.menuBar) as UIElement? else {
      return
    }
    
    let children: [UIElement] = (try? menuBar.arrayAttribute(.children)) ?? []
    
    // ignore first menuIterm which is Apple
    let main = children[safe: 1]
    
    try? main?.performAction(.press)
  }
  
}
