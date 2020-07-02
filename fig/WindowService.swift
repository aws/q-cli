//
//  WindowService.swift
//  fig
//
//  Created by Matt Schrage on 6/28/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa

protocol WindowService {

    func topmostWhitelistedWindow() -> ExternalWindow?
    func topmostWindow(for app: NSRunningApplication) -> ExternalWindow?
    func previousFrontmostApplication() -> NSRunningApplication?
    func currentApplicationIsWhitelisted() -> Bool
    func allWindows() -> [ExternalWindow]
    func allWhitelistedWindows() -> [ExternalWindow]
    func previousWhitelistedWindow() -> ExternalWindow?
    func bringToFront(window: ExternalWindow)
}


class WindowServer : WindowService {
    func bringToFront(window: ExternalWindow) {
        
        let appRef = AXUIElementCreateApplication(window.app.processIdentifier)
        var appWindows: CFArray?
        let error = AXUIElementCopyAttributeValues(appRef, kAXWindowsAttribute as CFString, 0, 99999, &appWindows)
        
        if error == .noValue || error == .attributeUnsupported {
            return
        }

        guard error == .success, let windows = appWindows as? [AXUIElement] else {
            return
        }
        
        let potentialTarget = windows.filter { PrivateWindow.getCGWindowID(fromRef: $0) == window.windowId}
        
        guard let target = potentialTarget.first else {
            return
        }

        AXUIElementPerformAction(target, kAXRaiseAction as CFString);

    }
    
    static let whitelistedWindowDidChangeNotification: NSNotification.Name = Notification.Name("whitelistedWindowDidChangeNotification")

    func previousWhitelistedWindow() -> ExternalWindow? {
        return self.previousWindow
    }
    
    func topmostWhitelistedWindow() -> ExternalWindow? {
        guard self.currentApplicationIsWhitelisted() else { return nil }
        return topmostWindow(for: NSWorkspace.shared.frontmostApplication!)
    }
    
    func currentApplicationIsWhitelisted() -> Bool{
        let whitelistedBundleIds = Integrations.whitelist
        if let app = NSWorkspace.shared.frontmostApplication,
            let bundleId = app.bundleIdentifier {
            return whitelistedBundleIds.contains(bundleId)
        }
        
        return false
    }
    
    func allWindows() -> [ExternalWindow] {
        guard let rawWindows = CGWindowListCopyWindowInfo(.optionAll, kCGNullWindowID) as? [[String: Any]] else {
            return []
        }
        
        var allWindows: [ExternalWindow] = []
        for rawWindow in rawWindows {
            if let window = ExternalWindow(raw: rawWindow) {
                allWindows.append(window)
            }
        }
        allWindows.forEach{ print($0.bundleId ?? "", $0.windowId)}
        return allWindows
    }
    
    func allWhitelistedWindows() -> [ExternalWindow] {
        return self.allWindows().filter { Integrations.whitelist.contains($0.bundleId ?? "") }
    }
    
    static let shared = WindowServer()

    func previousFrontmostApplication() -> NSRunningApplication? {
        return self.previousApplication
    }
    
    var previousApplication: NSRunningApplication?
    var previousWindow: ExternalWindow? {
        willSet(value) {
            if (self.previousWindow != value) {
                print("Old window \(self.previousWindow?.windowId ?? 0)")
                print("New window \(value?.windowId ?? 0)")
                NotificationCenter.default.post(name: WindowServer.whitelistedWindowDidChangeNotification, object: value)
            }
        }
    }
    
    init() {
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(setPreviousApplication(notification:)), name: NSWorkspace.didDeactivateApplicationNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(spaceChanged), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)
        _ = Timer.scheduledTimer(timeInterval: 0.15, target: self, selector: #selector(setPreviousWindow), userInfo: nil, repeats: true)

    }
    //https://stackoverflow.com/questions/853833/how-can-my-app-detect-a-change-to-another-apps-window
    @objc func setPreviousWindow() {
        self.previousWindow = self.topmostWhitelistedWindow()
    }
    
    @objc func spaceChanged() {
        // this is used to reset previous application when space is changed. Maybe should be nil.
        self.previousApplication = NSWorkspace.shared.frontmostApplication
    }

    @objc func setPreviousApplication(notification: NSNotification!) {
        self.previousApplication = notification!.userInfo![NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
    }

    
    func topmostWindow(for app: NSRunningApplication) -> ExternalWindow? {
        let appRef = AXUIElementCreateApplication(app.processIdentifier)
        var window: AnyObject?
        let result = AXUIElementCopyAttributeValue(appRef, kAXFocusedWindowAttribute as CFString, &window)
     
        if (result == .apiDisabled) {
            print("Accesibility needs to be enabled.")
            return nil
        }
             
        var position : AnyObject?
        var size : AnyObject?
     
        guard (window != nil) else {
            print("Window does not exist.")
            return nil
        }

        let windowId = PrivateWindow.getCGWindowID(fromRef: window as! AXUIElement)

        AXUIElementCopyAttributeValue(window as! AXUIElement, kAXPositionAttribute as CFString, &position)
        AXUIElementCopyAttributeValue(window as! AXUIElement, kAXSizeAttribute as CFString, &size)
     
        if let position = position, let size = size {
            let point = AXValueGetters.asCGPoint(value: position as! AXValue)
            let bounds = AXValueGetters.asCGSize(value: size as! AXValue)

            //https://stackoverflow.com/a/19887161/926887
            let windowFrame = NSRect.init(x: point.x,
                            y: NSMaxY(NSScreen.screens[0].frame) - point.y,
                            width:  bounds.width,
                            height: bounds.height)
            
            return ExternalWindow(windowFrame, windowId, app)
        }
        return nil

    }
}

class ExternalWindow {
    let frame: NSRect
    let windowId: CGWindowID
    let app: NSRunningApplication
    
    init?(raw: [String: Any]) {
        guard let pid = raw["kCGWindowOwnerPID"] as? pid_t else {
          return nil
        }
        
        guard let rect = raw["kCGWindowBounds"] as? [String: Any] else {
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

        guard let id = raw["kCGWindowNumber"] as? CGWindowID else {
          return nil
        }
        
        guard let app = NSRunningApplication(processIdentifier: pid) else {
            return nil
        }
        
        self.app = app
        self.windowId = id
        self.frame = CGRect(x: x, y: y, width: width, height: height)
    }
    
    init(_ frame: NSRect, _ windowId: CGWindowID, _ app: NSRunningApplication) {
        self.frame = frame
        self.windowId = windowId
        self.app = app
    }

    var frameWithoutTitleBar: NSRect {
        get {
            let titleBarHeight:CGFloat = 23.0;

            return NSRect.init(x: frame.origin.x,
                               y: frame.origin.y - titleBarHeight,
                               width:  frame.width,
                               height: frame.height - titleBarHeight)
        }
    }
    
    var title: String? {
        get {
            return self.app.localizedName
        }
    }
    
    var bundleId: String? {
        get {
           return self.app.bundleIdentifier
        }
    }
    
}

extension ExternalWindow: Hashable {
    func hash(into hasher: inout Hasher) {
         hasher.combine(self.windowId)
       }
       
       static func ==(lhs: ExternalWindow, rhs: ExternalWindow) -> Bool {
         return lhs.windowId == rhs.windowId
       }
}

