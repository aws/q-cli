//
//  CompanionWindow.swift
//  fig
//
//  Created by Matt Schrage on 4/18/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa

class CompanionWindow : NSWindow {
    var priorTargetFrame: NSRect = .zero
    var positioning: OverlayPositioning = .insideRightPartial
    override public var canBecomeKey: Bool {
        get { return true }
    }
    
    init(viewController: NSViewController) {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: 480, height: 300),
            styleMask: [.fullSizeContentView, .resizable],
            backing: .buffered, defer: false)
        self.center()
        self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        self.isMovableByWindowBackground = true
        self.isOpaque = false
        self.backgroundColor = .clear//NSColor.init(white: 1, alpha: 0.75)
        self.delegate = self
        self.level = .floating
        self.setFrameAutosaveName("Main Window")
//        window.contentView = NSHostingView(rootView: contentView)
        self.contentViewController = viewController //WebViewController()
        self.makeKeyAndOrderFront(nil)
        
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(positionWindow), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)
        
        let timer = Timer.scheduledTimer(timeInterval: 0.1, target: self, selector: #selector(positionWindow), userInfo: nil, repeats: true)
        
//        let trackingArea = NSTrackingArea(rect: self.contentViewController!.view.frame,
//                                                options: [NSTrackingArea.Options.activeAlways ,NSTrackingArea.Options.mouseEnteredAndExited],
//                      owner: self, userInfo: nil)
//        self.contentViewController!.view.addTrackingArea(trackingArea)

    }
        
//    override func mouseEntered(with event: NSEvent) {
//        print("mouse entered")
//    }
//    
//    override func mouseExited(with event: NSEvent) {
//        print("mouse exited")
//    }
    
    @objc func positionWindow() {
      repositionWindow(forceUpdate: true)
    }
    
    
    enum OverlayPositioning: Int {
        case coveringTitlebar = 0
        case insideRightFull = 1
        case insideRightPartial = 2
        case outsideRight = 3
        case atPrompt = 4
        case icon = 5
        case notification = 6

        func frame(targetWindowFrame:NSRect) -> NSRect {
            if targetWindowFrame.width < 100 || targetWindowFrame.height < 200 {
                 return .zero
             }
            
             let t_size = targetWindowFrame.size
             switch self {
             case .coveringTitlebar:
                 return NSRect(origin: targetWindowFrame.origin, size: CGSize.init(width: t_size.width, height: 100))
             case .insideRightFull:
                 return targetWindowFrame.divided(atDistance: 300, from: .maxXEdge).slice
             case .insideRightPartial:
                 return targetWindowFrame.divided(atDistance: 300, from: .maxXEdge).slice.divided(atDistance: t_size.height * ( 2 / 3 ), from: .maxYEdge).slice.offsetBy(dx: 0, dy: -t_size.height / 3)
             case .atPrompt:
             
                 let inner = targetWindowFrame.insetBy(dx: 30, dy: 45)
                 return NSRect(x: inner.origin.x, y: inner.origin.y - inner.height, width: inner.width, height: 100)
             case .outsideRight:
                 return targetWindowFrame.insetBy(dx: -300, dy: 0).divided(atDistance: 300, from: .maxXEdge).slice
             case .icon:
                 let i_size =  CGSize(width: 50, height: 50)
                 let i_padding = CGPoint(x: 12, y: -36);
                 return NSRect(origin: CGPoint.init(x:targetWindowFrame.maxX - i_size.width - i_padding.x,
                                                    y: targetWindowFrame.minY - i_size.height - i_padding.y), size: i_size)
             case .notification:
                 let i_size =  CGSize(width: 275, height: 140)
                 let i_padding = CGPoint(x: 12, y: -36);
                 return NSRect(origin: CGPoint.init(x:targetWindowFrame.maxX - i_size.width - i_padding.x,
                                                    y: targetWindowFrame.minY - i_size.height - i_padding.y), size: i_size)
             }
            
    
        }
        
    }
    
    
    func repositionWindow( forceUpdate:Bool = false) {
        let whitelistedBundleIds = Integrations.whitelist
        guard let app = NSWorkspace.shared.frontmostApplication,
              let bundleId = app.bundleIdentifier else {
                   return
               }
        
        if (whitelistedBundleIds.contains(bundleId)) {
            let targetFrame = topmostWindowFrameFor(app)
            if (forceUpdate || !targetFrame.equalTo(priorTargetFrame)) {
                priorTargetFrame = targetFrame
                let frame = overlayFrame(self.positioning,
                                         terminalFrame: targetFrame,
                                         screenBounds: .zero)
                setOverlayFrame(frame)
    
            }
            
        } else if (bundleId == "com.mschrage.fig") {
            print("fig window is active: previous: \(ShellBridge.shared.previousFrontmostApplication?.bundleIdentifier ?? "none" )");
        } else {
            self.orderOut(self)
        }
    }
    
        func setOverlayFrame(_ frame: NSRect) {
            // no update if frame hasn't changed
            self.windowController?.shouldCascadeWindows = false;
            self.setFrame(frame, display: true)
            self.setFrameTopLeftPoint(frame.origin)
            
            self.makeKeyAndOrderFront(nil)
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
              let i_size =  CGSize(width: 500, height: 50)
              let i_padding = CGPoint(x: 12, y: -36);
              return NSRect(origin: CGPoint.init(x:terminalFrame.maxX - i_size.width - i_padding.x,
                                                 y: terminalFrame.minY - i_size.height - i_padding.y), size: i_size)
          case .notification:
            let i_size =  CGSize(width: 275, height: 140)
            let i_padding = CGPoint(x: 12, y: -36);
            return NSRect(origin: CGPoint.init(x: terminalFrame.maxX - i_size.width - i_padding.x,
                                               y: terminalFrame.minY - i_size.height - i_padding.y), size: i_size)
        }
    
      }
}

extension CompanionWindow : NSWindowDelegate {
    
}
