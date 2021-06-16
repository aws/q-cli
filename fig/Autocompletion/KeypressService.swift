//
//  KeypressService.swift
//  fig
//
//  Created by Matt Schrage on 9/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
import Carbon
import Sentry
import Foundation
import AXSwift

protocol KeypressService {
  func keyBuffer(for window: ExternalWindow) -> KeystrokeBuffer
  func keyBuffer(for windowHash: ExternalWindowHash) -> KeystrokeBuffer
  func getTextRect(extendRange: Bool) -> CGRect?
  func clean()
  func addRedirect(for keycode: UInt16, in window: ExternalWindow)
  func removeRedirect(for keycode: UInt16, in window: ExternalWindow)
  
  func addRedirect(for keycode: Keystroke, in window: ExternalWindow)
  func removeRedirect(for keycode: Keystroke, in window: ExternalWindow)

  func setEnabled(value: Bool)
}

class KeypressProvider : KeypressService {
  var enabled = true
  var keyHandler: Any? = nil
  var tap: CFMachPort? = nil
  var mouseHandler: Any? = nil
  let windowServiceProvider: WindowService
  let throttler = Throttler(minimumDelay: 0.05)
  private let queue = DispatchQueue(label: "com.withfig.keypress.redirects", attributes: .concurrent)
  var redirects: [ExternalWindowHash:  Set<Keystroke>] = [:]
  var buffers: [ExternalWindowHash: KeystrokeBuffer] = [:]
  
  static var whitelist: Set<String> {
    get {
        return Integrations.terminalsWhereAutocompleteShouldAppear
    }
  }
  static let shared = KeypressProvider(windowServiceProvider: WindowServer.shared)
  
  init(windowServiceProvider: WindowService) {
    self.windowServiceProvider = windowServiceProvider
    registerKeystrokeHandler()
    NotificationCenter.default.addObserver(self,
                                           selector:#selector(lineAcceptedInKeystrokeBuffer),
                                           name: KeystrokeBuffer.lineResetInKeyStrokeBufferNotification,
                                           object:nil)
    
    NotificationCenter.default.addObserver(self,
                                           selector:#selector(accesibilityPermissionsUpdated),
                                           name: Accessibility.permissionDidUpdate,
                                           object:nil)
    
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(inputSourceChanged),
                                           name: KeyboardLayout.keyboardLayoutDidChangeNotification,
                                           object: nil)
    
  }
  func shouldRedirect(event: CGEvent, in window: ExternalWindow) -> Bool {
    
    guard KeypressProvider.shared.enabled else {
      return false
    }
    
    let keystroke = Keystroke.from(event: event)
    var windowHasRedirect: Bool!
    queue.sync {
      windowHasRedirect = self.redirects[window.hash]?.contains(keystroke) ?? false
    }
    
    return windowHasRedirect
  }
  
  func addRedirect(for keycode: Keystroke, in window: ExternalWindow) {
    queue.async(flags: .barrier) {
      var set = self.redirects[window.hash] ?? []
      set.insert(keycode)
      self.redirects[window.hash] = set
    }
  }
  
  func removeRedirect(for keycode: Keystroke, in window: ExternalWindow) {
    queue.async(flags: .barrier) {
      if var set = self.redirects[window.hash] {
        set.remove(keycode)
        self.redirects[window.hash] = set
      }
    }
  }
  
  func addRedirect(for keycode: UInt16, in window: ExternalWindow) {
    self.addRedirect(for: Keystroke(keyCode: keycode), in: window)
  }
  
  func removeRedirect(for keycode: UInt16, in window: ExternalWindow) {
    self.removeRedirect(for: Keystroke(keyCode: keycode), in: window)
  }
  
  func resetRedirects(for window: ExternalWindow) {
    queue.async(flags: .barrier) {
      self.redirects[window.hash] = []
    }
  }
  
  func resetAllRedirects() {
    queue.async(flags: .barrier) {
      self.redirects = [:]
    }
  }
  
  func setEnabled(value: Bool) {
    self.enabled = value
  }
  
  @objc func inputSourceChanged() {
    resetAllRedirects()
    Autocomplete.position(makeVisibleImmediately: false, completion: nil)
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
  
  @objc func accesibilityPermissionsUpdated(_ notification: Notification) {
    guard let granted = notification.object as? Bool else { return }
    
    if (granted) {
      self.registerKeystrokeHandler()
    } else {
      self.deregisterKeystrokeHandler()
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
    
    self.keyHandler = NSEvent.addGlobalMonitorForEvents(matching: [ .keyDown, .keyUp], handler: { (event) in
      guard Defaults.useAutocomplete else { return }
      
      switch event.type {
        case .keyDown:
          // Watch for Cmd+V and manually update ZLE buffer (because we don't recieve an event until the following keystroke)
          if AXWindowServer.shared.whitelistedWindow != nil, event.keyCode == Keycode.v && event.modifierFlags.contains(.command) {
              print("ZLE: Command+V")
              ZLEIntegration.paste()
          }
          
          // Handle Control+R searching -- this is needed for ZLE + fzf, normal history search is handled by integration.
          if AXWindowServer.shared.whitelistedWindow != nil, event.keyCode == Keycode.r && event.modifierFlags.contains(.control) {
            Autocomplete.hide()
          }
        case .keyUp:
          guard event.keyCode == Keycode.returnKey || event.modifierFlags.contains(.control) else { return }
          if let window = AXWindowServer.shared.whitelistedWindow, let tty = window.tty {
            Timer.delayWithSeconds(0.2) {
              DispatchQueue.global(qos: .userInteractive).async {
                tty.update()
              }
            }
          }
        default:
          print("Unknown keypress event")
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
  
  func deregisterKeystrokeHandler() {
    if let handler = self.mouseHandler {
      NSEvent.removeMonitor(handler)
      self.mouseHandler = nil
    }
    
    if let handler = self.keyHandler {
      NSEvent.removeMonitor(handler)
      self.keyHandler = nil
    }
    
    self.clean()
    
    if let tap = self.tap {
      CFMachPortInvalidate(tap)
      self.tap = nil
    }
  }
    
  func registerKeyInterceptor() -> CFMachPort? {
    guard AXIsProcessTrustedWithOptions(nil) else {
      print("KeypressService: Could not register without accesibility permissions")
      return nil
    }
    
    let eventMask = (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.keyUp.rawValue) | (1 << CGEventType.tapDisabledByTimeout.rawValue) | (1 << CGEventType.tapDisabledByUserInput.rawValue)
    
    // not sure what the difference is between passRetained vs passUnretained?
    let eventCallBack: CGEventTapCallBack = { (proxy, type, event, refcon) -> Unmanaged<CGEvent>? in
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
      
      // fixes slowdown when typing into Fig
      guard !(NSWorkspace.shared.frontmostApplication?.isFig ?? false) else {
        return Unmanaged.passUnretained(event)
      }
      
      // prevents keystrokes from being processed when typing into another application (specifically, spotlight)
      guard Accessibility.focusedApplicationIsSupportedTerminal() else {
          return Unmanaged.passUnretained(event)
      }
      
      guard Defaults.loggedIn, Defaults.useAutocomplete, let window = AXWindowServer.shared.whitelistedWindow else {
        print("eventTap window of \(AXWindowServer.shared.whitelistedWindow?.bundleId ?? "<none>") is not whitelisted")
        return Unmanaged.passUnretained(event)
      }
      
      print("tty: hash = \(window.hash) tty = \(window.tty?.descriptor ?? "nil") pwd = \(window.tty?.cwd ?? "<none>") \(window.tty?.isShell ?? true ? "shell!" : "not shell")")
      
      guard window.tty?.isShell ?? true else {
        print("tty: Is not in a shell")
        return Unmanaged.passUnretained(event)
      }
      
      
      switch KeypressProvider.shared.handleTabKey(event: event, in: window) {
        case .forward:
          return Unmanaged.passUnretained(event)
        case .consume:
          return nil
        case .ignore:
          break
      }
      
      // Toggle autocomplete on and off
      switch KeypressProvider.shared.handleEscapeKey(event: event, in: window) {
        case .forward:
          return Unmanaged.passUnretained(event)
        case .consume:
          return nil
        case .ignore:
          break
      }
      
      if [.keyDown , .keyUp].contains(type) {
        var keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
        print("eventTap", keyCode, event.getIntegerValueField(.eventTargetUnixProcessID))
        print("eventTap", "\(window.hash)")
        if KeypressProvider.shared.shouldRedirect(event: event, in: window) {
          print("eventTap", "Should redirect!")
          // prevent redirects when typing in VSCode editor
          if Integrations.electronTerminals.contains(window.bundleId ?? "") && Accessibility.findXTermCursorInElectronWindow(window) == nil {
            return Unmanaged.passUnretained(event)
          }
          
          guard WindowManager.shared.autocomplete?.isVisible ?? true else {
            return Unmanaged.passUnretained(event)
          }
          
          // fig.keypress only recieves keyDown events
          guard event.type == .keyDown else {
            Autocomplete.position(makeVisibleImmediately: false)
            return nil
          }

          if (keyCode == KeyboardLayout.shared.keyCode(for: "N") ?? Keycode.n) {
            keyCode = Keycode.downArrow
          }
          
          if (keyCode == KeyboardLayout.shared.keyCode(for: "P") ?? Keycode.p) {
            keyCode = Keycode.upArrow
          }
          
          WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.keypress(\"\(keyCode)\", \"\(window.hash)\", { command: \(event.flags.contains(.maskCommand)), control: \(event.flags.contains(.maskControl)), shift: \(event.flags.contains(.maskShift)) }) } catch(e) {}", completionHandler: nil)
          return nil
        } else {
          
          guard !FishIntegration.handleKeystroke(event: NSEvent(cgEvent: event), in: window) else {
            return Unmanaged.passUnretained(event)
          }
          
          autoreleasepool {
            KeypressProvider.shared.handleKeystroke(event: NSEvent(cgEvent: event), in: window)
          }
        }
      }
      return Unmanaged.passUnretained(event)
    }
    
    // Switching to CGEventTapLocation.cgAnnotatedSessionEventTap allows virtual keystrokes to be detected
    // But prevents us from seeing keypresses handled by other apps (like Spectacle)
    let tapLocation = Settings.shared.getValue(forKey: Settings.eventTapLocation) as? String == "session" ? CGEventTapLocation.cgAnnotatedSessionEventTap : CGEventTapLocation.cghidEventTap
    guard let eventTap: CFMachPort = CGEvent.tapCreate(tap: tapLocation,
                                                       place: CGEventTapPlacement.tailAppendEventTap,
                                                       options: CGEventTapOptions.defaultTap,
                                                       eventsOfInterest: CGEventMask(eventMask),
                                                       callback: eventCallBack, userInfo: nil) else {
      print("Could not create tap")
      SentrySDK.capture(message: "Could not create event tap")
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
  
  enum EventTapAction {
    case forward
    case consume
    case ignore
  }
  
  func handleTabKey(event:CGEvent, in window: ExternalWindow) -> EventTapAction {
    let keycode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
    guard Keycode.tab == keycode else {
      return .ignore
    }
    
    guard [.keyDown].contains(event.type) else {
      return .ignore
    }
    
    let autocompleteIsNotVisible = !(WindowManager.shared.autocomplete?.isVisible ?? false)

    let onlyShowOnTab = (Settings.shared.getValue(forKey: Settings.onlyShowOnTabKey) as? Bool) ?? false
    
    // if not enabled or if autocomplete is already visible, handle normally
    if !onlyShowOnTab || !autocompleteIsNotVisible {
      return .ignore
    }
    
    // toggle autocomplete on and consume tab keypress
    Autocomplete.toggle(for: window)
    return .consume
    
  }
  
  func handleEscapeKey(event:CGEvent, in window: ExternalWindow) -> EventTapAction {
    let keycode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
    guard Keycode.escape == keycode else {
      return .ignore
    }
    
    guard [.keyDown].contains(event.type) else {
      return .ignore
    }
    
    let autocompleteIsNotVisible = !(WindowManager.shared.autocomplete?.isVisible ?? false)
        
    // Don't intercept escape key when in VSCode editor
    if Integrations.electronTerminals.contains(window.bundleId ?? "") &&
        Accessibility.findXTermCursorInElectronWindow(window) == nil {
      return .forward
    }
    
    // Send <esc> key event directly to underlying app, if autocomplete is hidden and no modifiers
    if autocompleteIsNotVisible, !event.flags.containsKeyboardModifier {
      return .forward
    }
    
    // Allow user to opt out of escape key being intercepted by Fig
    if let behavior = Settings.shared.getValue(forKey: Settings.escapeKeyBehaviorKey) as? String,
       behavior == "ignore",
       !event.flags.containsKeyboardModifier {
        return .forward
    }
    
    // control+esc toggles autocomplete on and off
    Autocomplete.toggle(for: window)
    
    return WindowManager.shared.autocomplete?.isVisible ?? false ? .consume : .forward

  }
  
  func handleKeystroke(event: NSEvent?, in window: ExternalWindow) {
    
    // handle keystrokes in VSCode editor
    if Integrations.electronTerminals.contains(window.bundleId ?? "") && self.getTextRect() == nil {
        return
    }
    

    let keyBuffer = self.keyBuffer(for: window)
    guard !keyBuffer.backedByShell else {
      
      
      // trigger positioning updates for hotkeys, like cmd+w, cmd+t, cmd+n, or Spectacle
      if let event = event {
        
        if event.keyCode == KeyboardLayout.shared.keyCode(for: "W") && event.modifierFlags.contains(.command) {
          Autocomplete.hide()
        } else if event.modifierFlags.contains(.command) || event.modifierFlags.contains(.option) {
          Autocomplete.position()

        }
      }
      
      return
    }

    if let event = event, event.type == NSEvent.EventType.keyDown {
      Autocomplete.update(with: keyBuffer.handleKeystroke(event: event), for: window.hash)
    }
    
    Autocomplete.position()
 
  }
  
  func getTextRect(extendRange: Bool = true) -> CGRect? {
    
    // prevent cursor position for being returned when apps like spotlight & alfred are active
    
    guard Accessibility.focusedApplicationIsSupportedTerminal() else {
        return nil
    }
    
    if let window = AXWindowServer.shared.whitelistedWindow, Integrations.electronTerminals.contains(window.bundleId ?? "") {
      return Accessibility.findXTermCursorInElectronWindow(window)
    }
    
    let systemWideElement = AXUIElementCreateSystemWide()
    var focusedElement : AnyObject?
    let error = AXUIElementCopyAttributeValue(systemWideElement, kAXFocusedUIElementAttribute as CFString, &focusedElement)
    guard error == .success else {
      print("cursor: Couldn't get the focused element. Probably a webkit application")
      return nil
    }
    
    var selectedRangeValue : AnyObject?
    let selectedRangeError = AXUIElementCopyAttributeValue(focusedElement as! AXUIElement, kAXSelectedTextRangeAttribute as CFString, &selectedRangeValue)
    
    guard selectedRangeError == .success else {
      print("cursor: couldn't get selected range")
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
    
    // https://linear.app/fig/issue/ENG-109/ - autocomplete-popup-shows-when-copying-and-pasting-in-terminal
    if selectedRange.length > 1 {
      print("cursor: selectedRange length > 1")
      return nil
    }
    
    let selectedBoundsError = AXUIElementCopyParameterizedAttributeValue(focusedElement as! AXUIElement, kAXBoundsForRangeParameterizedAttribute as CFString, selectedRangeValue!, &selectBounds)
    
    guard selectedBoundsError == .success else {
      print("cursor: selectedBoundsError")
      return nil
    }
    
    AXValueGetValue(selectBounds as! AXValue, .cgRect, &selectRect)
    print("selected", selectRect)
    //prevent spotlight search from recieving keypresses, this is sooo hacky
//    guard selectRect.size.height != 30 else {
//      print("cursor: prevent spotlight search from recieving keypresses, this is sooo hacky")
//      return nil
//    }
    
    // Sanity check: prevents flashing autocomplete in bottom corner
    guard selectRect.size != .zero else {
      print("cursor: prevents flashing autocomplete in bottom corner")
      return nil
    }
    
    // convert Quartz coordinate system to Cocoa!
    return NSRect(x: selectRect.origin.x, y: NSMaxY(NSScreen.screens[0].frame) - selectRect.origin.y, width:  selectRect.width, height: selectRect.height)
  }
}

class Throttler {
  private var workItem: DispatchWorkItem = DispatchWorkItem(block: {})
  private var previousRun: Date = Date.distantPast
  private let queue: DispatchQueue
  private let minimumDelay: TimeInterval
  
  init(minimumDelay: TimeInterval, queue: DispatchQueue = DispatchQueue(label: "com.withfig.keyhandler", qos: .userInitiated)) {
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
