//
//  AppDelegate.swift
//  fig
//
//  Created by Matt Schrage on 4/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
import Sparkle
import WebKit

@NSApplicationMain
class AppDelegate: NSObject, NSApplicationDelegate,NSWindowDelegate {

    var window: NSWindow!
    var onboardingWindow: OnboardingWindow!
    var statusBarItem: NSStatusItem!
    var clicks:Int = 6;
    var hotKeyManager: HotKeyManager?
    let updater = SUUpdater.shared()
    let processPool = WKProcessPool()
    
    func applicationDidFinishLaunching(_ aNotification: Notification) {
//        AppMover.moveIfNecessary()
        let _ = ShellBridge.shared
        let _ = WindowManager.shared
        
        
        
//        updater?.checkForUpdateInformation()
        updater?.delegate = self as SUUpdaterDelegate;
//        updater?.checkForUpdateInformation()
        
//        let domain = Bundle.main.bundleIdentifier!
//        UserDefaults.standard.removePersistentDomain(forName: domain)
//        UserDefaults.standard.synchronize()
//        WebView.deleteCache()
        Defaults.build = .production
        
        let hasLaunched = UserDefaults.standard.bool(forKey: "hasLaunched")
        let email = UserDefaults.standard.string(forKey: "userEmail")

        if (!hasLaunched || email == nil ) {
            let onboardingViewController = WebViewController()
            onboardingViewController.webView?.defaultURL = nil
            onboardingViewController.webView?.loadBundleApp("landing")
//            onboardingViewController.webView?.loadRemoteApp(at: URL(string: "https://app.withfig.com/onboarding/landing.html")!)

            onboardingWindow = OnboardingWindow(viewController: onboardingViewController)
            onboardingWindow.makeKeyAndOrderFront(nil)
            onboardingWindow.setFrame(NSRect(x: 0, y: 0, width: 590, height: 480), display: true, animate: false)
            onboardingWindow.center()
            onboardingWindow.makeKeyAndOrderFront(self)
            
            UserDefaults.standard.set(true, forKey: "hasLaunched")
            UserDefaults.standard.synchronize()
        } else {
            if (!AXIsProcessTrustedWithOptions(nil)) {
                let enable = self.dialogOKCancel(question: "Enable Accesibility Permission?", text: "Fig needs this permission in order to connect to your terminal window.\n\nYou may need to toggle the setting in order for MacOS to update it.\n\nThis can occur when Fig is updated. If you are seeing this more frequently, get in touch with matt@withfig.com.", prompt: "Enable")
                
                if (enable) {
                    ShellBridge.promptForAccesibilityAccess()
                }
                
            }
            
            if (!FileManager.default.fileExists(atPath: "/usr/local/bin/fig")) {
                let enable = self.dialogOKCancel(question: "Install Fig CLI Tool?", text: "It looks like you haven't installed the Fig CLI tool. Fig doesn't work without it.")
                              
                  if (enable) {
                      ShellBridge.symlinkCLI()
                  }
            }
//            updater?.installUpdatesIfAvailable()
            self.setupCompanionWindow()
        }
        
        let statusBar = NSStatusBar.system
        statusBarItem = statusBar.statusItem(
               withLength: NSStatusItem.squareLength)
           statusBarItem.button?.title = "🍐"
           
           let statusBarMenu = NSMenu(title: "fig")
           statusBarItem.menu = statusBarMenu
           
//           statusBarMenu.addItem(
//               withTitle: "Send string",
//               action: #selector(AppDelegate.pasteStringToTerminal),
//               keyEquivalent: "")
//
//            statusBarMenu.addItem(
//            withTitle: "Check Windows",
//            action: #selector(AppDelegate.checkWinows),
//            keyEquivalent: "")
//
//            statusBarMenu.addItem(
//             withTitle: "Frontmost App",
//             action: #selector(AppDelegate.frontmostApplication),
//             keyEquivalent: "")
//
//            statusBarMenu.addItem(
//             withTitle: "Send string if active",
//             action: #selector(AppDelegate.sendStringIfTerminalActive),
//             keyEquivalent: "")
//
//            statusBarMenu.addItem(
//             withTitle: "Copy 'Helloworld' to Pastboard",
//             action: #selector(AppDelegate.copyToPasteboard),
//             keyEquivalent: "")
//
//        statusBarMenu.addItem(
//         withTitle: "Run 'script -q -t 0 <file>.fig' as User",
//         action: #selector(AppDelegate.runScriptCmd),
//         keyEquivalent: "")
//
//        statusBarMenu.addItem(
//         withTitle: "Run 'tail -F <file>.fig' as App",
//         action: #selector(AppDelegate.runTailCmd),
//         keyEquivalent: "")
//
//        statusBarMenu.addItem(
//         withTitle: "Run 'exit' as User",
//         action: #selector(AppDelegate.runExitCmd),
//         keyEquivalent: "")
//
//        statusBarMenu.addItem(
//         withTitle: "Log all window",
//         action: #selector(AppDelegate.allWindows),
//         keyEquivalent: "")
//
//        statusBarMenu.addItem(
//         withTitle: "Top Terminal Window Bounds",
//         action: #selector(AppDelegate.getTopTerminalWindow),
//         keyEquivalent: "")
//
//        statusBarMenu.addItem(
//         withTitle: "Update Overlay Style",
//         action: #selector(AppDelegate.updateOverlayStyle),
//         keyEquivalent: "")
//
        
        
//        statusBarMenu.addItem(
//         withTitle: "Kill WebSocket Server",
//         action: #selector(AppDelegate.killSocketServer),
//         keyEquivalent: "")
    
//        statusBarMenu.addItem(
//         withTitle: "Bring Terminal Window",
//         action: #selector(AppDelegate.terminalWindowToFront),
//         keyEquivalent: "")
        statusBarMenu.addItem(
         withTitle: "Add CLI Tool",
         action: #selector(AppDelegate.addCLI),
         keyEquivalent: "")
        statusBarMenu.addItem(
         withTitle: "Prompt for Accesibility Access",
         action: #selector(AppDelegate.promptForAccesibilityAccess),
         keyEquivalent: "")
        statusBarMenu.addItem(
         withTitle: "Toggle Visibility",
         action: #selector(AppDelegate.toggleVisibility),
         keyEquivalent: "i")
        statusBarMenu.addItem(NSMenuItem.separator())
        statusBarMenu.addItem(
         withTitle: "Check for updates...",
         action: #selector(AppDelegate.checkForUpdates),
         keyEquivalent: "")
        statusBarMenu.addItem(
         withTitle: "Toggle Sidebar",
         action: #selector(AppDelegate.hide),
         keyEquivalent: "")
        statusBarMenu.addItem(
         withTitle: "Quit Fig",
         action: #selector(AppDelegate.quit),
         keyEquivalent: "")
//        statusBarMenu.addItem(
//         withTitle: "Add Observer",
//         action: #selector(AppDelegate.addAccesbilityObserver),
//         keyEquivalent: "")
        
        toggleLaunchAtStartup()
        
    }
    
    func dialogOKCancel(question: String, text: String, prompt:String = "OK") -> Bool {
        let alert = NSAlert()
        alert.messageText = question
        alert.informativeText = text
        alert.alertStyle = .warning
        alert.addButton(withTitle: prompt)
        alert.addButton(withTitle: "Not now")
        return alert.runModal() == .alertFirstButtonReturn
    }

    
    func setupCompanionWindow() {
        WindowManager.shared.createSidebar()
        //let companion = CompanionWindow(viewController: WebViewController())
        //companion.positioning = CompanionWindow.defaultPassivePosition
        //window = companion

        //(window as! CompanionWindow).repositionWindow(forceUpdate: true, explicit: true)
        //self.hotKeyManager = HotKeyManager(companion: window as! CompanionWindow)
    }
    var observer: AXObserver?

    @objc func addAccesbilityObserver() {
        let first = WindowServer.shared.topmostWindow(for: NSWorkspace.shared.frontmostApplication!)!
        print(first.bundleId)
        let axErr = AXObserverCreate(first.app.processIdentifier, { (observer: AXObserver, element: AXUIElement, notificationName: CFString, refcon: UnsafeMutableRawPointer?) -> Void in
                print("yoyoyo")
                print(notificationName)
            WindowManager.shared.requestWindowUpdate()
            
        }, &observer)
        
        //kAXWindowMovedNotification
        AXObserverAddNotification(observer!, first.accesibilityElement!, kAXWindowMovedNotification as CFString, nil)
        AXObserverAddNotification(observer!, first.accesibilityElement!, kAXWindowResizedNotification as CFString, nil)
//        _ element: AXUIElement,
//        _ notification: CFString,
//        _ refcon: UnsafeMutableRawPointer?)
//        AXObserverAddNotification(observer,
        
        //[[NSRunLoop currentRunLoop] getCFRunLoop]
        print(axErr)
        print(observer)
        CFRunLoopAddSource(CFRunLoopGetCurrent(), AXObserverGetRunLoopSource(observer!), CFRunLoopMode.defaultMode);

//        CFRunLoopAddSource( RunLoop.current.getCFRunLoop()), AXObserverGetRunLoopSource(observer), kCFRunLoopDefaultMode );
    }
    
    @objc func hide() {
//         if let companion = self.window as? CompanionWindow,
//            let vc = companion.contentViewController as? WebViewController,
//            let webView = vc.webView {
//            companion.positioning = .icon
//            webView.loadRemoteApp(at: Remote.baseURL.appendingPathComponent("hide"))
//
//        }
        
        Defaults.showSidebar = !Defaults.showSidebar
        WindowManager.shared.requestWindowUpdate()
        
    }
    
    @objc func terminalWindowToFront() {
        WindowManager.shared.bringTerminalWindowToFront()
    }
    

    @objc func checkForUpdates() {
        print("Checking")
        self.updater?.checkForUpdates(self)
    }
    @objc func toggleVisibility() {
        if let window = self.window {
           let companion = window as! CompanionWindow
           let position = companion.positioning
           
           if(NSWorkspace.shared.frontmostApplication?.isFig ?? false) {
               ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
           }
           
            if position == CompanionWindow.defaultPassivePosition {
               companion.positioning = CompanionWindow.defaultActivePosition
                NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
           } else {
               companion.positioning = CompanionWindow.defaultPassivePosition
//            ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
           }
       }
    }
    
    
    @objc func applicationIsInStartUpItems() -> Bool {
      return itemReferencesInLoginItems().existingReference != nil
    }

    func toggleLaunchAtStartup(shouldBeOff: Bool = false) {
      let itemReferences = itemReferencesInLoginItems()
      let shouldBeToggled = (itemReferences.existingReference == nil)
      let loginItemsRef = LSSharedFileListCreate(
        nil,
        kLSSharedFileListSessionLoginItems.takeRetainedValue(),
        nil
      ).takeRetainedValue() as LSSharedFileList?
      
      if loginItemsRef != nil {
        if shouldBeToggled {
            let appUrl = NSURL.fileURL(withPath: Bundle.main.bundlePath) as CFURL
            LSSharedFileListInsertItemURL(loginItemsRef, itemReferences.lastReference, nil, nil, appUrl, nil, nil)
            print("Application was added to login items")
        }
        else if (shouldBeOff) {
          if let itemRef = itemReferences.existingReference {
            LSSharedFileListItemRemove(loginItemsRef,itemRef);
            print("Application was removed from login items")
          }
        }
      }
    }

    func itemReferencesInLoginItems() -> (existingReference: LSSharedFileListItem?, lastReference: LSSharedFileListItem?) {
        
        var itemUrl = UnsafeMutablePointer<Unmanaged<CFURL>?>.allocate(capacity: 1)

        let appUrl = NSURL(fileURLWithPath: Bundle.main.bundlePath)
        let loginItemsRef = LSSharedFileListCreate(
          nil,
          kLSSharedFileListSessionLoginItems.takeRetainedValue(),
          nil
        ).takeRetainedValue() as LSSharedFileList?
        
        if loginItemsRef != nil {
          let loginItems = LSSharedFileListCopySnapshot(loginItemsRef, nil).takeRetainedValue() as NSArray
          print("There are \(loginItems.count) login items")
          
          if(loginItems.count > 0) {
            let lastItemRef = loginItems.lastObject as! LSSharedFileListItem
        
            for i in 0...loginItems.count-1 {
                let currentItemRef = loginItems.object(at: i) as! LSSharedFileListItem
              
              if LSSharedFileListItemResolve(currentItemRef, 0, itemUrl, nil) == noErr {
                if let urlRef: NSURL = itemUrl.pointee?.takeRetainedValue() {
                    print("URL Ref: \(urlRef.lastPathComponent ?? "")")
                  if urlRef.isEqual(appUrl) {
                    return (currentItemRef, lastItemRef)
                  }
                }
              }
              else {
                print("Unknown login application")
              }
            }
            // The application was not found in the startup list
            return (nil, lastItemRef)
            
          } else  {
            let addatstart: LSSharedFileListItem = kLSSharedFileListItemBeforeFirst.takeRetainedValue()
            return(nil,addatstart)
          }
      }
      
      return (nil, nil)
    }
    
    @objc func quit() {
        ShellBridge.shared.stopWebSocketServer {
            DispatchQueue.main.async {
                NSApp.terminate(self)
            }
        }
    }
    
    @objc func promptForAccesibilityAccess() {
        ShellBridge.promptForAccesibilityAccess()
    }
    
    @objc func addCLI() {
        ShellBridge.symlinkCLI()
    }
    
    @objc func killSocketServer() {
        ShellBridge.shared.stopWebSocketServer()
    }
    
    @objc func startSocketServer() {
        ShellBridge.shared.startWebSocketServer()
    }

    @objc func spaceChanged() {
        print("spaceChanged!");
    }
    
    @objc func newActiveApp() {
        print("newActiveApp!");
    }
    func applicationWillTerminate(_ aNotification: Notification) {
        ShellBridge.shared.stopWebSocketServer()
    }
    
    @objc func runScriptCmd() {
        let path = "~/session.fig"//getDocumentsDirectory().appendingPathComponent("user.fig")
        print(path)
        injectStringIntoTerminal("script -q -t 0 \(path)")
    }
    
    @objc func runTailCmd() {
        let path = "~/session.fig"//getDocumentsDirectory().appendingPathComponent("user.fig")

        let output = "tail -F \(path)".runAsCommand()
        
        print(output)
    }
        
    @objc func runExitCmd() {
         injectStringIntoTerminal("exit")
     }
    

    // > fig search
    @objc func getTopTerminalWindow() {
        guard let app = NSWorkspace.shared.frontmostApplication else {
            return
        }
        
        if app.bundleIdentifier == "com.googlecode.iterm2" {
            let appRef = AXUIElementCreateApplication(app.processIdentifier)
            
            var window: AnyObject?
            let result = AXUIElementCopyAttributeValue(appRef, kAXFocusedWindowAttribute as CFString, &window)
            // add error handling
            
            if (result == .apiDisabled) {
                print("Accesibility needs to be enabled.")
                return
            }
            
            print(window ?? "<none>" )
            
            var position : AnyObject?
            var size : AnyObject?

            let result2 = AXUIElementCopyAttributeValue(window as! AXUIElement, kAXPositionAttribute as CFString, &position)
            AXUIElementCopyAttributeValue(window as! AXUIElement, kAXSizeAttribute as CFString, &size)

            switch(result2) {
            case .parameterizedAttributeUnsupported:
                    print("parameterizedAttributeUnsupported")
            case .success:
                print("success")

            case .failure:
                print("error")

            case .illegalArgument:
                print("error")

            case .invalidUIElement:
                print("error")

            case .invalidUIElementObserver:
                print("error")

            case .cannotComplete:
                print("error")

            case .attributeUnsupported:
                print("error")

            case .actionUnsupported:
                print("error")

            case .notificationUnsupported:
                print("error")
            case .notImplemented:
                 print("error")
                
            case .notificationAlreadyRegistered:
                print("error")

            case .notificationNotRegistered:
                print("error")

            case .apiDisabled:
                print("error")

            case .noValue:
                print("error")

            case .notEnoughPrecision:
                print("error")

            @unknown default:
                print("error")

            }
            
            if let position = position, let size = size {
                let point = AXValueGetters.asCGPoint(value: position as! AXValue)
                let bounds = AXValueGetters.asCGSize(value: size as! AXValue)
                print(point, bounds)
                
                
                let titleBarHeight:CGFloat = 23.0;
                
                let includeTitleBarHeight = false;
                
                let terminalWindowFrame = NSRect.init(x: point.x, y: (NSScreen.main?.visibleFrame.height)! - point.y + ((includeTitleBarHeight) ? titleBarHeight : 0), width: bounds.width, height: bounds.height - ((includeTitleBarHeight) ? 0 : titleBarHeight))
                    //CGRect.init(origin: point, size: bounds)
                print(terminalWindowFrame)
//                let terminalFrame = NSRectFromCGRect(terminalWindowFrame)
                self.window.windowController?.shouldCascadeWindows = false;
                
//                print("Before:", terminalWindowFrame)
//                let figWindow = overlayFrame(OverlayPositioning.init(rawValue: self.clicks % 7)!, terminalFrame: terminalWindowFrame, screenBounds: .zero)
//                print("After:", figWindow)

//                self.window.setFrame(figWindow, display: true)
//                self.window.setFrameTopLeftPoint(figWindow.origin)
                self.clicks += 1;
//                self.window.setFrameOrigin(NSPoint.init(x: point.x, y: (point.y < NSScreen.main!.frame.height/2) ? point.y + bounds.height : point.y - bounds.height) )
////                self.window.cascadeTopLeft(from: NSPointFromCGPoint(point))

                print(self.window.frame)
            }
            


            //
        }

//        let type = CGWindowListOption.optionOnScreenOnly
//        let windowList = CGWindowListCopyWindowInfo(type, kCGNullWindowID) as NSArray? as? [[String: AnyObject]]
//
//        for entry  in windowList!
//        {
//          let owner = entry[kCGWindowOwnerName as String] as! String
//          var bounds = entry[kCGWindowBounds as String] as? [String: Int]
//          let pid = entry[kCGWindowOwnerPID as String] as? Int32
//
//          if owner == "iTerm2"
//          {
//            let appRef = AXUIElementCreateApplication(pid!);  //TopLevel Accessability Object of PID
//
//            var value: AnyObject?
//            let result = AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &value)
//
//            if let windowList = value as? [AXUIElement]
//            { print ("windowList #\(windowList)")
//              if let window = windowList.first
//              {
//                print(window)
//                var position : CFTypeRef
//                var size : CFTypeRef
//                var  newPoint = CGPoint(x: 0, y: 0)
//                var newSize = CGSize(width: 800, height: 800)
//
//                position = AXValueCreate(AXValueType(rawValue: kAXValueCGPointType)!,&newPoint)!;
//                AXUIElementSetAttributeValue(windowList.first!, kAXPositionAttribute as CFString, position);
//
//               // AXUIElementCopyAttributeValue(windowList.first!, kAXPositionAttribute as CFString, )
//
//                size = AXValueCreate(AXValueType(rawValue: kAXValueCGSizeType)!,&newSize)!;
//                AXUIElementSetAttributeValue(windowList.first!, kAXSizeAttribute as CFString, size);
//
//                print(newSize)
//              }
//            }
//          }
//        }
    }
    
    @objc func allWindows() {
        guard let jsons = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[String: Any]] else {
            return
        }

        let infos = jsons.compactMap({ WindowInfo(json: $0) })
        print (infos)
        
//        print (infos.filter ({
//            return $0.name == "iTerm2"
//        }))
//        if let info = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[ String : Any]] {
//            for dict in info {
//                print(dict)
//            }
//        }
    }
    
    @objc func pasteStringToTerminal() {
        let terminals = NSRunningApplication.runningApplications(withBundleIdentifier: "com.googlecode.iterm2")
        if let activeTerminal = terminals.first {
            activeTerminal.activate(options: NSApplication.ActivationOptions.init())
            simulateKeyPress(pid: activeTerminal.processIdentifier)
        }
               
 
    }
    
    @objc func frontmostApplication() {
        print (NSWorkspace.shared.frontmostApplication?.localizedName ?? "")
    }
    
    @objc func copyToPasteboard() {
        let input = "echo \"hello world\""

        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(input, forType: .string)
        
    }
    
    func injectStringIntoTerminal(_ cmd: String, runImmediately: Bool = false) {
         if let currentApp = NSWorkspace.shared.frontmostApplication {
                
            if (currentApp.bundleIdentifier == "com.googlecode.iterm2") {
                // save current pasteboard
                let pasteboard = NSPasteboard.general
                let copiedString = pasteboard.string(forType: .string) ?? ""
                
                // add our script to pasteboard
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(cmd, forType: .string)
                print(pasteboard.string(forType: .string) ?? "")
                    self.simulate(keypress: .cmdV)
                    self.simulate(keypress: .rightArrow)
                    self.simulate(keypress: .enter)
 
                // need delay so that terminal responds
                Timer.delayWithSeconds(1) {
                    // restore pasteboard
                    NSPasteboard.general.clearContents()
                    pasteboard.setString(copiedString, forType: .string)
                }
            }
        }
    }
    
    @objc func sendStringIfTerminalActive() {
        
        let input = "echo \"hello world\""
        if let currentApp = NSWorkspace.shared.frontmostApplication {
        
            if (currentApp.bundleIdentifier == "com.googlecode.iterm2") {
                // save current pasteboard
                let pasteboard = NSPasteboard.general
                let copiedString = pasteboard.string(forType: .string) ?? ""
                
                // add our script to pasteboard
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(input, forType: .string)
                print(pasteboard.string(forType: .string) ?? "")
//                simulateRawKeyPress(flag: true)
                    self.simulate(keypress: .cmdV)
                    self.simulate(keypress: .rightArrow)
                    self.simulate(keypress: .enter)
 
                // need delay so that terminal responds
                Timer.delayWithSeconds(1) {
                    // restore pasteboard
                    NSPasteboard.general.clearContents()
                    pasteboard.setString(copiedString, forType: .string)
                }
            }
        }
    }
    
    @objc func checkWinows() {
        
        let windowNumbers = NSWindow.windowNumbers(options: [])
        windowNumbers?.forEach( { print($0.decimalValue) })
    }
    
    // /Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/Events.h
    //https://gist.github.com/eegrok/949034
    enum Keypress: UInt16 {
        case cmdV = 9
        case enter = 36
        case rightArrow = 124
    }
    
    func simulate(keypress: Keypress) {
        let keyCode = keypress.rawValue as CGKeyCode
//        print(keypress.rawValue, keyCode)
        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let keydown = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: true)
        let keyup = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: false)
        
        if (keypress == .cmdV){
            keydown?.flags = CGEventFlags.maskCommand;
        }
        
        let loc = CGEventTapLocation.cghidEventTap
        keydown?.post(tap: loc)
        keyup?.post(tap: loc)
    }
    
    func simulateRawKeyPress(flag: Bool = false) {
        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let v_down = CGEvent(keyboardEventSource: src, virtualKey: 9 as CGKeyCode, keyDown: true)
        let v_up = CGEvent(keyboardEventSource: src, virtualKey: 9 as CGKeyCode, keyDown: false)
        
        if (flag){
            v_down?.flags = CGEventFlags.maskCommand;
        }
        
        let loc = CGEventTapLocation.cghidEventTap
        v_down?.post(tap: loc)
        v_up?.post(tap: loc)
    }

    func simulateKeyPress(pid: pid_t, flag: Bool = false) {
        print("Simulate keypress for process: \(pid)")

        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let v_down = CGEvent(keyboardEventSource: src, virtualKey: 9 as CGKeyCode, keyDown: true)
        let v_up = CGEvent(keyboardEventSource: src, virtualKey: 9 as CGKeyCode, keyDown: false)
//        let spcd = CGEvent(keyboardEventSource: src, virtualKey: 0x31, keyDown: true)
//        let spcu = CGEvent(keyboardEventSource: src, virtualKey: 0x31, keyDown: false)

        if (flag){
            v_down?.flags = CGEventFlags.maskCommand;
        }
//        v_up?.flags = CGEventFlags.maskCommand;

//        let loc = CGEventTapLocation.cghidEventTap
        

        v_down?.postToPid(pid)
        v_up?.postToPid(pid)

//        v_down?.post(tap: loc)
//        v_up?.post(tap: loc)

//        spcd?.post(tap: loc)
//        spcu?.post(tap: loc)
//        cmdu?.post(tap: loc)
    }
    
    func windowDidMove(_ notification: Notification) {
        print(notification.object ?? "<none>")
        
        print("WINDOW MOVED", window.frame)
//        print("SCREEN", NSScreen.main?.frame ?? "<none>")
    }


}

fileprivate func delayWithSeconds(_ seconds: Double, completion: @escaping () -> ()) {
    DispatchQueue.main.asyncAfter(deadline: .now() + seconds) {
        completion()
    }
}

func getDocumentsDirectory() -> URL {
    return URL(fileURLWithPath: NSSearchPathForDirectoriesInDomains(.documentDirectory, .userDomainMask, true)[0])
}

struct WindowInfo {
    let frame: CGRect
    let name: String
    let pid: Int
    let number: Int
    let visible: Bool

    init?(json: [String: Any]) {
        guard let pid = json["kCGWindowOwnerPID"] as? Int else {
            return nil
        }

        guard let name = json["kCGWindowOwnerName"] as? String else {
            return nil
        }
        
        guard let onScreen = json["kCGWindowIsOnscreen"] as? Bool else {
            return nil
        }
        
        guard let rect = json["kCGWindowBounds"] as? [String: Any] else {
                  return nil
              }

        guard let x = rect["X"] as? CGFloat else {
            return nil
        }

        guard let y = rect["Y"] as? CGFloat else {
            return nil
        }

        guard let height = rect["Height"] as? CGFloat else {
            return nil
        }

        guard let width = rect["Width"] as? CGFloat else {
            return nil
        }

        guard let number = json["kCGWindowNumber"] as? Int else {
            return nil
        }

        self.pid = pid
        self.name = name
        self.number = number
        self.frame = CGRect(x: x, y: y, width: width, height: height)
        self.visible = onScreen
    }
}

class AXValueGetters {

    class func asCGRect(value: AXValue) -> CGRect {
        var val = CGRect.zero
        AXValueGetValue(value, AXValueType.cgRect, &val)
        return val
    }

    class func asCGPoint(value: AXValue) -> CGPoint {
        var val = CGPoint.zero
        AXValueGetValue(value, AXValueType.cgPoint, &val)
        return val
    }

    class func asCFRange(value: AXValue) -> CFRange {
        var val = CFRange(location: 0, length: 0)
        AXValueGetValue(value, AXValueType.cfRange, &val)
        return val
    }

    class func asCGSize(value: AXValue) -> CGSize {
        var val = CGSize.zero
        AXValueGetValue(value, AXValueType.cgSize, &val)
        return val
    }

}

extension AppDelegate : SUUpdaterDelegate {
    func updater(_ updater: SUUpdater, didAbortWithError error: Error) {
        
    }
    
    func updaterDidNotFindUpdate(_ updater: SUUpdater) {
        
    }
    
    func updater(_ updater: SUUpdater, didFindValidUpdate item: SUAppcastItem) {
        print("Found valid update")
    }
    
    func updater(_ updater: SUUpdater, didFinishLoading appcast: SUAppcast) {
//        let item = (appcast.items?.first! as! SUAppcastItem)
//        item.
    }
}
