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
    var hotkey = HotKey(key: .grave, modifiers: [.command])
    let companionWindow: CompanionWindow
    let webview: WebView
    init (companion: CompanionWindow){
        companionWindow = companion
        webview = (companion.contentViewController as! WebViewController).webView!
        self.hotkey.keyDownHandler = {
            switch self.companionWindow.positioning {
            case CompanionWindow.defaultPassivePosition:
                self.shouldTab = true
                NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
                WebBridge.tabInSidebar(webView: self.webview)
            case CompanionWindow.defaultActivePosition:
                (NSApplication.shared.delegate as! AppDelegate).toggleVisibility()
            default:
                (NSApplication.shared.delegate as! AppDelegate).toggleVisibility()
            }
        }
        
        NSEvent.addLocalMonitorForEvents(matching: .flagsChanged) { (event) -> NSEvent? in
            self.flagsChanged(event: event)
            return event
        }
        NSEvent.addGlobalMonitorForEvents(matching: .flagsChanged, handler: flagsChanged)
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
