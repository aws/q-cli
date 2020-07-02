//
//  WindowManager.swift
//  fig
//
//  Created by Matt Schrage on 6/28/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

protocol WindowManagementService {
    func tether(window: CompanionWindow)
    func untether(window: CompanionWindow)
    func close(window: CompanionWindow)
    func shouldAppear(window: CompanionWindow, explicitlyRepositioned: Bool) -> Bool
//    func shouldReposition(window: CompanionWindow, explicitlyRepositioned: Bool) -> Bool

}

class WindowManager : NSObject {
    static let shared = WindowManager(windowService: WindowServer.shared)
    var hotKeyManager: HotKeyManager?

    var sidebar: CompanionWindow?
    var windows: [ExternalWindow: CompanionWindow] = [:]
    var untetheredWindows: [CompanionWindow] = []

    let windowServiceProvider: WindowService
    init(windowService: WindowService) {

        self.windowServiceProvider = windowService
        super.init()

        NotificationCenter.default.addObserver(self, selector: #selector(recievedDataFromPipe(_:)), name: .recievedDataFromPipe, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(recievedStdoutFromTerminal(_:)), name: .recievedStdoutFromTerminal, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(recievedUserInputFromTerminal(_:)), name: .recievedUserInputFromTerminal, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(recievedDataFromPty(_:)), name: .recievedDataFromPty, object: nil)
        
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(spaceChanged), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(activateApp), name: NSWorkspace.didActivateApplicationNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(deactivateApp), name: NSWorkspace.didDeactivateApplicationNotification, object: nil)

        NotificationCenter.default.addObserver(self, selector: #selector(windowChanged), name: WindowServer.whitelistedWindowDidChangeNotification, object: nil)

        
//        _ = Timer.scheduledTimer(timeInterval: 0.25, target: self, selector: #selector(updatePositionTimer), userInfo: nil, repeats: true)
        
            //var mouseDownPolling: Timer?
            NSEvent.addGlobalMonitorForEvents(matching: .leftMouseDown) { (event) in
                self.updatePosition(for: .mouseDown)
            }

            NSEvent.addGlobalMonitorForEvents(matching: .leftMouseDragged) { (event) in
                self.updatePosition(for: .mouseDown)
            }

            NSEvent.addGlobalMonitorForEvents(matching: .leftMouseUp) { (event) in
                self.updatePosition(for: .mouseUp)
            }
        
            NSEvent.addGlobalMonitorForEvents(matching: .flagsChanged) { (event) in
                self.updatePosition(for: .flagsChanged)
            }
        
        _ = Timer.scheduledTimer(timeInterval: 60, target: self, selector: #selector(cleanup), userInfo: nil, repeats: true)


       }
    
    @objc func cleanup() {
        let existingWindows = Set(self.windowServiceProvider.allWhitelistedWindows().map { $0.windowId })
        
        print("Existing Windows:\(existingWindows.count), Tracked Windows: \(self.windows.count)")
               
        let zombieWindows = self.windows.filter { !existingWindows.contains($0.key.windowId) }
        zombieWindows.forEach { $0.value.close() }
        
        self.windows = self.windows.filter { existingWindows.contains($0.key.windowId) }
        print("Tracked windows after cleaning: \(self.windows.count)")

    }
    
    @objc func spaceChanged() {
        if let controller = self.sidebar?.contentViewController as? WebViewController,
            let webview = controller.webView {
            webview.trackMouse = false;
            // wait for window reposition to take effect before handling mouse events
            // This fixes a bug when the user changes spaces but their mouse remains in the companion window
            Timer.delayWithSeconds(0.2) {
                webview.trackMouse = true;

            }
        }
        updatePosition(for: .spaceChanged)
    }
    
    @objc func activateApp(){
        updatePosition(for: .applicationActivated)
    }
    
    @objc func windowChanged(){
        updatePosition(for: .windowChanged)
    }
    
    @objc func deactivateApp(){
        updatePosition(for: .applicationDeactivated)
    }
    
    enum WindowUpdateReason {
        case timer
        case spaceChanged
        case applicationActivated
        case applicationDeactivated
        case windowChanged
        case figWindowClosed
        case figWindowTethered
        case figWindowUntethered
        case explictlyTriggered
        case mouseDown
        case mouseUp
        case flagsChanged
        case becomeKeyWindow
        case resignKeyWindow

        
        var repositionParameters: (Bool, Bool) {
            get {
                switch self {
                case .timer, .mouseDown, .flagsChanged, .becomeKeyWindow, .resignKeyWindow:
                    return (false, false)
                case .applicationActivated, .applicationDeactivated, .mouseUp:
                    return (true, false)
                case .windowChanged:
                    return (true, false)
                case .explictlyTriggered, .figWindowClosed, .spaceChanged, .figWindowUntethered, .figWindowTethered:
                    return (true, true)

                }
                
            }
        }
    }
    
    @objc func updatePositionTimer() {
        updatePosition(for: .timer)
    }
    
    func updatePosition(for reason: WindowUpdateReason) {
//        if (reason == .timer) { return }
        // handle tracked windows; ideally be smarter about this.
        let allCompanionWindows = Set(self.windows.map { $0.value }).union(untetheredWindows)

        let (forceUpdate, explicit) = reason.repositionParameters
        allCompanionWindows.forEach { $0.repositionWindow(forceUpdate: forceUpdate, explicit: explicit) }
       
        //sidebar
        sidebar?.repositionWindow(forceUpdate: forceUpdate, explicit: explicit)
        
        let visibleWindows = self.windows.filter { $0.value.isVisible }
        print("Visible Fig windows: \(visibleWindows.count)")
        for w in visibleWindows {
            print("\(w.value.title)", "\(w.value.positioning)")
        }
        self.hotKeyManager?.companionWindow = visibleWindows.first?.value ?? self.sidebar!

//        if let keyWindow = NSApp.keyWindow as? CompanionWindow, untetheredWindows.contains(keyWindow) {
//            
//        } else {
//            self.hotKeyManager?.companionWindow = visibleWindows.first?.value ?? self.sidebar!
//        }
    
    }
    
    func createSidebar() {
        let web = WebViewController()
        web.webView?.defaultURL = nil
        web.webView?.loadRemoteApp(at: URL(string: "https://app.withfig.com/sidebar")!)
        let companion = CompanionWindow(viewController: web)
        companion.positioning = CompanionWindow.defaultPassivePosition
        companion.repositionWindow(forceUpdate: true, explicit: true)
        self.hotKeyManager = HotKeyManager(companion:companion)
        self.sidebar = companion
        
    }
    
    func newCompanionWindow() -> CompanionWindow {
        let web = WebViewController()
        web.webView?.defaultURL = nil
        let window =  CompanionWindow(viewController: web)
        window.makeKeyAndOrderFront(nil)
        return window
    }
}

extension WindowManager : ShellBridgeEventListener {
    @objc func recievedDataFromPty(_ notification: Notification) {
      
    }
    
    @objc func recievedUserInputFromTerminal(_ notification: Notification) {
    }
    
    @objc func recievedStdoutFromTerminal(_ notification: Notification) {
    
    }
    
    @objc func recievedDataFromPipe(_ notification: Notification) {
        NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
        
        let msg = (notification.object as! ShellMessage)
        DispatchQueue.main.async {
            if let parent = self.windowServiceProvider.topmostWhitelistedWindow() {
                
                if let companion = self.windows[parent], let web = companion.contentViewController as? WebViewController {
                    self.windows[parent] = companion
                    companion.tetheredWindowId = parent.windowId
                    companion.tetheredWindow = parent
                    companion.delegate = self

                    FigCLI.route(msg, webView: web.webView!, companionWindow: companion)
                    companion.oneTimeUse = true

                } else {
                    let web = WebViewController()
                    web.webView?.defaultURL = nil

                    let companion = self.windows[parent] ?? CompanionWindow(viewController: web)
                    self.windows[parent] = companion
                    companion.tetheredWindowId = parent.windowId
                    companion.tetheredWindow = parent
                    companion.delegate = self

                    companion.makeKeyAndOrderFront(nil)

                    FigCLI.route(msg, webView: web.webView!, companionWindow: companion)
                    companion.oneTimeUse = true 

                    
                }
            }
        }
    }
    
    func companionWindowForWindowId(_ id: CGWindowID) -> CompanionWindow {
        let pair = (self.windows.filter { $0.key.windowId == id}).first!
        return pair.value
    }
}

extension WindowManager : WindowManagementService {
    
    func shouldAppear(window: CompanionWindow, explicitlyRepositioned: Bool) -> Bool {
        window.configureWindow(for: window.positioning)

        if !window.isDocked {
            print("shouldAppear: Is untethered")
            return true
        }
        
        if let keyWindow = NSApp.keyWindow as? CompanionWindow, untetheredWindows.contains(keyWindow) {
            print("shouldAppear: current keywindow is undocked")
            return false
        }
        
        let whitelistedBundleIds = Integrations.whitelist
        guard let app = NSWorkspace.shared.frontmostApplication,
              let bundleId = app.bundleIdentifier else {
                   print("shouldAppear: app or bundle ??")
                   return false
               }


        if (whitelistedBundleIds.contains(bundleId)) {
            guard let targetWindow = WindowServer.shared.topmostWhitelistedWindow() else {
                print("shouldAppear: [\(bundleId)] targetWindow ??")
                return false
            }

            if window.tetheredWindow?.windowId == targetWindow.windowId {
                print("shouldAppear: [\(bundleId)] Companion tethered to active window")
                return true // Companion is tethered to activeWindow
            } else if (self.windows[targetWindow] == nil && window == self.sidebar) {
                print("shouldAppear: [\(bundleId)] Sidebar connected to active window")
                return true
            } else {
                print("shouldAppear: [\(bundleId)] Hide")
                return false
            }
            
        } else if (app.isFig) {
            if (explicitlyRepositioned) {
                guard let targetWindow = self.windowServiceProvider.previousWhitelistedWindow() else {
                    print("shouldAppear: [\(bundleId)] previousTargetWindow ??")
                    return true
                }
                
                if window.tetheredWindow?.windowId == targetWindow.windowId {
                    print("shouldAppear: [\(bundleId)] Companion tethered to previous active window")
                    return true // Companion is tethered to activeWindow
                } else if (self.windows[targetWindow] == nil && window == self.sidebar) {
                   print("shouldAppear: [\(bundleId)] Sidebar connected to previous active window")
                    return true
                } else {
                    print("shouldAppear: [\(bundleId)] Hide")
                    return false
                }
            }
            
            print("shouldAppear: [\(bundleId)] Fig active & not explicitly positioned")
            return true
        } else {
            print("shouldAppear: [\(bundleId)] Not on whitelist")
            return false
        }
    }
    
    
//    func shouldAppear(window: CompanionWindow, explicitlyRepositioned: Bool) -> Bool {
//        print("ShouldAppear: \(window.positioning) - \(window.title) - \(window.tetheredWindow?.title ?? "")")
//
//        if !Set(windows.values).union(untetheredWindows).union((self.sidebar != nil) ? [self.sidebar!] : []).contains(window) {
//            print("Zombie Window! \(window.positioning)")
//            print("\(NSApp.windows.count-1) ?= \(self.windows.count + 1 + self.untetheredWindows.count)")
//            print("\(NSApp.windows.contains(window))")
//            return false
//        }
//
//        if !window.isDocked {
//            return true
//        }
//
//        if let external = WindowServer.shared.topmostWhitelistedWindow() {
//
//            // Only show the fig window associated with the current external window
//            if let tetheredWindow = window.tetheredWindow,
//                   tetheredWindow.windowId != external.windowId {
//                return false
//            }
//
//            // Hide sidebar if there is an active fig window associated with the current external window
//            if let sidebar = self.sidebar, window == sidebar && (self.windows[external] != nil)  {
//                return false
//            }
//
//            return true
//        }
//
//        if (NSWorkspace.shared.frontmostApplication?.isFig ?? false), explicitlyRepositioned {
//            // check if window corresponds to previous application
//            if let external = self.windowServiceProvider.previousWhitelistedWindow() {
//                print("previousWindow = \(external.bundleId ?? "") \(external.title ?? "")")
//                if let tetheredWindow = window.tetheredWindow,
//                    tetheredWindow.windowId != external.windowId {
//                    return false
//                }
//
//                if let sidebar = self.sidebar, window == sidebar && (self.windows[external] != nil)  {
//                    return false
//                }
//
//                return true
//
//            }
//
//        } else {
//            return true
//        }
//
//        return false
//    }
    
    func close(window: CompanionWindow) {
        window.orderOut(nil)
        window.close()
        
        if let parent = window.tetheredWindow, self.windows[parent] != nil {
            self.windows.removeValue(forKey: parent)
            self.updatePosition(for: .figWindowClosed)
            
            if (NSWorkspace.shared.frontmostApplication?.isFig ?? false) {
                self.windowServiceProvider.previousFrontmostApplication()?.activate(options: .activateIgnoringOtherApps)
            }
        }
        
        self.untetheredWindows = self.untetheredWindows.filter { $0 != window }
        
    }
    
    func untether(window: CompanionWindow) {
        if let parent = window.tetheredWindow {
            self.windows.removeValue(forKey: parent)
            self.untetheredWindows.append(window)
        }
        
//        self.updatePosition(for: .figWindowUntethered)
    }
    
    func tether(window: CompanionWindow) {
        guard let parent = window.tetheredWindow else { return }
        
        //self.windowServiceProvider.bringToFront(window: parent)
        
        if let replacement = self.windows[parent] {
            self.close(window: replacement)
        }
        
        self.windows[parent] = window
        self.updatePosition(for: .figWindowTethered)
    }
}

extension WindowManager : NSWindowDelegate {
    func windowDidBecomeKey(_ notification: Notification) {
        self.updatePosition(for: .becomeKeyWindow)
    }
    
    func windowDidResignKey(_ notification: Notification) {
        self.updatePosition(for: .resignKeyWindow)

    }
}
