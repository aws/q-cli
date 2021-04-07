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
    var hotKeyManager: HotKeyManager?

    var sidebar: CompanionWindow?
    var autocomplete: CompanionWindow?
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
//            NSEvent.addGlobalMonitorForEvents(matching: .any) { (event) in
//                //self.updatePosition(for: .mouseDown)
//                print(event)
//            }
        
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
    
    @objc func windowChanged(){
        updatePosition(for: .windowChanged)
        self.autocomplete?.maxHeight = 0
//
//        DispatchQueue.main.async {
//            self.autocomplete?.orderOut(nil)
//        }

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

        if let keyWindow = NSApp.keyWindow as? CompanionWindow, untetheredWindows.contains(keyWindow) {
            self.hotKeyManager?.companionWindow = keyWindow
        } else if let sidebar = self.sidebar {
            self.hotKeyManager?.companionWindow = visibleWindows.first ?? sidebar
        }
        
        print(reason)
    
    }
    
    func createSidebar() {
        
        if let sidebar = self.sidebar {
            sidebar.close()
            self.sidebar = nil
        }
        
        let web = WebViewController()
        web.webView?.defaultURL = nil
        web.webView?.loadRemoteApp(at: Remote.baseURL.appendingPathComponent("sidebar"))
        let companion = CompanionWindow(viewController: web)
        companion.positioning = CompanionWindow.defaultPassivePosition
        companion.repositionWindow(forceUpdate: true, explicit: true)
        self.hotKeyManager = HotKeyManager(companion:companion)
        self.sidebar = companion
        
    }
    
    func createAutocomplete() {
        if let autocomplete = self.autocomplete {
            autocomplete.webViewController?.pty.close()
            autocomplete.orderOut(nil)
            self.autocomplete = nil
        }
        
        let web = WebViewController()
        web.webView?.defaultURL = nil
//        web.webView?.loadBundleApp("tutorial")
        
        web.webView?.loadAutocomplete()
        let companion = CompanionWindow(viewController: web)
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
            ShellBridge.simulate(keypress: .cmdN)
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
                    ShellBridge.simulate(keypress: .cmdN)
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
}

extension WindowManager : ShellBridgeEventListener {
    func shellPromptWillReturn(_ notification: Notification) {
        
    }
    
    func startedNewTerminalSession(_ notification: Notification) {
        
    }
    
    func currentTabDidChange(_ notification: Notification) {
        
    }
    
    @objc func currentDirectoryDidChange(_ notification: Notification) {
        
    }
    
    @objc func recievedDataFromPty(_ notification: Notification) {
      
    }
    
    @objc func recievedUserInputFromTerminal(_ notification: Notification) {
    }
    
    @objc func recievedStdoutFromTerminal(_ notification: Notification) {
    
    }
    
    @objc func recievedDataFromPipe(_ notification: Notification) {
        // Prevent windows from being launched from CLI if the user hasn't signed in
        guard Defaults.email != nil else {
            SentrySDK.capture(message: "Attempting to run CLI command before signup")
            Logger.log(message: "Attempting to run CLI command before signup")
            return
        }
        
//        NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
        
        let msg = (notification.object as! ShellMessage)
        
        // don't create new windows for background events
        guard !(msg.options?.first ?? "").hasPrefix("bg:") else {
            print("Handing background event elsewhere")
            return
        }
        
        DispatchQueue.main.async {

            if let parent = self.windowServiceProvider.topmostWhitelistedWindow() {
                
//                if let nativeCommand = NativeCLICommand(rawValue: msg.options?.first ?? ""), !nativeCommand.openInNewWindow {
//                    
//                }

                
                if let companion = self.windows[parent], let web = companion.contentViewController as? WebViewController {
                    self.windows[parent] = companion
//                    companion.tetheredWindowId = parent.windowId
                    companion.tetheredWindow = parent
                    companion.delegate = self
                    companion.sessionId = msg.session

                    FigCLI.route(msg, webView: web.webView!, companionWindow: companion)
                    companion.oneTimeUse = true
                    
                    if (companion.isVisible) {
                        WindowServer.shared.takeFocus()
                    }

                } else {
                    let web = WebViewController()
                    web.webView?.defaultURL = nil

                    let companion = self.windows[parent] ?? CompanionWindow(viewController: web)
                    self.windows[parent] = companion
//                    companion.tetheredWindowId = parent.windowId
                    companion.tetheredWindow = parent
                    companion.delegate = self
                    companion.sessionId = msg.session
                    
                    companion.makeKeyAndOrderFront(nil)

                    FigCLI.route(msg, webView: web.webView!, companionWindow: companion)
                    companion.oneTimeUse = true 
                    
                    if (companion.isVisible) {
                        WindowServer.shared.takeFocus()
                    }
                }
            } else {
                // check accessibility permissions
                SentrySDK.capture(message: "Notify Accesibility Error in CLI")

                FigCLI.notifyAccessibilityError(msg)
            }
        }
    }
    
    func companionWindowForWindowId(_ id: CGWindowID) -> CompanionWindow {
        let pair = (self.windows.filter { $0.key.windowId == id}).first!
        return pair.value
    }
}

// Open WindowManager
extension WindowManager {
    func openCompanionWindow(from shell: ShellMessage) {
        if let parent = self.windowServiceProvider.topmostWhitelistedWindow() {
             
             if let companion = self.windows[parent], let web = companion.contentViewController as? WebViewController {
                 self.windows[parent] = companion
//                 companion.tetheredWindowId = parent.windowId
                 companion.tetheredWindow = parent
                 companion.delegate = self

                 FigCLI.route(shell, webView: web.webView!, companionWindow: companion)
                 companion.oneTimeUse = true

             } else {
                 let web = WebViewController()
                 web.webView?.defaultURL = nil

                 let companion = self.windows[parent] ?? CompanionWindow(viewController: web)
                 self.windows[parent] = companion
//                 companion.tetheredWindowId = parent.windowId
                 companion.tetheredWindow = parent
                 companion.delegate = self

                 companion.makeKeyAndOrderFront(nil)

                 FigCLI.route(shell, webView: web.webView!, companionWindow: companion)
                 companion.oneTimeUse = true

                 
             }
        }
    }
    
    func popupCompanionWindow(from navigationAction: WKNavigationAction, with configuration: WKWebViewConfiguration, frame: NSRect) -> WKWebView? {

        
        let web = WebViewController(configuration)
        web.webView?.load(navigationAction.request)
        web.webView?.defaultURL = nil

        let companion = CompanionWindow(viewController: web)

//        companion.tetheredWindowId = parent.windowId
//        companion.tetheredWindow = parent

        
//        companion.setFrame(C, display: true, animate: false)
        companion.oneTimeUse = true
//        companion.isDocked = false
        companion.configureWindow(for: CompanionWindow.defaultActivePosition)
        companion.positioning = CompanionWindow.defaultActivePosition
//        self.untether(window: companion)
        companion.makeKeyAndOrderFront(nil)
        companion.orderFrontRegardless()
        companion.delegate = self

        
        return web.webView

                    
        
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
        
        if !Defaults.loggedIn {
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
            guard !Defaults.debugAutocomplete else {
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
        (window.contentViewController as? WebViewController)?.pty.close()
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
          let heightLimit: CGFloat = Settings.shared.getValue(forKey: Settings.autocompleteHeight) as? CGFloat ?? 140.0 //300.0//
            
          let isAbove = window.frame.height < window.frame.origin.y - rect.origin.y + rect.height + heightLimit
                        && rect.origin.y + heightLimit <= NSScreen.main?.frame.maxY ?? 0.0
                        // *visor* I'm not sure what the second conditional is for...


            
            let height:CGFloat = isAbove ? 0 : heightLimit
            let translatedOrigin = isAbove ? NSPoint(x: rect.origin.x, y: rect.origin.y + height + 5) :
                                             NSPoint(x: rect.origin.x, y: rect.origin.y - rect.height - 5) //below
            
            
            // Prevent arrow keys
            if ((WindowManager.shared.autocomplete?.maxHeight != 0)) {
                KeypressProvider.shared.addRedirect(for: Keycode.upArrow, in: window)
                KeypressProvider.shared.addRedirect(for: Keycode.downArrow, in: window)
                KeypressProvider.shared.addRedirect(for: Keycode.tab, in: window)
                if (!Defaults.onlyInsertOnTab) {
                    KeypressProvider.shared.addRedirect(for: Keycode.returnKey, in: window)
                }
                KeypressProvider.shared.addRedirect(for: Keystroke(modifierFlags: [.control], keyCode: Keycode.n), in: window)
                KeypressProvider.shared.addRedirect(for: Keystroke(modifierFlags: [.control], keyCode: Keycode.p), in: window)


            } else {
                KeypressProvider.shared.removeRedirect(for: Keycode.upArrow, in: window)
                KeypressProvider.shared.removeRedirect(for: Keycode.downArrow, in: window)
                KeypressProvider.shared.removeRedirect(for: Keycode.returnKey, in: window)
                KeypressProvider.shared.removeRedirect(for: Keycode.tab, in: window)
                KeypressProvider.shared.removeRedirect(for: Keystroke(modifierFlags: [.control], keyCode: Keycode.n), in: window)
                KeypressProvider.shared.removeRedirect(for: Keystroke(modifierFlags: [.control], keyCode: Keycode.p), in: window)

            }
            
//            WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try { fig.autocomplete_above = \(isAbove)} catch(e) {}", completionHandler: nil)
            let maxWidth =  Settings.shared.getValue(forKey: Settings.autocompleteWidth) as? CGFloat
            let popup = NSRect(origin: translatedOrigin, size: CGSize(width: WindowManager.shared.autocomplete?.width ?? maxWidth ?? Defaults.autocompleteWidth ?? 200
                , height: height))
            let sidebarInsetBuffer:CGFloat = 0.0//60;
            let w = (NSScreen.main!.frame.maxX - sidebarInsetBuffer) - popup.maxX
            var x = popup.origin.x
            print("edge",w, x, x + w)

            if (w < 0) {
               x += w
            }
            
            if (Defaults.debugAutocomplete) {
                WindowManager.shared.autocomplete?.maxHeight = heightLimit//140
            }
          
            DispatchQueue.main.async {
              WindowManager.shared.autocomplete?.tetheredWindow = window
              WindowManager.shared.autocomplete?.setOverlayFrame(NSRect(x: x, y: popup.origin.y, width: popup.width, height: height), makeVisible: makeVisibleImmediately)//140
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
