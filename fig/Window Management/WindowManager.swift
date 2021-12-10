//
//  WindowManager.swift
//  fig
//
//  Created by Matt Schrage on 6/28/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
import WebKit
import Sentry

protocol WindowManagementService {
    func tether(window: CompanionWindow)
    func untether(window: CompanionWindow)
    func close(window: CompanionWindow)
    func shouldAppear(window: CompanionWindow, explicitlyRepositioned: Bool) -> Bool
    func requestWindowUpdate()
    func isSidebar(window: CompanionWindow) -> Bool

//    func shouldReposition(window: CompanionWindow, explicitlyRepositioned: Bool) -> Bool

}

class WindowManager : NSObject {
    static let shared = WindowManager(windowService: WindowServer.shared)

    var sidebar: CompanionWindow?
    var autocomplete: CompanionWindow?
    var windows: [ExternalWindow: CompanionWindow] = [:]
    var untetheredWindows: [CompanionWindow] = []

    let windowServiceProvider: WindowService
    init(windowService: WindowService) {

        self.windowServiceProvider = windowService
        super.init()
        
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(spaceChanged), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(activateApp), name: NSWorkspace.didActivateApplicationNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(deactivateApp), name: NSWorkspace.didDeactivateApplicationNotification, object: nil)

      NotificationCenter.default.addObserver(self, selector: #selector(windowChanged(_:)), name: WindowServer.whitelistedWindowDidChangeNotification, object: nil)
        
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
        let existingWindows = Set(self.windowServiceProvider.allWhitelistedWindows(onScreen: false).map { $0.windowId })
        
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
        print("---\nSpaced Changed\n\n")
        updatePosition(for: .spaceChanged)
    }
    
    @objc func activateApp(){
        updatePosition(for: .applicationActivated)
    }
    
    static let focusedWindowChangedNotification = Notification.Name("focusedWindowChangedNotification")

    @objc func windowChanged(_ notification: Notification? = nil){
        updatePosition(for: .windowChanged)
        self.autocomplete?.maxHeight = 0
      if let notification = notification,
         let app = notification.object as? ExternalWindow,
         let bundleId = app.bundleId  {
        Autocomplete.runJavascript("fig.currentApp = '\(bundleId)'")
      }
    
        NotificationCenter.default.post(name: WindowManager.focusedWindowChangedNotification,
                                        object: notification?.object as? ExternalWindow ?? AXWindowServer.shared.topWindow)
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
    
    @objc func requestWindowUpdate() {
        self.updatePosition(for: .explictlyTriggered)
    }
    
    @objc func updatePositionTimer() {
        updatePosition(for: .timer)
    }
    
    func updatePosition(for reason: WindowUpdateReason) {
//        if ([.mouseUp, .mouseDown].contains(reason) ) { return }
        // handle tracked windows; ideally be smarter about this.
        let allCompanionWindows = Set(self.windows.map { $0.value }).union(untetheredWindows)

        let (forceUpdate, explicit) = reason.repositionParameters
        allCompanionWindows.forEach { $0.repositionWindow(forceUpdate: forceUpdate, explicit: explicit) }
       
        //sidebar
        sidebar?.repositionWindow(forceUpdate: forceUpdate, explicit: explicit)
        if (reason != .flagsChanged) {
            autocomplete?.repositionWindow(forceUpdate: forceUpdate, explicit: explicit)
        }
        
        let visibleWindows = self.windows.map { $0.value } .filter { $0.isVisible }
        print("---\nAll Fig windows: \(allCompanionWindows.count)\nVisible Fig windows: \(visibleWindows.count)\nUntethered Windows: \(self.untetheredWindows.count)\n---")
        for w in visibleWindows {
            print("\(w.title)", "\(w.positioning)")
        }
        
        
//        print(NSApp.keyWindow as? CompanionWindow)
        
        // this is needed to fix a bug when the the mouse is inside the sidebar when a fig window is closed (yes, I know this is also in the close() logic. -- Needs to be in both places to prevent flickering!)
        if (reason == .figWindowClosed) {
            if self.sidebar?.frame.contains(NSEvent.mouseLocation) ?? false {
               print("sidebar contains mouse on Window close")
                WindowManager.shared.windowServiceProvider.takeFocus()
           }
        }
        
        print(reason)
    
    }
    
    func createAutocomplete() {
        guard self.autocomplete == nil else {
          self.autocomplete?.webView?.loadAutocomplete()
          return
        }
        
        let web = WebViewController()
        web.webView?.defaultURL = nil
        
        web.webView?.loadAutocomplete()
        let companion = CompanionWindow(viewController: web)
        // prevents strange artifacts at top of autocomplete window
        // shadow can be added back in using CSS
        companion.hasShadow = false
        companion.positioning = .hidden
        companion.repositionWindow(forceUpdate: true, explicit: true)
        companion.maxHeight = 0
        companion.loaded = false
        self.autocomplete = companion
        
        
    }
    
    func newCompanionWindow() -> CompanionWindow {
        let web = WebViewController()
        web.webView?.defaultURL = nil
        let window =  CompanionWindow(viewController: web)
        window.makeKeyAndOrderFront(nil)
        return window
    }
    
    func newNativeTerminalSession(completion: (() -> Void)? = nil) {
        // if the topmost application is a terminal, create new session
        guard !Integrations.nativeTerminals.contains(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "") else {
            Logger.log(message: "New terminal session!")
            ShellBridge.simulate(keypress: .n, maskCommand: true)
            if let completion = completion {
                completion()
            }
            return
        }
        
        let running = NSWorkspace.shared.runningApplications.filter { Integrations.nativeTerminals.contains($0.bundleIdentifier ?? "")}
        
        // launch terminal (detect if iTerm is installed?)
        if (running.count == 0) {
            Logger.log(message: "term: No terminal applications are running. Open iTerm or Terminal.app.")

            if let iTerm = NSWorkspace.shared.urlForApplication(withBundleIdentifier: "com.googlecode.iterm2") {
                let app = try? NSWorkspace.shared.launchApplication(at: iTerm, options: .default, configuration: [:])
                guard app == nil  else {
                    Logger.log(message: "Opening iTerm!")
                    print("Success!")
                    if let completion = completion {
                        completion()
                    }
                    return
                }
            }
            Logger.log(message: "Opening Terminal.app!")
            NSWorkspace.shared.launchApplication("Terminal")
        } else {
            Logger.log(message: "term: \(running.count) currently running terminal(s).")
            let iTerm = running.filter { $0.bundleIdentifier == "com.googlecode.iterm2" }.first
            let target = iTerm ?? running.first!
            
            target.activate(options: .activateIgnoringOtherApps)
            Logger.log(message: "term: Activating \(target.bundleIdentifier ?? "<none>")")

            var kvo: NSKeyValueObservation? = nil
            kvo = NSWorkspace.shared.observe(\.frontmostApplication, options: [.new]) { (workspace, delta) in
                if let app = delta.newValue, let bundleId = app?.bundleIdentifier, Integrations.nativeTerminals.contains(bundleId) {
                    Logger.log(message: "term: Openning new window in \(target.bundleIdentifier ?? "<none>")")
                    ShellBridge.simulate(keypress: .n, maskCommand: true)
                    kvo?.invalidate()
                    if let completion = completion {
                        completion()
                    }
                }
            }
        }
    }

    func bringTerminalWindowToFront() {
        let terminalWindows = self.windowServiceProvider.allWhitelistedWindows(onScreen: true).filter { Integrations.terminals.contains($0.bundleId ?? "") }
        
        if (terminalWindows.count == 0) {
            NSWorkspace.shared.launchApplication("Terminal")
        } else {
            let target = terminalWindows.first!
            NSRunningApplication(processIdentifier: target.app.processIdentifier)?.activate(options: .activateIgnoringOtherApps)
        }
    }
  
    func companionWindowForWindowId(_ id: CGWindowID) -> CompanionWindow {
        let pair = (self.windows.filter { $0.key.windowId == id}).first!
        return pair.value
    }
}

extension WindowManager : WindowManagementService {
    func isSidebar(window: CompanionWindow) -> Bool {
        guard let sidebar = self.sidebar else {
            return false
        }
        return window == sidebar
    }
    
    func shouldAppear(window: CompanionWindow, explicitlyRepositioned: Bool) -> Bool {
        window.configureWindow(for: window.positioning)
        
        if !Defaults.shared.loggedIn {
            print("shouldAppear: Not logged in")
            return false
        }
        
        if window.isSidebar && UserDefaults.standard.string(forKey: "sidebar") == "hidden" {
            print("shouldAppear: Is sidebar and sidebar preference is hidden.")
            return false
        }
        
        if !window.isDocked {
            print("shouldAppear: Is untethered")
            return true
        }
        
        if let keyWindow = NSApp.keyWindow as? CompanionWindow, untetheredWindows.contains(keyWindow) {
            print("shouldAppear: current keywindow is undocked")
            return false
        }

        if window.isAutocompletePopup {
            guard !Defaults.shared.debugAutocomplete else {
                return true
            }
            if let max = window.maxHeight, max == 0 {
                print("shouldAppear: autocomplete should be hidden")
                return false
            } else {
                print("shouldAppear: autocomplete should be shown")
                // false (creates control + click disappearance)
                // true introduces flicker
                return false
            }

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
//            if (explicitlyRepositioned) {
                guard let targetWindow = self.windowServiceProvider.previousWhitelistedWindow() else {
                    print("shouldAppear: [\(bundleId)] previousTargetWindow ??")
                    // prevents sidebar from appearing in addition to other views
//                    guard let key = NSApplication.shared.keyWindow as? CompanionWindow else {
//                        return window.isKeyWindow
//                     || (key.positioning == .outsideRight && window.isSidebar)
//                    }
//                    if let key = NSApplication.shared.keyWindow as? CompanionWindow, key.positioning == .outsideRight {
//                        print("shouldAppear: [\(bundleId)] window is sidebar, and underneath pos3")
//                        return window.isSidebar || window.isKeyWindow
//                    }
                    
                    return true || window.isKeyWindow
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
//            }
            
            // this keeps Fig windows open by default when Fig is the active apps,
            // which makes sense most of the time
            // but there are some issues here. Probably need a condition tying the current parentWindow id to previous whitelisted window.
            //print("shouldAppear: [\(bundleId)] Fig active & not explicitly positioned")
            //return false
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
        (window.contentViewController as? WebViewController)?.cleanUp()
        window.orderOut(nil)
        window.close()
        
        if let parent = window.tetheredWindow, window.isDocked, self.windows[parent] != nil {
            self.windows.removeValue(forKey: parent)
            self.updatePosition(for: .figWindowClosed)
            
            if (NSWorkspace.shared.frontmostApplication?.isFig ?? false) {
                // fixes bug where if fig window was closed and mouse was located over sidebar, sidebar did not become active.
                if self.sidebar?.frame.contains(NSEvent.mouseLocation) ?? false {
                    print("sidebar contains mouse on Window close")
                  //WindowManager.shared.windowServiceProvider.takeFocus()
                } else {
                    self.windowServiceProvider.previousFrontmostApplication()?.activate(options: .activateIgnoringOtherApps)
                }

            }
        } else {
            self.untetheredWindows = self.untetheredWindows.filter { $0 != window }
        }
        
        
    }
    
    func untether(window: CompanionWindow) {
        window.makeKeyAndOrderFront(self)
        if let parent = window.tetheredWindow {
            self.windows.removeValue(forKey: parent)
        }
        
        self.untetheredWindows.append(window)
        
        self.updatePosition(for: .figWindowUntethered)
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
    
  func positionAutocompletePopover(textRect: CGRect?, makeVisibleImmediately: Bool = true, completion: (() -> Void)? = nil) {
        if let rect = textRect, let window = AXWindowServer.shared.whitelistedWindow {
          
            // get 'true' main screen (accounting for the fact that fullScreen workspaces default to laptop screen)
            let currentScreen = NSScreen.screens.filter { (screen) -> Bool in
            return screen.frame.contains(rect)
            }.first ?? NSScreen.main
            let heightLimit: CGFloat = Settings.shared.getValue(forKey: Settings.autocompleteHeight) as? CGFloat ?? 140.0 

            let maxWidth =  Settings.shared.getValue(forKey: Settings.autocompleteWidth) as? CGFloat


            if (Defaults.shared.debugAutocomplete) {
                WindowManager.shared.autocomplete?.maxHeight = heightLimit
                WindowManager.shared.autocomplete?.backgroundColor = .red
              
                if WindowManager.shared.autocomplete?.width == 0 {
                  WindowManager.shared.autocomplete?.width = 1
                }
                Logger.log(message: "Note: Debug mode is enabled!", subsystem: .positioning)
            }

            let positioning = WindowPositioning.frameRelativeToCursor(currentScreenFrame: currentScreen?.frame ?? .zero,
                                                currentWindowFrame: window.frame,
                                                cursorRect: rect,
                                                width: WindowManager.shared.autocomplete?.width ?? maxWidth ?? Defaults.shared.autocompleteWidth ?? 200,
                                                height: WindowManager.shared.autocomplete?.maxHeight ?? 0,
                                                anchorOffset: WindowManager.shared.autocomplete?.anchorOffsetPoint ?? .zero,
                                                maxHeight: heightLimit)
            Logger.log(message: "New calculation: \(positioning.frame)", subsystem: .positioning)

            DispatchQueue.main.async {
                WindowManager.shared.autocomplete?.tetheredWindow = window
                WindowManager.shared.autocomplete?.setOverlayFrame(positioning.frame, makeVisible: makeVisibleImmediately)//140
                completion?()

            }
            
        } else {
            // workaround
            DispatchQueue.main.async {
                WindowManager.shared.autocomplete?.orderOut(nil)
                completion?()
            }
        }
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
