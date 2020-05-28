//
//  AppDelegate.swift
//  fig
//
//  Created by Matt Schrage on 4/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
import SwiftUI

@NSApplicationMain
class AppDelegate: NSObject, NSApplicationDelegate,NSWindowDelegate {

    var window: NSWindow!
    var statusBarItem: NSStatusItem!
    var clicks:Int = 6;

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        let _ = ShellBridge.shared
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
        
        statusBarMenu.addItem(
         withTitle: "Add CLI Tool",
         action: #selector(AppDelegate.addCLI),
         keyEquivalent: "")
        statusBarMenu.addItem(
         withTitle: "Prompt for Accesibility Access",
         action: #selector(AppDelegate.promptForAccesibilityAccess),
         keyEquivalent: "")
        statusBarMenu.addItem(
         withTitle: "Quit Fig",
         action: #selector(AppDelegate.quit),
         keyEquivalent: "")
        // Create the SwiftUI view that provides the window contents.
//        let contentView = ContentView()

        window = CompanionWindow(viewController: WebViewController())
        // Create the window and set the content view.
//        window = NSWindow(
//            contentRect: NSRect(x: 0, y: 0, width: 480, height: 300),
//            styleMask: [.fullSizeContentView ],
//            backing: .buffered, defer: false)
//        window.center()
//        window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .transient]
//        window.isMovableByWindowBackground = true
//        window.isMovable = true
//        window.isOpaque = false
//        window.backgroundColor = .clear//NSColor.init(white: 1, alpha: 0.75)
//        window.delegate = self
//        window.level = .floating
//        window.setFrameAutosaveName("Main Window")
////        window.contentView = NSHostingView(rootView: contentView)
//        window.contentViewController = WebViewController()
        window.makeKeyAndOrderFront(nil)
//
//        let timer = Timer.scheduledTimer(timeInterval: 0.1, target: self, selector: #selector(positionWindow), userInfo: nil, repeats: true)
//
//        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(positionWindow), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)
//        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(positionWindow), name: NSWorkspace.didActivateApplicationNotification, object: nil)
//
//        let terminals = NSRunningApplication.runningApplications(withBundleIdentifier: "com.googlecode.iterm2")
//
//        terminals.forEach({ print("\($0.localizedName ?? "Anonymous") (\($0.processIdentifier)) -- \($0.isActive)") })
        
//        if let activeTerminal = terminals.first {
            // save current pasteboard
//            let pasteboard = NSPasteboard.general
//            let copiedString = pasteboard.string(forType: .string) ?? ""
//            print(copiedString)
//            NSPasteboard.general.clearContents()
//
//            NSPasteboard.general.setString("Hello there", forType: .string)
//            let insertString = pasteboard.string(forType: .string) ?? "<none>"
//            print(insertString)
            
//
//            NSPasteboard.general.clearContents()
//            pasteboard.setString(copiedString, forType: .string)
//        }
//        let src = CGEventSource(stateID: .hidSystemState)
//        if let event = CGEvent(keyboardEventSource: src, virtualKey: 0x0f, keyDown: true)
//        {
//            event.flags = .maskCommand
//            event.postToPid()
//        }
    }
    
    @objc func quit() {
        NSApp.terminate(self)
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
    
    enum OverlayPositioning: Int {
        case coveringTitlebar = 0
        case insideRightFull = 1
        case insideRightPartial = 2
        case outsideRight = 3
        case atPrompt = 4
        case icon = 5
        case notification = 6
    
    }
    
    var priorTargetFrame: NSRect = .zero
    
    @objc func positionWindow() {
        repositionWindow(forceUpdate: true)
    }
    func repositionWindow( forceUpdate:Bool = false) {
        let whitelistedBundleIds = Integrations.whitelist
//                                    ["com.googlecode.iterm2",
//                                    "com.google.Chrome",
//                                    "com.sublimetext.3",
//                                    "com.apple.dt.Xcode",
//                                    "com.apple.Terminal"]
        guard let app = NSWorkspace.shared.frontmostApplication,
              let bundleId = app.bundleIdentifier else {
                   return
               }
        
        if (whitelistedBundleIds.contains(bundleId)) {
            let targetFrame = topmostWindowFrameFor(app)

            if (forceUpdate || !targetFrame.equalTo(priorTargetFrame)) {
                priorTargetFrame = targetFrame
                let frame = overlayFrame(OverlayPositioning.init(rawValue: self.clicks % 7)!,
                                         terminalFrame: targetFrame,
                                         screenBounds: .zero)
                setOverlayFrame(frame)
    
            }
            
        } else if (bundleId == "com.mschrage.fig") {
            print("fig window is active: previous: \(ShellBridge.shared.previousFrontmostApplication?.bundleIdentifier ?? "none" )");
        } else {
            window.orderOut(self)
        }
    }
    
    func setOverlayFrame(_ frame: NSRect) {
        // no update if frame hasn't changed
        self.window.windowController?.shouldCascadeWindows = false;
        self.window.setFrame(frame, display: true)
        self.window.setFrameTopLeftPoint(frame.origin)
        
        window.makeKeyAndOrderFront(self)
//        NSApp.activate(ignoringOtherApps: true)
    }
    
    func topmostWindowFrameFor(_ app: NSRunningApplication, includingTitleBar: Bool = false) -> NSRect {
        let appRef = AXUIElementCreateApplication(app.processIdentifier)
        
        var window: AnyObject?
        let result = AXUIElementCopyAttributeValue(appRef, kAXFocusedWindowAttribute as CFString, &window)
        
        if (result == .apiDisabled) {
            print("Accesibility needs to be enabled.")
            return .zero
        }
                
        var position : AnyObject?
        var size : AnyObject?
        
        guard (window != nil) else {
            print("Window does not exist.")
            return .zero
        }

        AXUIElementCopyAttributeValue(window as! AXUIElement, kAXPositionAttribute as CFString, &position)
        AXUIElementCopyAttributeValue(window as! AXUIElement, kAXSizeAttribute as CFString, &size)
        
        if let position = position, let size = size {
            let point = AXValueGetters.asCGPoint(value: position as! AXValue)
            let bounds = AXValueGetters.asCGSize(value: size as! AXValue)
            
            let titleBarHeight:CGFloat = 23.0;
                         
            return NSRect.init(x: point.x,
                               y: (NSScreen.main?.visibleFrame.height)! - point.y + ((includingTitleBar) ? titleBarHeight : 0),
                               width:  bounds.width,
                               height: bounds.height - ((includingTitleBar) ? 0 : titleBarHeight))
        }
        return .zero
    }
    
    func overlayFrame( _ positioning: OverlayPositioning, terminalFrame: NSRect, screenBounds: NSRect) -> NSRect {
        if terminalFrame.width < 100 || terminalFrame.height < 200 {
            return .zero
        }
        let t_size = terminalFrame.size
        switch positioning {
        case .coveringTitlebar:
            return NSRect(origin: terminalFrame.origin, size: CGSize.init(width: t_size.width, height: 100))
        case .insideRightFull:
            return terminalFrame.divided(atDistance: 300, from: .maxXEdge).slice
        case .insideRightPartial:
            return terminalFrame.divided(atDistance: 300, from: .maxXEdge).slice.divided(atDistance: t_size.height * ( 2 / 3 ), from: .maxYEdge).slice.offsetBy(dx: 0, dy: -t_size.height / 3)
        case .atPrompt:
        
            let inner = terminalFrame.insetBy(dx: 30, dy: 45)
            return NSRect(x: inner.origin.x, y: inner.origin.y - inner.height, width: inner.width, height: 100)
        case .outsideRight:
            return terminalFrame.insetBy(dx: -300, dy: 0).divided(atDistance: 300, from: .maxXEdge).slice
        case .icon:
            let i_size =  CGSize(width: 50, height: 50)
            let i_padding = CGPoint(x: 12, y: -36);
            return NSRect(origin: CGPoint.init(x:terminalFrame.maxX - i_size.width - i_padding.x,
                                               y: terminalFrame.minY - i_size.height - i_padding.y), size: i_size)
        case .notification:
            let i_size =  CGSize(width: 300, height: 120)
            let i_padding = CGPoint(x: 12, y: -120 + 12);
            return NSRect(origin: CGPoint.init(x:terminalFrame.maxX - i_size.width - i_padding.x,
                                               y: terminalFrame.minY - i_size.height - i_padding.y), size: i_size)
        }
  
    }
    
    @objc func updateOverlayStyle() {
        self.clicks += 1;
        self.repositionWindow(forceUpdate: true)
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
                
                print("Before:", terminalWindowFrame)
                let figWindow = overlayFrame(OverlayPositioning.init(rawValue: self.clicks % 7)!, terminalFrame: terminalWindowFrame, screenBounds: .zero)
                print("After:", figWindow)

                self.window.setFrame(figWindow, display: true)
                self.window.setFrameTopLeftPoint(figWindow.origin)
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
        
        print (infos.filter ({
            return $0.name == "iTerm2"
        }))
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

    init?(json: [String: Any]) {
        guard let pid = json["kCGWindowOwnerPID"] as? Int else {
            return nil
        }

        guard let name = json["kCGWindowOwnerName"] as? String else {
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
