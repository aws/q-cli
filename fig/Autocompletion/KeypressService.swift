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
import Sentry

protocol KeypressService {
    func keyBuffer(for window: ExternalWindow) -> KeystrokeBuffer
    func keyBuffer(for windowHash: ExternalWindowHash) -> KeystrokeBuffer

//    func redirects(for window: ExternalWindow) -> Set<UInt16>

    func getTextRect(extendRange: Bool) -> CGRect?
    func clean()
    func addRedirect(for keycode: UInt16, in window: ExternalWindow)
    func removeRedirect(for keycode: UInt16, in window: ExternalWindow)
    func setEnabled(value: Bool)
}

class KeypressProvider : KeypressService {
    static let shared = KeypressProvider(windowServiceProvider: WindowServer.shared)
    let windowServiceProvider: WindowService
    static let whitelist = Integrations.nativeTerminals
    var buffers: [ExternalWindowHash: KeystrokeBuffer] = [:]
    let throttler = Throttler(minimumDelay: 0.05)
    let keyThrottler = Throttler(minimumDelay: 0.75)

    
    var mouseHandler: Any? = nil
    var keyHandler: Any? = nil
    var tap: CFMachPort? = nil

    
//    fileprivate var redirects: Set<UInt16> = []
    var redirects: [ExternalWindowHash:  Set<UInt16>] = [:]
    var enabled = true

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
    
    func setEnabled(value: Bool) {
        self.enabled = value
    }

    
    init(windowServiceProvider: WindowService) {
        self.windowServiceProvider = windowServiceProvider
        registerKeystrokeHandler()
        
        NotificationCenter.default.addObserver(self, selector:#selector(lineAcceptedInKeystrokeBuffer), name: KeystrokeBuffer.lineResetInKeyStrokeBufferNotification, object:nil)
        NotificationCenter.default.addObserver(self, selector:#selector(firstCharacterInKeystrokeBuffer), name: KeystrokeBuffer.firstCharacterInKeystrokeBufferNotification, object:nil)
    }
    
    @objc func firstCharacterInKeystrokeBuffer() {
//        if let window = AXWindowServer.shared.whitelistedWindow, let tty = window.tty {
//            DispatchQueue.global(qos: .userInteractive).async {
//                tty.update()
//            }
//        }
    }

    @objc func lineAcceptedInKeystrokeBuffer() {
        if let window = AXWindowServer.shared.whitelistedWindow, let tty = window.tty {
            Timer.delayWithSeconds(0.2) {
                DispatchQueue.global(qos: .userInteractive).async {
                    tty.update()
                }
            }
        }
    }
    
    func registerKeystrokeHandler() {
        if let handler = self.mouseHandler {
            NSEvent.removeMonitor(handler)
        }

        self.mouseHandler = NSEvent.addGlobalMonitorForEvents(matching: .leftMouseUp) { (event) in
            if let window = self.windowServiceProvider.topmostWhitelistedWindow(), KeypressProvider.whitelist.contains(window.bundleId ?? "") {
               
                // option click, moves cursor to unknown location
                if (event.modifierFlags.contains(.option)) {
                    let keyBuffer = self.keyBuffer(for: window)
                    keyBuffer.buffer = nil
                }
           }
       }
        
        if let handler = self.keyHandler {
            NSEvent.removeMonitor(handler)
        }

        self.keyHandler = NSEvent.addGlobalMonitorForEvents(matching: [ .keyUp], handler: { (event) in
            guard Defaults.useAutocomplete else { return }
            guard event.keyCode == Keycode.returnKey || event.modifierFlags.contains(.control) else { return }
            if let window = AXWindowServer.shared.whitelistedWindow, let tty = window.tty {
                Timer.delayWithSeconds(0.2) {
                    DispatchQueue.global(qos: .userInteractive).async {
                        tty.update()
                    }
                }
            }
        })
        
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
        guard AXIsProcessTrustedWithOptions(nil) else {
            print("KeypressService: Could not register without accesibility permissions")
            return nil
        }
        let eventMask = (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.keyUp.rawValue) | (1 << CGEventType.tapDisabledByTimeout.rawValue) | (1 << CGEventType.tapDisabledByUserInput.rawValue)
        
        // not sure what the difference is between passRetained vs passUnretained?
        guard let eventTap: CFMachPort = CGEvent.tapCreate(tap: CGEventTapLocation.cghidEventTap,
                                                     place: CGEventTapPlacement.tailAppendEventTap,
                                                     options: CGEventTapOptions.defaultTap,
                                                     eventsOfInterest: CGEventMask(eventMask),
                                                     callback: { (proxy, type, event, refcon) -> Unmanaged<CGEvent>? in
//                                                        return Unmanaged.passRetained(event)
                                                        print("Keystroke event!")
                                                           print("eventTap", event.getIntegerValueField(.eventTargetUnixProcessID))


                                                        guard event.type != .tapDisabledByTimeout else {
                                                            if let tap = KeypressProvider.shared.tap {
                                                                CGEvent.tapEnable(tap: tap, enable: true)
                                                                SentrySDK.capture(message: "tapDisabledByTimeout")

                                                            }
                                                            return Unmanaged.passUnretained(event)
                                                        }
                                                        
                                                        guard event.type != .tapDisabledByUserInput else {
                                                            if let tap = KeypressProvider.shared.tap {
                                                                CGEvent.tapEnable(tap: tap, enable: true)
                                                                SentrySDK.capture(message: "tapDisabledByUserInput")

                                                            }
                                                            return Unmanaged.passUnretained(event)
                                                        }
                                                        
//                                                        let keyCode2 = event.getIntegerValueField(.keyboardEventKeycode)
//                                                        guard [Keycode.upArrow, Keycode.downArrow, Keycode.returnKey, Keycode.tab, Keycode.escape].contains(UInt16(keyCode2)) else {
//                                                               return Unmanaged.passRetained(event)
//                                                           }
                                                        
                                                        // fixes slowdown when typing into Fig
                                                        guard !(NSWorkspace.shared.frontmostApplication?.isFig ?? false) else {
                                                            return Unmanaged.passUnretained(event)
                                                        }
                                                        
                                                        guard Defaults.loggedIn, Defaults.useAutocomplete, let window = AXWindowServer.shared.whitelistedWindow, KeypressProvider.whitelist.contains(window.bundleId ?? "") else {
                                                            print("eventTap window of \(AXWindowServer.shared.whitelistedWindow?.bundleId ?? "<none>") is not whitelisted")
                                                            return Unmanaged.passUnretained(event)
                                                        }
                                                        
                                                        print("tty: hash = \(window.hash) tty = \(window.tty?.descriptor) pwd = \(window.tty?.cwd ?? "<none>") \(window.tty?.isShell ?? true ? "shell!" : "not shell")")
                                                        
                                                        guard window.tty?.isShell ?? true else {
                                                            print("tty: Is not in a shell")
                                                            return Unmanaged.passUnretained(event)
                                                        }
                                                        
                                                        if [.keyDown , .keyUp].contains(type) {
                                                            let keyCode = event.getIntegerValueField(.keyboardEventKeycode)
                                                            print("eventTap", keyCode, event.getIntegerValueField(.eventTargetUnixProcessID))
                                                            print("eventTap", "\(window.hash)")

                                                            if (type == .keyDown && KeypressProvider.shared.enabled && KeypressProvider.shared.redirects[window.hash]?.contains(UInt16(keyCode)) ?? false &&
                                                                !event.flags.contains(.maskCommand)
) {
                                                            
                                                                
                                                                print("eventTap", "Should redirect!")
                                                                WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.keypress(\"\(keyCode)\", \"\(window.hash)\") } catch(e) {}", completionHandler: nil)
                                                                
                                                                
                                                                
                                                                return nil
                                                            } else {
                                                                autoreleasepool {
                                                                    KeypressProvider.shared.handleKeystroke(event: NSEvent(cgEvent: event), in: window)
                                                                }

//                                                                DispatchQueue.global(qos: .background).async {
                                                                //KeypressProvider.shared.handleKeystroke(event: NSEvent.init(cgEvent: event), in: window)
//                                                                }
                                                                
                                                               

                                                            }
                                                            //event.setIntegerValueField(.keyboardEventKeycode, value: keyCode)
                                                        }
                                                        return Unmanaged.passUnretained(event) },
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
        return self.keyBuffer(for: window.hash)
    }
    
    func keyBuffer(for windowHash: ExternalWindowHash) -> KeystrokeBuffer {
        if let buffer = self.buffers[windowHash] {
            return buffer
        } else {
            let buffer = KeystrokeBuffer()
            self.buffers[windowHash] = buffer
            return buffer
        }
    }
    
    func handleKeystroke(event: NSEvent?, in window: ExternalWindow) {
//        if let rect = getTextRect() {
            let keyBuffer = self.keyBuffer(for: window)
        if let event = event, event.type == NSEvent.EventType.keyDown {
                    let tty = window.tty?.descriptor == nil ? "null" : "'\(window.tty!.descriptor)'"
                    let cmd = window.tty?.cmd == nil ? "null" : "'\(window.tty!.cmd!)'"
                    let cwd = window.tty?.cwd == nil ? "null" : "`\(window.tty!.cwd!.trimmingCharacters(in: .whitespacesAndNewlines))`"

//                    let process: proc? = tty?.running
//                       guard process?.isShell ?? true else {
//                           //keyBuffer.buffer = nil
//                           return
//                       }

                    if let (buffer, index) = keyBuffer.handleKeystroke(event: event),
                        let b64 = buffer.data(using: .utf8)?.base64EncodedString() {

                        WindowManager.shared.autocomplete?.tetheredWindow = window
                        // error here!
                        // fig.autocomplete(bufferB64, index, windowHash, tty?, cwd, cmd)
                        print("fig.autocomplete(b64DecodeUnicode(`\(b64)`), \(index), '\(window.hash)', \(tty), \(cwd), \(cmd))")
                        WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.autocomplete(b64DecodeUnicode(`\(b64)`), \(index), '\(window.hash)', \(tty), \(cwd), \(cmd)) } catch(e){} ", completionHandler: nil)

                       
                    } else {

                        WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.nocontext('\(window.hash)') } catch(e){} ", completionHandler: nil)
                    }
   

            }

        self.throttler.throttle {

            if let rect = self.getTextRect() {
                WindowManager.shared.positionAutocompletePopover(textRect: rect)

                    
            } else {
                KeypressProvider.shared.removeRedirect(for: Keycode.upArrow, in: window)
                KeypressProvider.shared.removeRedirect(for: Keycode.downArrow, in: window)
                KeypressProvider.shared.removeRedirect(for: Keycode.tab, in: window)
                KeypressProvider.shared.removeRedirect(for: Keycode.escape, in: window)
                KeypressProvider.shared.removeRedirect(for: Keycode.returnKey, in: window)
                
            }
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
         //prevent spotlight search from recieving keypresses, this is sooo hacky
         guard selectRect.size.height != 30 else {
            return nil
         }
        
        // Sanity check: prevents flashing autocomplete in bottom corner
        guard selectRect.size != .zero else {
            return nil
        }
        
        // convert Quartz coordinate system to Cocoa!
        return NSRect.init(x: selectRect.origin.x,
                           y: NSMaxY(NSScreen.screens[0].frame) - selectRect.origin.y,
                           width:  selectRect.width,
                           height: selectRect.height)
       
    }
}

class Throttler {

    private var workItem: DispatchWorkItem = DispatchWorkItem(block: {})
    private var previousRun: Date = Date.distantPast
    private let queue: DispatchQueue
    private let minimumDelay: TimeInterval

    init(minimumDelay: TimeInterval, queue: DispatchQueue = DispatchQueue.main) {
        self.minimumDelay = minimumDelay
        self.queue = queue
    }

    func throttle(_ block: @escaping () -> Void) {
        // Cancel any existing work item if it has not yet executed
        workItem.cancel()

        // Re-assign workItem with the new block task, resetting the previousRun time when it executes
        workItem = DispatchWorkItem() {
            [weak self] in
            self?.previousRun = Date()
            block()
        }

        // If the time since the previous run is more than the required minimum delay
        // => execute the workItem immediately
        // else
        // => delay the workItem execution by the minimum delay time
        let delay = previousRun.timeIntervalSinceNow > minimumDelay ? 0 : minimumDelay
        queue.asyncAfter(deadline: .now() + Double(delay), execute: workItem)
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
