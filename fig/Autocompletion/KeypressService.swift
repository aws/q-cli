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
import FigAPIBindings

enum EventTapAction {
  case forward
  case consume
  case ignore
}

typealias EventTapHandler = (_ event: CGEvent, _ in: ExternalWindow) -> EventTapAction
protocol KeypressService {
  func register(handler: @escaping EventTapHandler )
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
  var redirectsEnabled: Bool = true
  let throttler = Throttler(minimumDelay: 0.05)
  var buffers: [ExternalWindowHash: KeystrokeBuffer] = [:]
  fileprivate let handlers: [EventTapHandler] =
    [ InputMethod.keypressTrigger
    , Autocomplete.handleShowOnTab
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
        action = "pressed"
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

  func setRedirectsEnabled(value: Bool) {
    KeypressProvider.shared.redirectsEnabled = value
  }
  
  static func handleRedirect(event:CGEvent, in window: ExternalWindow) -> EventTapAction {
    // prevent redirects when typing in VSCode editor
    guard window.isFocusedTerminal else {
      return .forward
    }
    
    guard KeypressProvider.shared.redirectsEnabled else {
      return .ignore
    }
    
    if let keybindingString = KeyboardLayout.humanReadableKeyName(event) {
      if let bindings = Settings.shared.getKeybindings(forKey: keybindingString) {
        // Right now only handle autocomplete.keybindings
        if let autocompleteBinding = bindings["autocomplete"] {
          let autocompleteIsHidden = WindowManager.shared.autocomplete?.isHidden ?? true

          guard autocompleteBinding.contains("--global") || !autocompleteIsHidden else {
            return .ignore
          }
          
          guard autocompleteBinding != "ignore" else {
            return .ignore
          }
          
          // fig.keypress only recieves keyDown events
          guard event.type == .keyDown else {
            Autocomplete.position(makeVisibleImmediately: false)
            return .consume
          }
          
          Logger.log(message: "Redirecting keypress '\(keybindingString)' to autocomplete", subsystem: .keypress)
          let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
            
          // Legacy fig.keypress implementation
          Autocomplete.redirect(keyCode: keyCode, event: event, for: window.hash)
          
          // Protobuf API implementation
          API.notifications.post(Fig_KeybindingPressedNotification.with {
              if let action = autocompleteBinding.split(separator: " ").first {
                  $0.action = String(action)
              }
              
              if let event = NSEvent(cgEvent: event) {
                  $0.keypress = event.fig_keyEvent
              }
          })
            
          return .consume
        }
      }
    }
    
    return .ignore
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
    guard window.isFocusedTerminal else {
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
