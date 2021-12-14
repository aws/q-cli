//
//  AXWindowServer.swift
//  fig
//
//  Created by Matt Schrage on 9/20/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
typealias AXCallbackHandler = (AXUIElement, CFString) -> Void

class ExternalApplication {
  let pid: pid_t
  let bundleId: String?
  let title: String?
  var axAppRef: AXUIElement

  var observer: AXObserver?
  var handler: AXCallbackHandler?

  init(from app: NSRunningApplication) {
    pid = app.processIdentifier
    bundleId = app.bundleIdentifier
    title = app.localizedName
    axAppRef = AXUIElementCreateApplication(self.pid)
  }

  func registerObserver(_ handler: @escaping AXCallbackHandler ) -> Bool {

    let error = AXObserverCreate(self.pid, axcallback, &observer)

    guard error == .success else {
      print("AXWindowServer: error when registering observer for ExternalApplication")
      self.handler = nil
      self.observer = nil
      return false
    }

    let selfPtr = UnsafeMutableRawPointer(Unmanaged.passUnretained(self).toOpaque())

    let trackedNotifications = [ kAXWindowCreatedNotification,
                                 kAXFocusedWindowChangedNotification,
                                 kAXMainWindowChangedNotification,
                                 kAXWindowMiniaturizedNotification,
                                 kAXWindowDeminiaturizedNotification,
                                 kAXApplicationShownNotification,
                                 kAXApplicationHiddenNotification,
                                 kAXApplicationActivatedNotification,
                                 kAXWindowResizedNotification,
                                 kAXWindowMovedNotification,
                                 kAXUIElementDestroyedNotification,
                                 kAXApplicationDeactivatedNotification,
                                 kAXFocusedUIElementChangedNotification,
                                 kAXTitleChangedNotification
    ]

    for notification in trackedNotifications {
      AXObserverAddNotification(observer!,
                                axAppRef,
                                notification as CFString,
                                selfPtr)
    }

    CFRunLoopAddSource(CFRunLoopGetCurrent(),
                       AXObserverGetRunLoopSource(observer!),
                       CFRunLoopMode.defaultMode)

    self.handler = handler
    return true

  }

  func deregisterObserver() {
    print("AXWindowServer: Deregister observer for '\(self.bundleId ?? "<none>")'")
    self.handler = nil
    guard observer != nil else { return }
    CFRunLoopRemoveSource(CFRunLoopGetCurrent(),
                          AXObserverGetRunLoopSource(observer!),
                          CFRunLoopMode.defaultMode)
    observer = nil
  }

}

private func axcallback(observer: AXObserver, element: AXUIElement, notificationName: CFString, refcon: UnsafeMutableRawPointer?) {
  guard let refcon = refcon else { fatalError("refcon should be an ExternalApplication") }

  let app = Unmanaged<ExternalApplication>.fromOpaque(refcon).takeUnretainedValue()
  if let handler = app.handler {
    handler(element, notificationName)
  }
}

extension ExternalApplication: Hashable {
  func hash(into hasher: inout Hasher) {
    hasher.combine(self.bundleId)
    hasher.combine(self.pid)
  }

  static func == (lhs: ExternalApplication, rhs: ExternalApplication) -> Bool {
    return lhs.bundleId == rhs.bundleId && lhs.pid == rhs.pid
  }
}

class AXWindowServer: WindowService {
  static func log(_ message: String) {
    Logger.log(message: message, subsystem: .windowServer)
  }
  static let windowTitleUpdatedNotification: NSNotification.Name = .init("windowTitleUpdatedNotification")
  static let windowDidChangeNotification = Notification.Name("AXWindowServerWindowDidChangeNotification")
  static let shared = AXWindowServer()
  private let queue = DispatchQueue(label: "com.withfig.windowserver", attributes: .concurrent)
  var tracked: Set<ExternalApplication> = []
  var topApplication: ExternalApplication? {
    didSet {

      // Trigger screenreader mode in electron apps. This is probably overkill, but doing it once at launch was brittle.
      if let app = self.topApplication, Integrations.electronTerminals.contains(app.bundleIdentifier ?? "") {
        Accessibility.triggerScreenReaderModeInChromiumApplication(app)
      }
    }
  }

  var topWindow: ExternalWindow? {
    didSet {
      NotificationCenter.default.post(name: AXWindowServer.windowDidChangeNotification, object: self.topWindow)
      AXWindowServer.log("Window = \(self.topWindow?.windowId ?? 0); App = \(self.topWindow?.bundleId ?? "<none>"); pid = \(self.topWindow?.app.processIdentifier ?? 0)")
    }
  }
  var allowlistedWindow: ExternalWindow? {
    return Integrations.bundleIsValidTerminal(self.topWindow?.bundleId) ? self.topWindow : nil

  }

  static let blocklist = [ "com.apple.ViewBridgeAuxiliary",
                           "com.apple.notificationcenterui",
                           "com.apple.WebKit.WebContent",
                           "com.apple.WebKit.Networking",
                           "com.apple.controlcenter",
                           "com.mschrage.fig"
  ]

  func register(_ app: NSRunningApplication, fromActivation: Bool = false) {
    guard AXIsProcessTrustedWithOptions(nil) else {
      AXWindowServer.log("cannot register to observe window events before accesibility permissions are enabled")
      return
    }

    let appRef  = ExternalApplication(from: app)

    // Trigger screenReaderMode for supported electron terminals (probably should be moved somewhere else)
    if Integrations.electronTerminals.contains(app.bundleIdentifier ?? "") {
      Accessibility.triggerScreenReaderModeInChromiumApplication(appRef)
    }

    // Cannot track fig windows... (too meta!)
    guard !app.isFig else {
      AXWindowServer.log("cannot register to observe window events on Fig")
      return
    }

    // Cannot track spotlight window, because we don't recieve a notification when it is dismissed.
    // Spotlight is handled by checking height of selectionRect (which goes to 30 when in spotlight!)
    //        guard app.bundleIdentifier != "com.apple.Spotlight" else {
    //            print("AXWindowServer: cannot register to observe window events on com.apple.Spotlight")
    //            return
    //        }

    guard app.bundleIdentifier != nil else {
      AXWindowServer.log("cannot register to observe apps without Bundle Id")
      return
    }

    guard !AXWindowServer.blocklist.contains(app.bundleIdentifier!) else {
      AXWindowServer.log("cannot register to observe window events on \(app.bundleIdentifier!)")
      return
    }

    // prevents hanging on certain applications (like com.apple.AirDrop.send)
    guard app.activationPolicy != .prohibited else {
      AXWindowServer.log("don't track application that are prohibited from launching windows by activation policy.")
      return
    }

    // Check if application is already tracked
    if self.trackedApplications().contains(appRef) {
      AXWindowServer.log("app '\(appRef.bundleId ?? "<none>") is already registered")
      self.deregister(app: app)
      // return
    }

    // If app is not tracked, then no event handlers are set up to capture top window
    // Manually determine which window is on top (with delay to ensure consistency)
    if fromActivation {
      Timer.delayWithSeconds(0.25) {
        var window: AnyObject?

        AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
        guard window != nil else { return }
        self.topApplication = appRef
        self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)

        AXWindowServer.log("Add window manually!")
      }
      AXWindowServer.log("fromActivation!")

    }
    // set up AXOberver tracking
    let success = appRef.registerObserver { (element, notification) in
      switch notification as String {
      case kAXFocusedWindowChangedNotification:
        print("AXWindowServer: \(appRef.bundleId!) kAXFocusedWindowChangedNotification")
        self.topApplication = appRef
        self.topWindow = ExternalWindow(backedBy: element, in: appRef)
      case kAXMainWindowChangedNotification:
        self.topWindow = ExternalWindow(backedBy: element, in: appRef)
        print("AXWindowServer: \(appRef.bundleId!) kAXMainWindowChangedNotification")
      case kAXWindowCreatedNotification:
        if appRef.bundleId == "com.apple.Spotlight" {
          var window: AnyObject?
          AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
          guard window != nil else { return }
          print("Spotlight", window!)
          // AXObserverAddNotification(observer!, window, kAXWindowResizedNotification as CFString, nil)

          // self.topWindow = nil
        }
        print("AXWindowServer: \(appRef.bundleId!) kAXWindowCreatedNotification")
      case kAXWindowMiniaturizedNotification:
        print("AXWindowServer: \(appRef.bundleId!) kAXWindowMiniaturizedNotification")
      case kAXWindowDeminiaturizedNotification:
        print("AXWindowServer: \(appRef.bundleId!) kAXWindowDeminiaturizedNotification")

      case kAXApplicationShownNotification:
        print("AXWindowServer: \(appRef.bundleId!) \(element) kAXApplicationShownNotification")
        var window: AnyObject?
        AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
        guard window != nil else { return }
        self.topApplication = appRef
        self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)
      case kAXApplicationHiddenNotification:
        print("AXWindowServer: \(appRef.bundleId!) \(element) kAXApplicationHiddenNotification")
      case kAXApplicationActivatedNotification:
        print("AXWindowServer: \(appRef.bundleId!) \(element) kAXApplicationActivatedNotification")
        var window: AnyObject?
        AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
        guard window != nil else { return }
        self.topApplication = appRef
        self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)

      case kAXApplicationDeactivatedNotification:
        // self.allowlistedWindow = nil
        print("AXWindowServer: \(appRef.bundleId!) \(element) kAXApplicationDeactivatedNotification")
      case kAXWindowMovedNotification:
        print("AXWindowServer: \(appRef.bundleId!) \(element) kAXWindowMovedNotification")
        // fixes issue where opening app from spotlight loses window tracking
        guard let app = NSWorkspace.shared.frontmostApplication, app.bundleIdentifier == appRef.bundleIdentifier else {
          print("AXWindowServer: resized window of '\(appRef.bundleId!)' is not associated with frontmost application.")
          return
        }

        // update window object so that origin is accurate
        var window: AnyObject?
        AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
        guard window != nil else { return }
        self.topApplication = appRef
        self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)
      case kAXWindowResizedNotification:
        print("AXWindowServer: \(appRef.bundleId!) \(element) kAXWindowResizedNotification")

        // fixes issue where opening app from spotlight loses window tracking
        guard let app = NSWorkspace.shared.frontmostApplication, app.bundleIdentifier == appRef.bundleIdentifier else {
          print("AXWindowServer: resized window of '\(appRef.bundleId!)' is not associated with frontmost application.")
          return
        }

        // update window object so that bounds are accurate
        var window: AnyObject?
        AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
        guard window != nil else { return }
        self.topApplication = appRef
        self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)
      case kAXUIElementDestroyedNotification:
        var pid: pid_t = 0
        _ = AXUIElementGetPid(element, &pid)

        // determine if AXUIElement is window???
        let app = NSRunningApplication(processIdentifier: pid)

        // spotlight style app
        if Integrations.searchBarApps.contains(app?.bundleIdentifier ?? "") {
          guard let frontmost = NSWorkspace.shared.frontmostApplication else { return }
          print("AXWindowServer: spotlightStyleAppDestroyed! frontmost = \(frontmost.bundleIdentifier ?? "<none>")")
          let axAppRef = AXUIElementCreateApplication(frontmost.processIdentifier)
          var window: AnyObject?
          AXUIElementCopyAttributeValue(axAppRef, kAXFocusedWindowAttribute as CFString, &window)
          guard window != nil else { return }
          self.topApplication = appRef
          self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: ExternalApplication(from: frontmost))
        }

      case kAXTitleChangedNotification:
        var window: AnyObject?
        AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
        guard window != nil else { return }
        let topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)
        guard Integrations.bundleIsValidTerminal(topWindow?.bundleId) else { return }
        NotificationCenter.default.post(name: AXWindowServer.windowTitleUpdatedNotification, object: topWindow)

      default:
        print("AXWindowServer: unknown case")
      }
    }

    if success {
      AXWindowServer.log("Began tracking '\(appRef.bundleId ?? "<none>")'")
      // EXC_BAD_ACCESS (code=EXC_I386_GPFLT
      // Duplicate elements of type 'ExternalApplication' were found in a Set
      startTracking(app: appRef)

    } else {
      AXWindowServer.log("Error setting up tracking for app '\(appRef.bundleId ?? "<none>")")
    }
  }

  func deregister(app: NSRunningApplication) {
    for trackedApp in self.trackedApplications() where trackedApp.bundleId == app.bundleIdentifier {
      // EXC_BAD_ACCESS (code=EXC_I386_GPFLT)
      stopTracking(app: trackedApp)
    }

  }

  func trackedApplications() -> Set<ExternalApplication> {
    var trackedApplications: Set<ExternalApplication>!

    queue.sync {
      trackedApplications = self.tracked
    }

    return trackedApplications
  }

  init() {

    registerWindowTracking()

    NSWorkspace.shared.notificationCenter.addObserver(self,
                                                      selector: #selector(didActivateApplication(notification:)),
                                                      name: NSWorkspace.didActivateApplicationNotification,
                                                      object: nil)

    NSWorkspace.shared.notificationCenter.addObserver(self,
                                                      selector: #selector(didTerminateApplication(notification:)),
                                                      name: NSWorkspace.didTerminateApplicationNotification,
                                                      object: nil)

    NSWorkspace.shared.notificationCenter.addObserver(self,
                                                      selector: #selector(didLaunchApplicationNotification(notification:)),
                                                      name: NSWorkspace.didLaunchApplicationNotification,
                                                      object: nil)

    NSWorkspace.shared.notificationCenter.addObserver(self,
                                                      selector: #selector(didDeactivateApplication(notification:)),
                                                      name: NSWorkspace.didDeactivateApplicationNotification,
                                                      object: nil)

    NSWorkspace.shared.notificationCenter.addObserver(self,
                                                      selector: #selector(activeSpaceDidChange),
                                                      name: NSWorkspace.activeSpaceDidChangeNotification,
                                                      object: nil)

    NotificationCenter.default.addObserver(self,
                                           selector: #selector(accesibilityPermissionsUpdated),
                                           name: Accessibility.permissionDidUpdate,
                                           object: nil)

  }

  @objc func accesibilityPermissionsUpdated(_ notification: Notification) {
    guard let granted = notification.object as? Bool else { return }

    if granted {
      self.registerWindowTracking()
    } else {
      self.deregisterWindowTracking()
    }
  }

  func registerWindowTracking() {
    deregisterWindowTracking()

    // capture topmost app's window
    if let app = NSWorkspace.shared.frontmostApplication {
      register(app, fromActivation: true)
    }

    for app in NSWorkspace.shared.runningApplications {// where Integrations.allowlist.contains(app.bundleIdentifier ?? "")  {
      register(app)
    }

    AXWindowServer.log("Tracking \(self.tracked.count) applications...")

  }

  func deregisterWindowTracking() {
    for app in self.trackedApplications() {
      stopTracking(app: app)
    }

    self.topApplication = nil
    self.topWindow = nil
  }

  func startTracking(app: ExternalApplication) {
    queue.async(flags: [.barrier]) {
      self.tracked.insert(app)
    }
  }

  func stopTracking(app: ExternalApplication) {
    app.deregisterObserver()

    queue.async(flags: [.barrier]) {
      self.tracked.remove(app)
    }

  }

  @objc func activeSpaceDidChange() {
    // this is used to reset previous application when space is changed. Maybe should be nil.
    // self.previousApplication =
    if let app = NSWorkspace.shared.frontmostApplication {
      Logger.log(message: "activeSpaceDidChange - \(app.bundleIdentifier ?? "<none>")", subsystem: .windowEvents)
      // self.register(app, fromActivation: true)
    }
  }

  @objc func didDeactivateApplication(notification: NSNotification!) {
    if let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication, Integrations.allowlist.contains(app.bundleIdentifier ?? "") {
      Logger.log(message: "didDeactivateApplication - \(app.bundleIdentifier ?? "<none>")", subsystem: .windowEvents)

    }

    // self.previousApplication = notification!.userInfo![NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
  }

  @objc func didActivateApplication(notification: Notification) {
    if let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication {
      Logger.log(message: "didActivateApplication - \( app.bundleIdentifier ?? "<nonde>")", subsystem: .windowEvents)
      self.register(app, fromActivation: true)
    }
  }

  @objc func didTerminateApplication(notification: Notification) {
    if let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication {
      Logger.log(message: "didTerminateApplication - \( app.bundleIdentifier ?? "<nonde>")", subsystem: .windowEvents)

      // Determine if this is the only instance of the application before removing window observers
      // This resolves VSCode window issues caused when running `code .` inside the integrated terminal
      if !NSWorkspace.shared.runningApplications.contains(where: { runningApp in
        return runningApp.bundleIdentifier == app.bundleIdentifier
      }) {
        Logger.log(message: "Deregistering app (\(app.bundleIdentifier ?? "???")) since no other instances are running",
                   subsystem: .windowServer)
        self.deregister(app: app)
      }
    }
  }

  @objc func didLaunchApplicationNotification(notification: Notification) {
    if let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication {
      Logger.log(message: "didLaunchApplication - \( app.bundleIdentifier ?? "<nonde>")", subsystem: .windowEvents)
      // This register function is required in order to track new windows when an app is launched!
      self.register(app, fromActivation: true)
    }
  }

  @objc func top() -> AXUIElement? {
    let systemWideElement: AXUIElement = AXUIElementCreateSystemWide()

    var window: AnyObject?
    _ = AXUIElementCopyAttributeValue(systemWideElement, kAXFocusedWindowAttribute as CFString, &window)
    // print("hacky", result, window)
    return nil
  }

  func topmostAllowlistedWindow() -> ExternalWindow? {
    return self.allowlistedWindow
  }

  func topmostWindow(for app: NSRunningApplication) -> ExternalWindow? {
    return nil
  }

  func previousFrontmostApplication() -> NSRunningApplication? {
    return nil
  }

  func currentApplicationIsAllowlisted() -> Bool {
    return false
  }

  func allWindows(onScreen: Bool) -> [ExternalWindow] {
    return []
  }

  func allAllowlistedWindows(onScreen: Bool) -> [ExternalWindow] {
    return []
  }

  func previousAllowlistedWindow() -> ExternalWindow? {
    return nil
  }

  func bringToFront(window: ExternalWindow) {}

  func takeFocus() {}

  func returnFocus() {}

  var isActivating: Bool = false

  var isDeactivating: Bool = false

}
