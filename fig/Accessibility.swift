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
    let previousPermissions = Defaults.shared.accessibilityEnabledOnPreviousLaunch
    let currentPermissions = Accessibility.enabled
    Defaults.shared.accessibilityEnabledOnPreviousLaunch = currentPermissions

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
      Accessibility.setGlobalTimeout(seconds: 2)
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
  
  // THANKFULLY WE DON'T DO THIS!
  // This is an unfortunate hack that is necessary because there is no VSCode API to determe when the terminal has focus.
  // Currently, we use whether the cursor AXElement exists and has focus as a proxy. This is cached to avoid performance penalty.
  // However, when the bottom panel is closed, the cache always misses and typing can become noticably slow.
  // We're going to fight fire with fire and use the AXAPI to check if the panel is open.
  // This is completely an implementation detail of VSCode and if (when) it changes in the future, this method will not work...
  
  // Approach:
  // The editor pane is contained in a <main> tag which WebKit maps to the subrole "AXLandmarkMain". (https://bugs.webkit.org/show_bug.cgi?id=103172)
  // This <main> tag is the direct child of a group that contains the bottom pane as well.
  // Identify <main> tag and than check the child of the parent. This should be more performant since the search is shallower and terminates early.
  //  static func findPanel(_ parent: UIElement, siblings: [UIElement]) -> UIElement? {
  //    let AXLandmarkMain = "AXLandmarkMain"
  //    let children: [UIElement] = (try? parent.arrayAttribute(.children)) ?? []
  //
  //
  //    var containsMainAsChild = false
  //    for element in children {
  //      if let subrole: String = try? element.attribute(.subrole),
  //         subrole == AXLandmarkMain {
  //        containsMainAsChild = true
  //        break;
  //      }
  //    }
  //
  //    guard !containsMainAsChild  else {
  //
  //      // if the parent has 1 sibling, then it must be the panel!
  //      if (siblings.count == 1) {
  //        return siblings.first!
  //      }
  //
  //      return nil
  //    }
  //
  //    // continue searching...
  //    let roles: Set<Role> = [.scrollArea, .group, .application, .browser]
  //
  //    for index in 0..<children.count {
  //      let element = children[index]
  //
  //      if let role = try? element.role(), !roles.contains(role) {
  //        continue
  //      }
  //
  //      let siblings: [UIElement] =  Array(children[0..<index] + children[index+1..<children.count])
  //      if let panel = findPanel(element, siblings:siblings) {
  //        return panel
  //      }
  //    }
  //
  //    return nil
  //  }
  
  static let throttler = Throttler(minimumDelay: 0.1, queue: DispatchQueue(label: "io.fig.electron-cursor"))

  fileprivate static var cachedCursor: UIElement? = nil
  fileprivate static var cursorCache: [ExternalWindowHash: [UIElement]] = [:]
  static func resetCursorCache() {
    cachedCursor = nil
    cursorCache = [:]
  }
  static func findXTermCursorInElectronWindow(_ window: ExternalWindow, skipCache: Bool = false) -> CGRect? {
    guard let axElement = window.accesibilityElement else { return nil }
    
    // remove invalid entries; this fixes the issue with VSCode where upon changing tabs, some cached cursors go stale
    cursorCache[window.hash] = cursorCache[window.hash]?.filter { isValidUIElement($0) }
    
    var cursor: UIElement? = cursorCache[window.hash]?.filter { cursorIsActive($0) }.reduce(nil, { (existing, cache) -> UIElement? in
      guard existing == nil else {
        return existing
      }
      
      return cache
    })
    
    let root = UIElement(axElement)
    
    if skipCache {
     Accessibility.xtermLog("skip cache")
      cursor = nil
    }
    
    print("xterm-cursor: windowId = \(window.windowId)")
    
    if cursor != nil,
       let toplevelElement: UIElement = try? cursor?.attribute(.topLevelUIElement),
       root != toplevelElement {
//       let windowTitle: String = try? root.attribute(.title),
//       let elementTitle: String = try? toplevelElement.attribute(.title),
//       windowTitle != elementTitle {
        print("xterm-cursor: window for cached cursor (\(String(describing: toplevelElement)) is not equal to current window (\(String(describing: root))")
//        print("xterm-cursor: window for cached cursor (\(elementTitle)) is not equal to current window (\(windowTitle)")
        cursor = nil
        cursorCache[window.hash] = []
    }
    
    // some additional checks and performance optimization are put in place for VSCode (and other electron IDEs) so only enable it for them!
    let isElectronIDE = Integrations.electronIDEs.contains(window.bundleId ?? "")

    if cursor == nil {
      
      // Hyper has a small enough a11y tree that we can synchronously find cursor every time
      if skipCache || !isElectronIDE {
        let root = UIElement(axElement)
        cachedCursor = findXTermCursor(root, inVSCodeIDE: isElectronIDE)
      } else {
        
        throttler.throttle {
          cachedCursor = findXTermCursor(root, inVSCodeIDE: isElectronIDE)
          // trigger reposition if cursor has been found
          print("xterm-cursor: finished searching for cursor (throttled) \(String(describing: cachedCursor))")
          if cachedCursor != nil {
            Autocomplete.position(makeVisibleImmediately: true, completion: nil)
          }
        }
                
      }
      
      cursor = cachedCursor


    } else {
     Accessibility.xtermLog("Cursor Cache hit!")
    }
    
    guard let currentCursor = cursor else {
      return nil
    }
    
    // ensure that pid associated cursor matches pid associated with window
    guard let pid = try? currentCursor.pid(),
          pid == window.app.processIdentifier else {
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
      print("xterm-cursor: no frame!")
      return nil
    }
    
    print("xterm-cursor: \(frame)")
    return  NSRect(x: frame.origin.x,
                   y: NSMaxY(NSScreen.screens[0].frame) - frame.origin.y,
                   width:  frame.width,
                   height: frame.height)
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
  
  fileprivate static func isValidUIElement(_ elm: UIElement?) -> Bool {
    guard let elm = elm else { return false }
    do {
      let _ = try elm.role()
      return true
    } catch AXError.invalidUIElement {
      return false
    } catch {
      return true
    }
  }
  
  fileprivate static func findXTermCursor(_ root: UIElement, inVSCodeIDE: Bool = false, depth: Int = 0) -> UIElement? {
    
    if let role = try? root.role(), role == .textField, let hasKeyboardFocus: Bool = try? root.attribute(.focused), hasKeyboardFocus == true {
      
      print("xterm-cursor: success \(depth)")
      // VSCode-specific cursor sanity checking to ensure Fig window doesn't appear in other textfields
      // Look for great great grand with subrole of AXDocument!
      if inVSCodeIDE,
         let up1:UIElement = try? root.attribute(.parent),
         let up2:UIElement = try? up1.attribute(.parent),
         let document:UIElement = try? up2.attribute(.parent),
         let subrole: String = try? document.attribute(.subrole),
         subrole == "AXDocument" {
          
        return root

      } else {
        return root
      }

      
    }
    
    
    let children: [UIElement] = (try? root.arrayAttribute(.children)) ?? []
    
    let roles: Set<Role> = [.scrollArea, .group, .textField, .application, .browser]
    let candidates = children.map { (element) -> UIElement? in
      // optimize which elements are checked
      if let role = try? element.role(), !roles.contains(role) {
//        print("role: ", role)
        return nil
      }
      return findXTermCursor(element, inVSCodeIDE: inVSCodeIDE, depth: depth + 1)
    }.filter { $0 != nil }
    
    guard let candidate = candidates.first else {
      return nil
    }
    
    if (candidates.count != 1) {
     Accessibility.xtermLog("There were two candidates!")
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
 
  static func focusedApplicationIsSupportedTerminal() -> Bool {
    let systemWideElement: UIElement = UIElement(AXUIElementCreateSystemWide())

    
    guard let focusedElement: UIElement = try? systemWideElement.attribute(.focusedUIElement) else {
      return false
    }

    guard let pid = try? focusedElement.pid(),
          let app = NSRunningApplication(processIdentifier: pid),
          Integrations.terminalsWhereAutocompleteShouldAppear.contains(app.bundleIdentifier ?? "") else {
      return false
    }
    
    return true
    
  }
    
static func getCursorRect(extendRange: Bool = true) -> NSRect? {
    let systemWideElement = AXUIElementCreateSystemWide()
    var focusedElement : AnyObject?
    let error = AXUIElementCopyAttributeValue(systemWideElement, kAXFocusedUIElementAttribute as CFString, &focusedElement)
    guard error == .success else {
      Logger.log(message: "Couldn't get the focused element.", subsystem: .cursor)
      return nil
    }
    
    var selectedRangeValue : AnyObject?
    let selectedRangeError = AXUIElementCopyAttributeValue(focusedElement as! AXUIElement, kAXSelectedTextRangeAttribute as CFString, &selectedRangeValue)
    
    guard selectedRangeError == .success else {
      Logger.log(message: "couldn't get selected range", subsystem: .cursor)
      return nil
    }
    
    var selectedRange = CFRange()
    AXValueGetValue(selectedRangeValue as! AXValue, .cfRange, &selectedRange)
    var selectRect = CGRect()
    var selectBounds : AnyObject?
    
    // ensure selected text range is at least 1 - in order to find rect.
    if (extendRange) {
      var updatedRange = CFRangeMake(selectedRange.location, 1)
      withUnsafeMutablePointer(to: &updatedRange) { (ptr) in
        selectedRangeValue = AXValueCreate(.cfRange, ptr)
      }
    }
    
    // https://linear.app/fig/issue/ENG-109/ - autocomplete-popup-shows-when-copying-and-pasting-in-terminal
    if selectedRange.length > 1 {
      Logger.log(message: "selectedRange length > 1", subsystem: .cursor)
      return nil
    }
    
    let selectedBoundsError = AXUIElementCopyParameterizedAttributeValue(focusedElement as! AXUIElement, kAXBoundsForRangeParameterizedAttribute as CFString, selectedRangeValue!, &selectBounds)
    
    guard selectedBoundsError == .success else {
      Logger.log(message: "selectedBoundsError", subsystem: .cursor)
      return nil
    }
    
    AXValueGetValue(selectBounds as! AXValue, .cgRect, &selectRect)
   Logger.log(message: "\(selectRect)", subsystem: .cursor)
    
    // Sanity check: prevents flashing autocomplete in bottom corner
    guard selectRect.size != .zero else {
      Logger.log(message: "prevents flashing autocomplete in bottom corner", subsystem: .cursor)
      return nil
    }
    
    // convert Quartz coordinate system to Cocoa!
    return NSRect(x: selectRect.origin.x, y: NSMaxY(NSScreen.screens[0].frame) - selectRect.origin.y, width:  selectRect.width, height: selectRect.height)
}
    
  static func getTextRect() -> CGRect? {
    
    // prevent cursor position for being returned when apps like spotlight & alfred are active
    
    guard Accessibility.focusedApplicationIsSupportedTerminal() else {
        return nil
    }
    
    guard let window = AXWindowServer.shared.whitelistedWindow else {
        return nil
    }
    
    return window.cursor
  }

  fileprivate static func xtermLog(_ message: String){
    Logger.log(message: message, subsystem: .xtermCursor)
  }
  
  static func setGlobalTimeout(seconds: Float) {
    let result = AXUIElementSetMessagingTimeout(AXUIElementCreateSystemWide(), seconds)
    
    if result != .success {
      SentrySDK.capture(message: "Error setting AX global timeout")
    }
  }
}

