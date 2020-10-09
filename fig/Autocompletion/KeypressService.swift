//
//  KeypressService.swift
//  fig
//
//  Created by Matt Schrage on 9/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa
import Carbon

protocol KeypressService {
    func keyBuffer(for window: ExternalWindow) -> KeystrokeBuffer
//    func redirects(for window: ExternalWindow) -> Set<UInt16>

    func getTextRect(extendRange: Bool) -> CGRect?
    func clean()
    func addRedirect(for keycode: UInt16, in window: ExternalWindow)
    func removeRedirect(for keycode: UInt16, in window: ExternalWindow)

}

class KeypressProvider : KeypressService {
    static let shared = KeypressProvider(windowServiceProvider: WindowServer.shared)
    let windowServiceProvider: WindowService
    static let whitelist = ["com.googlecode.iterm2", "com.apple.Terminal"]
    var buffers: [ExternalWindowHash: KeystrokeBuffer] = [:]
    
    var handler: Any? = nil
    var tap: CFMachPort? = nil

    
//    fileprivate var redirects: Set<UInt16> = []
    var redirects: [ExternalWindowHash:  Set<UInt16>] = [:]

    func addRedirect(for keycode: UInt16, in window: ExternalWindow) {
        var set = redirects[window.hash] ?? []
        set.insert(keycode)
        redirects[window.hash] = set
    }
    
    func removeRedirect(for keycode: UInt16, in window: ExternalWindow) {
        if var set = redirects[window.hash] {
            set.remove(keycode)
            redirects[window.hash] = set
        }
    }
    
    init(windowServiceProvider: WindowService) {
        self.windowServiceProvider = windowServiceProvider
        registerKeystrokeHandler()
    }
    
    func registerKeystrokeHandler() {
        if let handler = self.handler {
            NSEvent.removeMonitor(handler)
        }

        self.handler = NSEvent.addGlobalMonitorForEvents(matching: .leftMouseUp) { (event) in
           // only handle keypresses if they are in iTerm
            if let window = self.windowServiceProvider.topmostWhitelistedWindow(), KeypressProvider.whitelist.contains(window.bundleId ?? "") {
               
                if (event.modifierFlags.contains(.option)) {
                    let keyBuffer = self.keyBuffer(for: window)
                    keyBuffer.buffer = nil
                }
           }
       }
        
        self.clean()
        
        if let tap = self.tap {
            CFMachPortInvalidate(tap)
            self.tap = nil
        }
        
        if let tap = registerKeyInterceptor() {
            self.tap = tap
        }

    }
    
    func registerKeyInterceptor() -> CFMachPort? {
        let eventMask = (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.keyUp.rawValue) | (1 << CGEventType.tapDisabledByTimeout.rawValue)
        
        guard let eventTap: CFMachPort = CGEvent.tapCreate(tap: CGEventTapLocation.cghidEventTap,
                                                     place: CGEventTapPlacement.tailAppendEventTap,
                                                     options: CGEventTapOptions.defaultTap,
                                                     eventsOfInterest: CGEventMask(eventMask),
                                                     callback: { (proxy, type, event, refcon) -> Unmanaged<CGEvent>? in
//                                                        return Unmanaged.passRetained(event)
                                                        print("Keystroke event!")
                                                        guard event.type != .tapDisabledByTimeout else {
                                                            if let tap = KeypressProvider.shared.tap {
                                                                CGEvent.tapEnable(tap: tap, enable: true)

                                                            }
                                                            return Unmanaged.passRetained(event)
                                                        }
                                                        // fixes slowdown when typing into Fig
                                                        guard !(NSWorkspace.shared.frontmostApplication?.isFig ?? false) else {
                                                            return Unmanaged.passRetained(event)
                                                        }
                                                        guard Defaults.useAutocomplete, let window = AXWindowServer.shared.whitelistedWindow, KeypressProvider.whitelist.contains(window.bundleId ?? "") else {
                                                            return Unmanaged.passRetained(event)
                                                        }
                                                        
                                                        
                                                        if [.keyDown , .keyUp].contains(type) {
                                                            let keyCode = event.getIntegerValueField(.keyboardEventKeycode)
                                                            print("eventTap", keyCode, event.getIntegerValueField(.eventTargetUnixProcessID))
                                                            print("eventTap", "\(window.hash)")
                                                            if (type == .keyDown && KeypressProvider.shared.redirects[window.hash]?.contains(UInt16(keyCode)) ?? false &&
                                                                !event.flags.contains(.maskCommand)
) {
                                                            
                                                                
                                                                print("eventTap", "Should redirect!")
                                                                WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.keypress(\"\(keyCode)\", \"\(window.hash)\") } catch(e) {}", completionHandler: nil)
                                                                return nil
                                                            } else {
                                                                
                                                                KeypressProvider.shared.handleKeystroke(event: NSEvent.init(cgEvent: event), in: window)
                                                               

                                                            }
                                                            //event.setIntegerValueField(.keyboardEventKeycode, value: keyCode)
                                                        }
                                                        return Unmanaged.passRetained(event) },
                                                     userInfo: nil) else {
                                                        print("Could not create tap")
                                                        return nil
        }


        let runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, eventTap, 0)
        CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, .commonModes)
        CGEvent.tapEnable(tap: eventTap, enable: true)
        //CFRunLoopRun()
        return eventTap
        
    }
    
    func clean() {
        buffers = [:]
    }
    
    func keyBuffer(for window: ExternalWindow) -> KeystrokeBuffer {
        if let buffer = self.buffers[window.hash] {
            return buffer
        } else {
            let buffer = KeystrokeBuffer()
            self.buffers[window.hash] = buffer
            return buffer
        }
    }
    
    func handleKeystroke(event: NSEvent?, in window: ExternalWindow) {
        if let rect = getTextRect() {
            let keyBuffer = self.keyBuffer(for: window)
            var active = false;
            if let event = event, event.type == NSEvent.EventType.keyDown {
                if let tty = ShellHookManager.shared.tty[window.hash] {
                    guard tty.running?.isShell ?? true else {
                        keyBuffer.buffer = nil
                        return
                    }
                }
                if let (buffer, index) = keyBuffer.handleKeystroke(event: event),
                    let b64 = buffer.data(using: .utf8)?.base64EncodedString() {
                    
                    
                    WindowManager.shared.autocomplete?.tetheredWindow = window
                    WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.autocomplete(b64DecodeUnicode(`\(b64)`), \(index), '\(window.hash)') } catch(e){} ", completionHandler: nil)
                    active = true
                } else {
                    WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.nocontext('\(window.hash)') } catch(e){} ", completionHandler: nil)
                }
            }
            
//            if (event!.isARepeat || event?.type == NSEvent.EventType.keyUp) {
                
                WindowManager.shared.positionAutocompletePopover(active: active)

//            }
            
        } else {
            KeypressProvider.shared.removeRedirect(for: Keycode.upArrow, in: window)
            KeypressProvider.shared.removeRedirect(for: Keycode.downArrow, in: window)
            KeypressProvider.shared.removeRedirect(for: Keycode.returnKey, in: window)
            KeypressProvider.shared.removeRedirect(for: Keycode.tab, in: window)
            KeypressProvider.shared.removeRedirect(for: Keycode.escape, in: window)
        }
    }
    
    func getTextRect(extendRange: Bool = true) -> CGRect? {
        let systemWideElement = AXUIElementCreateSystemWide()
        var focusedElement : AnyObject?
        
        let error = AXUIElementCopyAttributeValue(systemWideElement, kAXFocusedUIElementAttribute as CFString, &focusedElement)
        
        guard error == .success else {
            print("Couldn't get the focused element. Probably a webkit application")
            return nil
        }
        
        var selectedRangeValue : AnyObject?
        let selectedRangeError = AXUIElementCopyAttributeValue(focusedElement as! AXUIElement, kAXSelectedTextRangeAttribute as CFString, &selectedRangeValue)
        
        guard selectedRangeError == .success else {
            return nil
        }
        
        var selectedRange = CFRange()
        AXValueGetValue(selectedRangeValue as! AXValue, .cfRange, &selectedRange)
        var selectRect = CGRect()
        var selectBounds : AnyObject?
        
        // ensure selected text range is at least 1 - in order to find rect.
        if (extendRange) {
            var updatedRange = CFRangeMake(selectedRange.location, 1)
            withUnsafeMutablePointer(to: &updatedRange) { (ptr) in
                selectedRangeValue = AXValueCreate(.cfRange, ptr)
            }
        }


        let selectedBoundsError = AXUIElementCopyParameterizedAttributeValue(focusedElement as! AXUIElement, kAXBoundsForRangeParameterizedAttribute as CFString, selectedRangeValue!, &selectBounds)
        
        guard selectedBoundsError == .success else {
            return nil
        }
        
        AXValueGetValue(selectBounds as! AXValue, .cgRect, &selectRect)
        print("selected", selectRect)
         //prevent spotlight search from recieving keypresses
         guard selectRect.size.height != 30 else {
            return nil
         }
        
        // convert Quartz coordinate system to Cocoa!
        return NSRect.init(x: selectRect.origin.x,
                           y: NSMaxY(NSScreen.screens[0].frame) - selectRect.origin.y,
                           width:  selectRect.width,
                           height: selectRect.height)
       
    }
}


//func  getSelectedText() {
//
//
//      WindowManager.shared.sidebar?.webView?.loadBundleApp("autocomplete")
//
//      NSEvent.addGlobalMonitorForEvents(matching: .keyUp) { (event) in
//          print("keylogger:", event.characters, event.keyCode)
//      let buffer = KeystrokeBuffer.shared.handleKeystroke(event: event)
//          guard buffer != nil else {
//              WindowManager.shared.requestWindowUpdate()
//              return
//
//          }
//      let systemWideElement = AXUIElementCreateSystemWide()
//      var focusedElement : AnyObject?
//
//      let error = AXUIElementCopyAttributeValue(systemWideElement, kAXFocusedUIElementAttribute as CFString, &focusedElement)
//      if (error != .success){
//          print("Couldn't get the focused element. Probably a webkit application")
//      } else {
//          var selectedRangeValue : AnyObject?
//          let selectedRangeError = AXUIElementCopyAttributeValue(focusedElement as! AXUIElement, kAXSelectedTextRangeAttribute as CFString, &selectedRangeValue)
//
//          if (selectedRangeError == .success){
//              var selectedRange : CFRange?
//              AXValueGetValue(selectedRangeValue as! AXValue, AXValueType(rawValue: kAXValueCFRangeType)!, &selectedRange)
//              var selectRect = CGRect()
//              var selectBounds : AnyObject?
//
//              //kAXInsertionPointLineNumberAttribute
//              //kAXRangeForLineParameterizedAttribute
//
//              let selectedBoundsError = AXUIElementCopyParameterizedAttributeValue(focusedElement as! AXUIElement, kAXBoundsForRangeParameterizedAttribute as CFString, selectedRangeValue!, &selectBounds)
//              if (selectedBoundsError == .success){
//                  AXValueGetValue(selectBounds as! AXValue, .cgRect, &selectRect)
//                  //do whatever you want with your selectRect
//                  print("selected", selectRect)
//                  let height:CGFloat = 0 //140
//                  let translatedOrigin = NSPoint(x: selectRect.origin.x, y: (NSScreen.main?.frame.height)! - selectRect.origin.y /*- selectRect.height*/ + height + 5)
//                  if let buffer = buffer {
//                      WindowManager.shared.sidebar?.webView?.evaluateJavaScript("try{ fig.autocomplete(`\(buffer)`, -1) } catch(e){} ", completionHandler: nil)
//                  }
//                  WindowManager.shared.sidebar?.setOverlayFrame(NSRect(origin: translatedOrigin, size: CGSize(width: 200, height: height)))//140
//              }
//          }
//      }
//      }
//  }
