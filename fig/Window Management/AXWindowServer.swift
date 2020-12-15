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
                                     kAXFocusedUIElementChangedNotification
            
                                   ]

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
        print("AXWindowServer: Deregister observer for '\(self.bundleId ?? "<none>")'")
        self.handler = nil
        guard observer != nil else { return }
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
    static let windowDidChangeNotification = Notification.Name("AXWindowServerWindowDidChangeNotification")
    static let shared = AXWindowServer()
    var tracked: Set<ExternalApplication> = []
    var topWindow: ExternalWindow? = nil {
        didSet {
            NotificationCenter.default.post(name: AXWindowServer.windowDidChangeNotification, object: self.topWindow)
            Logger.log(message: "AXWindowServer: Window = \(self.topWindow?.windowId ?? 0); App = \(self.topWindow?.bundleId ?? "<none>"); pid = \(self.topWindow?.app.processIdentifier ?? 0)")
       }
    }
    var whitelistedWindow: ExternalWindow? {
        get {
            return Integrations.whitelist.contains(self.topWindow?.bundleId ?? "") ? self.topWindow : nil
        }
       
    }
    
    func register(_ app: NSRunningApplication, fromActivation: Bool = false) {
        guard AXIsProcessTrustedWithOptions(nil) else {
            Logger.log(message: "AXWindowServer: cannot register to observe window events before accesibility permissions are enabled")
            return
        }

        let appRef  = ExternalApplication(from: app)
        
        // Cannot track fig windows... (too meta!)
        guard !app.isFig else {
            Logger.log(message: "AXWindowServer: cannot register to observe window events on Fig")
            return
        }
        
        // Cannot track spotlight window, because we don't recieve a notification when it is dismissed.
        // Spotlight is handled by checking height of selectionRect (which goes to 30 when in spotlight!)
//        guard app.bundleIdentifier != "com.apple.Spotlight" else {
//            print("AXWindowServer: cannot register to observe window events on com.apple.Spotlight")
//            return
//        }
        
        guard app.bundleIdentifier != "com.apple.ViewBridgeAuxiliary" else {
            Logger.log(message: "AXWindowServer: cannot register to observe window events on com.apple.ViewBridgeAuxiliary")
            return
        }
        
        guard app.bundleIdentifier != "com.apple.notificationcenterui" else {
            Logger.log(message: "AXWindowServer: cannot register to observe window events on com.apple.notificationcenterui")
            return
        }
        
        guard app.bundleIdentifier != nil else {
           Logger.log(message: "AXWindowServer: cannot register to observe apps without Bundle Id")
            return
        }
        
        guard app.bundleIdentifier != "com.apple.WebKit.WebContent" else {
            Logger.log(message: "AXWindowServer: cannot register to observe window events on com.apple.WebKit.WebContent")
            return
        }
        
        guard app.bundleIdentifier != "com.apple.WebKit.Networking" else {
            Logger.log(message: "AXWindowServer: cannot register to observe window events on com.apple.WebKit.Networking")
            return
        }
        
        // prevents hanging on certain applications (like com.apple.AirDrop.send)
        guard app.activationPolicy != .prohibited else {
            Logger.log(message: "AXWindowServer: don't track application that are prohibited from launching windows by activation policy.")
            return
        }
        
//        if appRef.observer == nil || appRef.handler == nil {
//            self.tracked = tracked.filter { $0 != appRef}
//        }
        
        
//        self.tracked.contains { return $0.bundleId == app.bundleIdentifier && $0.pid == app.processIdentifier}
        // Check if application is already tracked
        if self.tracked.contains(appRef)  {
            Logger.log(message: "AXWindowServer: app '\(appRef.bundleId ?? "<none>") is already registered")
            self.deregister(app: app)
            //return
        }
        
        // If app is not tracked, then no event handlers are set up to capture top window
        // Manually determine which window is on top (with delay to ensure consistency)
        if (fromActivation) {
            Timer.delayWithSeconds(0.25) {
                var window: AnyObject?

                AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
                guard window != nil else { return }
                self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)

                Logger.log(message: "AXWindowServer: Add window manually!")
            }
           Logger.log(message: "AXWindowServer: fromActivation!")


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
                if (appRef.bundleId == "com.apple.Spotlight") {
//                    Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { (timer) in
//                        let spotlight = WindowServer.shared.allWindows().filter { $0.bundleId == "com.apple.Spotlight" }
//                        print("spotlight: \(spotlight.count) count")
//                        spotlight.forEach {
//                            print("spotlight: \($0.frame)")
//                        }
//                        if (spotlight.count == 1) {
//                            print("spotlight: dismissed")
//                            timer.invalidate()
//
//                            if let app = NSWorkspace.shared.frontmostApplication {
//                                self.register(app, fromActivation: true)
//                            }
//                        }
//                    }
                    var window: AnyObject?
                    AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
                    guard window != nil else { return }
                    print("Spotlight", window)
                    //AXObserverAddNotification(observer!, window, kAXWindowResizedNotification as CFString, nil)

                    //self.topWindow = nil
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
                // fixes issue where opening app from spotlight loses window tracking
                guard let app = NSWorkspace.shared.frontmostApplication, app.bundleIdentifier == appRef.bundleIdentifier else {
                    print("AXWindowServer: resized window of '\(appRef.bundleId!)' is not associated with frontmost application.")
                    return
                }
                
                //update window object so that origin is accurate
                var window: AnyObject?
                AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
                guard window != nil else { return }
                self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)
            case kAXWindowResizedNotification:
                print("AXWindowServer: \(appRef.bundleId!) \(element) kAXWindowResizedNotification")
                
                // fixes issue where opening app from spotlight loses window tracking
                guard let app = NSWorkspace.shared.frontmostApplication, app.bundleIdentifier == appRef.bundleIdentifier else {
                    print("AXWindowServer: resized window of '\(appRef.bundleId!)' is not associated with frontmost application.")
                    return
                }
                
                //update window object so that bounds are accurate
                var window: AnyObject?
                AXUIElementCopyAttributeValue(appRef.axAppRef, kAXFocusedWindowAttribute as CFString, &window)
                guard window != nil else { return }
                self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: appRef)
            case kAXUIElementDestroyedNotification:
                var pid: pid_t = 0
                let err = AXUIElementGetPid(element, &pid)
                
                // determine if AXUIElement is window???
              
                let app = NSRunningApplication(processIdentifier: pid)
                print("AXWindowServer: \(app?.bundleIdentifier)! \(element) kAXUIElementDestroyedNotification")
                
                // spotlight style app
                if (Integrations.searchBarApps.contains(app?.bundleIdentifier ?? "") ) {
                        guard let frontmost = NSWorkspace.shared.frontmostApplication else { return }
                        print("AXWindowServer: frontmost = \(frontmost.bundleIdentifier ?? "<none>")")
                        let axAppRef = AXUIElementCreateApplication(frontmost.processIdentifier)
                        var window: AnyObject?
                        AXUIElementCopyAttributeValue(axAppRef, kAXFocusedWindowAttribute as CFString, &window)
                        guard window != nil else { return }
                        self.topWindow = ExternalWindow(backedBy: window as! AXUIElement, in: ExternalApplication(from: frontmost))
                }
                

                
            
                break;


//                print("AXWindowServer: \(appRef.bundleId!) \(element) kAXUIElementDestroyedNotification")
            case kAXFocusedUIElementChangedNotification:
                print("AXWindowServer: \(appRef.bundleId!) \(element) kAXFocusedUIElementChangedNotification")
            default:
                print("AXWindowServer: unknown case")
            }
        }
        
        if (success) {
            Logger.log(message: "AXWindowServer: Began tracking '\(appRef.bundleId ?? "<none>")'")
            //EXC_BAD_ACCESS (code=EXC_I386_GPFLT
            // Duplicate elements of type 'ExternalApplication' were found in a Set
            tracked.insert(appRef)
        } else {
            Logger.log(message: "AXWindowServer: Error setting up tracking for app '\(appRef.bundleId ?? "<none>")")
        }
    }
    
    func deregister(app: NSRunningApplication) {
        for trackedApp in self.tracked where trackedApp.bundleId == app.bundleIdentifier {
            // EXC_BAD_ACCESS (code=EXC_I386_GPFLT)
            tracked.remove(trackedApp)
            trackedApp.deregisterObserver()
        }

    }
    
    init() {
        
       registerWindowTracking()
            
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(didActivateApplication(notification:)), name: NSWorkspace.didActivateApplicationNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(didTerminateApplication(notification:)), name: NSWorkspace.didTerminateApplicationNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(didLaunchApplicationNotification(notification:)), name: NSWorkspace.didLaunchApplicationNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(didDeactivateApplication(notification:)), name: NSWorkspace.didDeactivateApplicationNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(activeSpaceDidChange), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)
        
//        NSEvent.addGlobalMonitorForEvents(matching: .flagsChanged) { (event) in
//            if let window = WindowServer.shared.allWindows(onScreen: false).first {
//                print("AXWindowServer: flags changed; top window: \(window.bundleId ?? "<none>")")
//
//            }
//            if let app = NSWorkspace.shared.frontmostApplication {
//                print("AXWindowServer: flags changed; frontmost app \(app.bundleIdentifier ?? "<none>")")
//                //self.register(app, fromActivation: true)
//            }
//        }
     
    }
    
    func registerWindowTracking() {
        for app in tracked {
            app.deregisterObserver()
            tracked.remove(app)
        }
//        tracked = []
        
        // capture topmost app's window
        if let app = NSWorkspace.shared.frontmostApplication {
            register(app, fromActivation: true)
        }
        
        for app in NSWorkspace.shared.runningApplications {// where Integrations.whitelist.contains(app.bundleIdentifier ?? "")  {
            register(app)
        }
        
        Logger.log(message: "AXWindowServer: Tracking \(self.tracked.count) applications...")

    }
    
    
    @objc func activeSpaceDidChange() {
        // this is used to reset previous application when space is changed. Maybe should be nil.
        //self.previousApplication =
        if let app = NSWorkspace.shared.frontmostApplication {
            print("AXWindowServer: space changed - \(app.bundleIdentifier ?? "<none>")")
            //self.register(app, fromActivation: true)
        }
    }

    @objc func didDeactivateApplication(notification: NSNotification!) {
        if let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication, Integrations.whitelist.contains(app.bundleIdentifier ?? "") {
            print("AXWindowServer:", app.bundleIdentifier ?? "")
        }
        
        //self.previousApplication = notification!.userInfo![NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
    }
    
    
    @objc func didActivateApplication(notification: Notification) {
        if let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication { //Integrations.whitelist.contains(app.bundleIdentifier ?? "") {
//            self.tracked = self.tracked.filter { return $0.handler != nil && $0.observer != nil}
            print("AXWindowServer - register", app.bundleIdentifier ?? "")
            self.register(app, fromActivation: true)
        }
    }
    
    @objc func didTerminateApplication(notification: Notification) {
        if let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication { //Integrations.whitelist.contains(app.bundleIdentifier ?? "") {
//            self.tracked = self.tracked.filter { return $0.handler != nil && $0.observer != nil}
            print("AXWindowServer - terminate", app.bundleIdentifier ?? "")
            self.deregister(app: app)
        }
    }
    
    @objc func didLaunchApplicationNotification(notification: Notification) {
        if let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication { //Integrations.whitelist.contains(app.bundleIdentifier ?? "") {
//            self.tracked = self.tracked.filter { return $0.handler != nil && $0.observer != nil}
            print("AXWindowServer - launch", app.bundleIdentifier ?? "")
//            self.register(app, fromActivation: true)
        }
    }
    
    
    @objc func top() -> AXUIElement? {
        let systemWideElement: AXUIElement = AXUIElementCreateSystemWide()

        var window: AnyObject?
        let result = AXUIElementCopyAttributeValue(systemWideElement, kAXFocusedWindowAttribute as CFString, &window)
        print("hacky", result, window)
        return nil
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
