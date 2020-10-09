//
//  AXWindowServer.swift
//  fig
//
//  Created by Matt Schrage on 9/20/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
//NSAccessibility.Notification
//NSAccessibility.Notification.window
typealias AXCallbackHandler = (AXUIElement, CFString) -> Void
fileprivate extension String {
    
}

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
                                     kAXApplicationDeactivatedNotification ]

        for notification in trackedNotifications {
            AXObserverAddNotification(observer!,
                                      axAppRef,
                                      notification as CFString,
                                      selfPtr)
        }
        
        
        CFRunLoopAddSource(CFRunLoopGetCurrent(),
                           AXObserverGetRunLoopSource(observer!),
                           CFRunLoopMode.defaultMode);
        
        self.handler = handler
        return true

    }
    
    func deregisterObserver() {
        guard self.handler != nil else { return }
        self.handler = nil
        CFRunLoopRemoveSource(CFRunLoopGetCurrent(),
                              AXObserverGetRunLoopSource(observer!),
                              CFRunLoopMode.defaultMode)
        observer = nil
    }
    
}

fileprivate func axcallback(observer: AXObserver, element: AXUIElement, notificationName: CFString, refcon: UnsafeMutableRawPointer?) -> Void {
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
       
    static func ==(lhs: ExternalApplication, rhs: ExternalApplication) -> Bool {
        return lhs.bundleId == rhs.bundleId && lhs.pid == rhs.pid
    }
}




class AXWindowServer : WindowService {
    static let shared = AXWindowServer()
    var tracked: [ExternalApplication] = []
    var topWindow: ExternalWindow? = nil {
        didSet {
           print("AXWindowServer: Window = \(self.topWindow?.windowId ?? 0); App = \(self.topWindow?.bundleId ?? "<none>")")
       }
    }
    var whitelistedWindow: ExternalWindow? {
        get {
            return Integrations.whitelist.contains(self.topWindow?.bundleId ?? "") ? self.topWindow : nil
        }
       
    }
    
    func register(_ app: NSRunningApplication, fromActivation: Bool = false) {
        let appRef  = ExternalApplication(from: app)
        
        // Cannot track fig windows... (too meta!)
        guard !app.isFig else {
            print("AXWindowServer: cannot register to observe window events on Fig")
            return
        }
        
        // Cannot track spotlight window, because we don't recieve a notification when it is dismissed.
        // Spotlight is handled by checking height of selectionRect (which goes to 30 when in spotlight!)
        guard app.bundleIdentifier != "com.apple.Spotlight" else {
            print("AXWindowServer: cannot register to observe window events on com.apple.Spotlight")
            return
        }
        
        // Check if application is already tracked
        guard !self.tracked.contains(appRef) else {
            print("AXWindowServer: app '\(appRef.bundleId ?? "<none>") is already registered")
            return
        }
        
        // If app is not tracked, then no event handlers are set up to capture top window
        // Manually determine which window is on top (with delay to ensure consistency)
        if (fromActivation) {
            Timer.delayWithSeconds(0.25) {
                var window: AnyObject?

                AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
                guard window != nil else { return }
                self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)

                print("AXWindowServer: Add window manually!")
            }
            print("AXWindowServer: fromActivation!")


        }
        // set up AXOberver tracking
        let success = appRef.registerObserver { (element, notification) in
            switch (notification as String) {
            case kAXFocusedWindowChangedNotification:
                print("AXWindowServer: \(appRef.bundleId!) kAXFocusedWindowChangedNotification")
                self.topWindow = ExternalWindow(backedBy: element, in: appRef)
            case kAXMainWindowChangedNotification:
                self.topWindow = ExternalWindow(backedBy: element, in: appRef)
                print("AXWindowServer: \(appRef.bundleId!) kAXMainWindowChangedNotification")
            case kAXWindowCreatedNotification:
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
                self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)
            case kAXApplicationHiddenNotification:
                print("AXWindowServer: \(appRef.bundleId!) \(element) kAXApplicationHiddenNotification")
            case kAXApplicationActivatedNotification:
                print("AXWindowServer: \(appRef.bundleId!) \(element) kAXApplicationActivatedNotification")
                var window: AnyObject?
                AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
                guard window != nil else { return }
                self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)

            case kAXApplicationDeactivatedNotification:
                //self.whitelistedWindow = nil
                print("AXWindowServer: \(appRef.bundleId!) \(element) kAXApplicationDeactivatedNotification")
            case kAXWindowMovedNotification:
                print("AXWindowServer: \(appRef.bundleId!) \(element) kAXWindowMovedNotification")
            case kAXWindowResizedNotification:
                print("AXWindowServer: \(appRef.bundleId!) \(element) kAXWindowResizedNotification")
            case kAXUIElementDestroyedNotification:
//                guard let app = NSWorkspace.shared.frontmostApplication else { return }
//                
//                let axAppRef = AXUIElementCreateApplication(app.processIdentifier)
//                var window: AnyObject?
//                AXUIElementCopyAttributeValue(axAppRef, kAXFocusedWindowAttribute as CFString, &window)
//                guard window != nil else { return }
//                self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)

                print("AXWindowServer: \(appRef.bundleId!) \(element) kAXUIElementDestroyedNotification")
            default:
                print("AXWindowServer: unknown case")
            }
        }
        
        if (success) {
            tracked.append(appRef)
        } else {
            print("AXWindowServer: Error setting up tracking for app '\(appRef.bundleId ?? "<none>")")
        }
    }
        
    init() {
        
       registerWindowTracking()
        
        print("AXWindowServer: Tracking \(self.tracked.count) applications...")
        
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(didActivateApplication(notification:)), name: NSWorkspace.didActivateApplicationNotification, object: nil)
//        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(didDeactivateApplication(notification:)), name: NSWorkspace.didDeactivateApplicationNotification, object: nil)
//        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(activeSpaceDidChange), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)
     
    }
        
    func registerWindowTracking() {
        for app in tracked {
            app.deregisterObserver()
        }
        tracked = []
        
        for app in NSWorkspace.shared.runningApplications {// where Integrations.whitelist.contains(app.bundleIdentifier ?? "")  {
                   register(app)
        }
    }
    
    
    @objc func activeSpaceDidChange() {
        // this is used to reset previous application when space is changed. Maybe should be nil.
        //self.previousApplication = NSWorkspace.shared.frontmostApplication
    }

    @objc func didDeactivateApplication(notification: NSNotification!) {
        if let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication, Integrations.whitelist.contains(app.bundleIdentifier ?? "") {
            print("AXWindowSever:", app.bundleIdentifier ?? "")
        }
        
        //self.previousApplication = notification!.userInfo![NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
    }
    
    
    @objc func didActivateApplication(notification: Notification) {
        if let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication { //Integrations.whitelist.contains(app.bundleIdentifier ?? "") {
            print("AXWindowServer - register", app.bundleIdentifier ?? "")
            self.register(app, fromActivation: true)
        }
    }
    
    func topmostWhitelistedWindow() -> ExternalWindow? {
        return self.whitelistedWindow
    }
    
    func topmostWindow(for app: NSRunningApplication) -> ExternalWindow? {
        return nil
    }
    
    func previousFrontmostApplication() -> NSRunningApplication? {
        return nil
    }
    
    func currentApplicationIsWhitelisted() -> Bool {
        return false
    }
    
    func allWindows(onScreen: Bool) -> [ExternalWindow] {
        return []
    }
    
    func allWhitelistedWindows(onScreen: Bool) -> [ExternalWindow] {
        return []
    }
    
    func previousWhitelistedWindow() -> ExternalWindow? {
        return nil
    }
    
    func bringToFront(window: ExternalWindow) {
        
    }
    
    func takeFocus() {
        
    }
    
    func returnFocus() {
        
    }
    
    var isActivating: Bool = false
    
    var isDeactivating: Bool = false
    
    
}
