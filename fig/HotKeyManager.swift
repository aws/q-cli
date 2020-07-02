//
//  HotKeyManager.swift
//  fig
//
//  Created by Matt Schrage on 6/9/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa
import HotKey

class HotKeyManager {
//    var hotkey = HotKey(key: .grave, modifiers: [.command])
    var hotkey = HotKey(key: .i, modifiers: [.command])
    var focusKey = HotKey(key: .i, modifiers: [.command, .shift])

    var companionWindow: CompanionWindow
    var webview: WebView {
        get {
            return (self.companionWindow.contentViewController as! WebViewController).webView!
        }
    }
    init (companion: CompanionWindow) {
        companionWindow = companion
//        webview = (companion.contentViewController as! WebViewController).webView!
        self.hotkey.keyDownHandler = {
            switch self.companionWindow.positioning {
            case CompanionWindow.defaultPassivePosition:
                self.shouldTab = true
                NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
                WebBridge.tabInSidebar(webView: self.webview)
            case CompanionWindow.defaultActivePosition:
                self.companionWindow.toSidebar()
                //(NSApplication.shared.delegate as! AppDelegate).toggleVisibility()
            default:
                self.companionWindow.toSidebar()
                //(NSApplication.shared.delegate as! AppDelegate).toggleVisibility()
            }
        }
        
        self.focusKey.keyDownHandler = {
            switch self.companionWindow.positioning {
            case CompanionWindow.defaultPassivePosition:
                self.shouldTab = true
                NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
                WebBridge.tabInSidebar(webView: self.webview, shift: true)
            default:
                if let app = NSWorkspace.shared.frontmostApplication, app.isFig {
                    ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
                } else {
                    NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
                }
            }
        }
        
        NSEvent.addLocalMonitorForEvents(matching: .keyDown) { (event) -> NSEvent? in
            // ESC
            if (event.keyCode == 53) {
                self.shouldTab = false
                self.webview.evaluateJavaScript("document.activeElement.blur();", completionHandler: nil)
                if let app = NSWorkspace.shared.frontmostApplication, app.isFig {
                   ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
                }
                return nil
            }
            
            // control-d
            if (event.keyCode == 2 && event.modifierFlags.contains(.control)) {
                if (self.companionWindow.positioning != CompanionWindow.defaultPassivePosition) {
                    self.companionWindow.toSidebar()
                }
                return nil
            }
            
            // control-c
            if (event.keyCode == 8 && event.modifierFlags.contains(.control)) {
                if (self.companionWindow.positioning != CompanionWindow.defaultPassivePosition) {
                    self.companionWindow.toSidebar()
                }
                return nil
            }
            
            // control-z
            if (event.keyCode == 6 && event.modifierFlags.contains(.control)) {
                if (self.companionWindow.positioning != CompanionWindow.defaultPassivePosition) {
                    self.companionWindow.toSidebar()
                }
                return nil
            }

            
            if (event.keyCode == 3 && event.modifierFlags.contains(.command)) {
                switch self.companionWindow.positioning {
                case CompanionWindow.defaultPassivePosition:
                    return nil
                case .fullscreenInset:
                    self.companionWindow.positioning = CompanionWindow.defaultActivePosition
                    self.companionWindow.repositionWindow(forceUpdate: true, explicit: true)

                    return nil
                default:
                    self.companionWindow.positioning = .fullscreenInset
                    return nil
                }

            }
            
            if (event.keyCode == 2 && event.modifierFlags.contains(.command)) {
                 switch self.companionWindow.positioning {
                 case CompanionWindow.defaultPassivePosition:
                     return nil
                 case .untethered:
                     self.companionWindow.positioning = CompanionWindow.defaultActivePosition
                     self.companionWindow.repositionWindow(forceUpdate: true, explicit: true)

                     return nil
                 default:
                     self.companionWindow.positioning = .untethered
                     return nil
                 }

             }
            
            return event;
        }
        
        NSEvent.addLocalMonitorForEvents(matching: .flagsChanged) { (event) -> NSEvent? in
            self.flagsChanged(event: event)
            return event
        }
        NSEvent.addGlobalMonitorForEvents(matching: .flagsChanged, handler: flagsChanged)
        
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(toggleHotkey), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(toggleHotkey), name: NSWorkspace.didActivateApplicationNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(toggleHotkey), name: NSWorkspace.didDeactivateApplicationNotification, object: nil)
    }
    
    @objc func toggleHotkey() {
        if let app = NSWorkspace.shared.frontmostApplication, let bundleId = app.bundleIdentifier {
            self.hotkey.isPaused = !(Integrations.whitelist.contains(bundleId) || app.isFig)
            self.focusKey.isPaused = !(Integrations.whitelist.contains(bundleId) || app.isFig)

        } else {
            self.hotkey.isPaused = true;
            self.focusKey.isPaused = true;

        }
        
    }

    var shouldTab = false;
    var oldModifiers: NSEvent.ModifierFlags = .deviceIndependentFlagsMask
    func flagsChanged(event : NSEvent) {
       switch event.modifierFlags.intersection(.deviceIndependentFlagsMask) {
       case .command:
           print("Command pressed")
       default:
           break
       }
       
       switch oldModifiers.subtracting(event.modifierFlags.intersection(.deviceIndependentFlagsMask)) {
           case .command:
               print("command released")
               if !(NSWorkspace.shared.frontmostApplication?.isFig ?? false) || !self.shouldTab { return }
               
               WebBridge.activateSelectedAppFromSidebar(webView: self.webview)
          
               self.shouldTab = false

           default:
               break
           
       }
       self.oldModifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)

   }
}
