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

enum EventTapAction {
  case forward
  case consume
  case ignore
}

typealias EventTapHandler = (_ event: CGEvent, _ in: ExternalWindow) -> EventTapAction
protocol KeypressService {
  func register(handler: @escaping EventTapHandler )
}

protocol KeypressRedirectService {
  func addRedirect(for keycode: UInt16, in window: ExternalWindow)
  func removeRedirect(for keycode: UInt16, in window: ExternalWindow)
  
  func addRedirect(for keycode: Keystroke, in window: ExternalWindow)
  func removeRedirect(for keycode: Keystroke, in window: ExternalWindow)

  func setEnabled(value: Bool)
}

protocol EditBufferService {
  func keyBuffer(for window: ExternalWindow) -> KeystrokeBuffer
  func keyBuffer(for windowHash: ExternalWindowHash) -> KeystrokeBuffer
  func clean()
}

class KeypressProvider {
  // MARK: - Properties
  var enabled = true
  var keyHandler: Any? = nil
  var tap: CFMachPort? = nil
  var mouseHandler: Any? = nil
  let throttler = Throttler(minimumDelay: 0.05)
  private let queue = DispatchQueue(label: "com.withfig.keypress.redirects", attributes: .concurrent)
  var redirects: [ExternalWindowHash:  Set<Keystroke>] = [:]
  var buffers: [ExternalWindowHash: KeystrokeBuffer] = [:]
  fileprivate let handlers: [EventTapHandler] =
    [ Autocomplete.handleTabKey
    , Autocomplete.handleEscapeKey
    , Autocomplete.handleCommandIKey
    , KeypressProvider.processRegisteredHandlers
    , KeypressProvider.handleRedirect
    ]
  
  var registeredHandlers: [EventTapHandler] = []
  
  static var whitelist: Set<String> {
    get {
        return Integrations.terminalsWhereAutocompleteShouldAppear
    }
  }
  static let shared = KeypressProvider()
  
  // MARK: - Setup

  init() {
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
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(windowDidChange(_:)),
                                           name: AXWindowServer.windowDidChangeNotification,
                                           object: nil)
    
  }
  
  func registerKeystrokeHandler() {
    if let handler = self.mouseHandler {
      NSEvent.removeMonitor(handler)
    }
    
    self.mouseHandler = NSEvent.addGlobalMonitorForEvents(matching: .leftMouseUp) { (event) in
      if let window = AXWindowServer.shared.whitelistedWindow {
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
    
    // todo: it bothers me that this is here since wi. Should we consolidate
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
    
    let eventMask = (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.keyUp.rawValue) | (1 << CGEventType.tapDisabledByTimeout.rawValue)// | (1 << CGEventType.tapDisabledByUserInput.rawValue)
    
    let eventCallBack: CGEventTapCallBack = { (proxy, type, event, refcon) -> Unmanaged<CGEvent>? in
      
      switch event.type {
        case .tapDisabledByTimeout:
          print("eventTap: disabled by timeout")
          if let tap = KeypressProvider.shared.tap {
            CGEvent.tapEnable(tap: tap, enable: true)
          }
          return Unmanaged.passUnretained(event)

        case .tapDisabledByUserInput:
          // This is triggered if we manually disable the event tap
          print("eventTap: disabled by user input")
          return Unmanaged.passUnretained(event)
        default:
          break
      }

      // fixes slowdown when typing into Fig
      if let isFigActive = NSWorkspace.shared.frontmostApplication?.isFig, isFigActive {
        return Unmanaged.passUnretained(event)
      }
      
      // prevents keystrokes from being processed when typing into another application (specifically, spotlight)
      guard Defaults.loggedIn,
            Defaults.useAutocomplete,
            Accessibility.focusedApplicationIsSupportedTerminal() else {
        return Unmanaged.passUnretained(event)
      }
      
      guard let window = AXWindowServer.shared.whitelistedWindow else {
        print("eventTap window of \(AXWindowServer.shared.whitelistedWindow?.bundleId ?? "<none>") is not whitelisted")
        return Unmanaged.passUnretained(event)
      }
      
      var action = ""
      if (event.type == .keyDown) {
        action = "pressed "
      } else if (event.type == .keyUp) {
        action = "released"
      }
      
      
      let keyName = KeyboardLayout.humanReadableKeyName(event) ?? "?"
      
      Logger.log(message: "\(action) '\(keyName)' in \(window.bundleId ?? "<unknown>") [\(window.hash)], \(window.tty?.descriptor ?? "???") (\(window.tty?.name ?? "???"))", subsystem: .keypress)

      
      guard window.tty?.isShell ?? true else {
        print("tty: Is not in a shell")
        return Unmanaged.passUnretained(event)
      }
      
      // process handlers (order is important)
      for handler in KeypressProvider.shared.handlers {
        let action = handler(event, window)
        switch action {
          case .forward:
            return Unmanaged.passUnretained(event)
          case .consume:
            return nil
          case .ignore:
            continue
        }
      }
      
      autoreleasepool {
        KeypressProvider.shared.handleKeystroke(event: NSEvent(cgEvent: event), in: window)
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
                                                       callback: eventCallBack,
                                                       userInfo: nil) else {
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
  
    
  
  // MARK: - Notifications
  @objc func windowDidChange(_ notification: Notification) {
    guard let window = notification.object as? ExternalWindow else { return }
    
    let enabled = Integrations.terminalsWhereAutocompleteShouldAppear.contains(window.bundleId ?? "")
    print("eventTap: \(enabled)")
    if let tap = self.tap {
      // turn off event tap for windows that belong to applications
      // which are not supported terminals
      CGEvent.tapEnable(tap: tap, enable: enabled)
    }

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

  // MARK: - EventTapHandlers

  static func handleRedirect(event:CGEvent, in window: ExternalWindow) -> EventTapAction {
    var keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
    
    guard KeypressProvider.shared.shouldRedirect(event: event, in: window) else {
      return .ignore
    }
    
    // prevent redirects when typing in VSCode editor
    if Integrations.electronTerminals.contains(window.bundleId ?? "") && Accessibility.findXTermCursorInElectronWindow(window) == nil {
      return .forward
    }
    
    guard !(WindowManager.shared.autocomplete?.isHidden ?? true) else {
      return .forward
    }
    
    // fig.keypress only recieves keyDown events
    guard event.type == .keyDown else {
      Autocomplete.position(makeVisibleImmediately: false)
      return .consume
    }
    

    // todo(mschrage): This should be handled by autocomplete. Not hard coded in macOS app.
    if (keyCode == KeyboardLayout.shared.keyCode(for: "N") ?? Keycode.n) {
      keyCode = Keycode.downArrow
    }
    
    if (keyCode == KeyboardLayout.shared.keyCode(for: "P") ?? Keycode.p) {
      keyCode = Keycode.upArrow
    }
    
    if (keyCode == KeyboardLayout.shared.keyCode(for: "J") ?? Keycode.j) {
      keyCode = Keycode.downArrow
    }
    
    if (keyCode == KeyboardLayout.shared.keyCode(for: "K") ?? Keycode.k) {
      keyCode = Keycode.upArrow
    }
    
    Autocomplete.redirect(keyCode: keyCode, event: event, for: window.hash)
    return .consume
  }
  
  static func processRegisteredHandlers(event:CGEvent, in window: ExternalWindow) -> EventTapAction {
    for handler in KeypressProvider.shared.registeredHandlers {
      let action = handler(event, window)
      switch action {
        case .forward, .consume:
          return action
        case .ignore:
          continue
      }
    }
    
    return .ignore
  }
  
  func handleKeystroke(event: NSEvent?, in window: ExternalWindow) {
    
    // handle keystrokes in VSCode editor
    if Integrations.electronTerminals.contains(window.bundleId ?? "") && Accessibility.getTextRect() == nil {
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
}

// MARK: - Extensions -

extension KeypressProvider: KeypressService {
  
  func register(handler: @escaping EventTapHandler) {
    self.registeredHandlers.append(handler)
  }
}

extension KeypressProvider: EditBufferService {
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
  
  func clean() {
    buffers = [:]
  }
}

extension KeypressProvider: KeypressRedirectService {
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
  
}
