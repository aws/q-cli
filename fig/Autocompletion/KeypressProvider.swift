//
//  KeypressProvider.swift
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

class KeypressProvider {
  // MARK: - Properties
  var enabled = true
  var keyHandler: Any?
  var tap: CFMachPort?
  var redirectsEnabled: Bool = true
  var globalKeystrokeInterceptsEnabled: Bool = false
  var registeredHandlers: [EventTapHandler] = [ InputMethod.keypressTrigger ]

  func register(handler: @escaping EventTapHandler) {
    self.registeredHandlers.append(handler)
  }

  static var allowlist: Set<String> {
    return Integrations.terminalsWhereAutocompleteShouldAppear
  }
  static let shared = KeypressProvider()

  // MARK: - Setup

  init() {
    registerKeystrokeHandler()

    NotificationCenter.default.addObserver(self,
                                           selector: #selector(accesibilityPermissionsUpdated),
                                           name: Accessibility.permissionDidUpdate,
                                           object: nil)

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
    deregisterKeystrokeHandler()

    // todo: it bothers me that this is here since wi. Should we consolidate
    self.keyHandler = NSEvent.addGlobalMonitorForEvents(matching: [ .keyDown, .keyUp], handler: { (event) in
      guard Defaults.shared.useAutocomplete else { return }

      switch event.type {
      case .keyDown:
        // Handle Control+R searching -- this is needed for ZLE + fzf, normal history search is handled by integration.
        if AXWindowServer.shared.allowlistedWindow != nil,
           event.keyCode == Keycode.r.rawValue,
           event.modifierFlags.contains(.control) {
          Autocomplete.hide()
        }
      case .keyUp:
        break
      default:
        print("Unknown keypress event")
      }
    })

    if let tap = registerKeyInterceptor() {
      self.tap = tap
    }
  }

  func deregisterKeystrokeHandler() {
    if let handler = self.keyHandler {
      NSEvent.removeMonitor(handler)
      self.keyHandler = nil
    }

    if let tap = self.tap {
      CFMachPortInvalidate(tap)
      self.tap = nil
    }
  }

  func registerKeyInterceptor() -> CFMachPort? {
    guard AXIsProcessTrustedWithOptions(nil) else {
      print("KeypressProvider: Could not register without accesibility permissions")
      return nil
    }

    let eventMask = (1 << CGEventType.keyDown.rawValue)
      | (1 << CGEventType.keyUp.rawValue)
      | (1 << CGEventType.tapDisabledByTimeout.rawValue)
    // | (1 << CGEventType.tapDisabledByUserInput.rawValue)

    let eventCallBack: CGEventTapCallBack = { (_, type, event, _) -> Unmanaged<CGEvent>? in

      switch event.type {
      case .tapDisabledByTimeout:
        Logger.log(message: "disabled by timeout", subsystem: .keypress)
        if let tap = KeypressProvider.shared.tap {
          CGEvent.tapEnable(tap: tap, enable: true)
        }
        return Unmanaged.passUnretained(event)

      case .tapDisabledByUserInput:
        // This is triggered if we manually disable the event tap
        Logger.log(message: "eventTap disabled by user input", subsystem: .keypress)
        return Unmanaged.passUnretained(event)
      default:
        break
      }

      // fixes slowdown when typing into Fig
      if let isFigActive = NSWorkspace.shared.frontmostApplication?.isFig, isFigActive {
        return Unmanaged.passUnretained(event)
      }

      // prevents keystrokes from being processed when typing into another application (specifically, spotlight)
      guard Defaults.shared.loggedIn,
            Defaults.shared.useAutocomplete,
            Accessibility.focusedApplicationIsSupportedTerminal() else {

        let conditions = "loggedIn: \(Defaults.shared.loggedIn)"
                       + ", useAutocomplete: \(Defaults.shared.useAutocomplete)"
                       + ", focusedApplicationIsSupportedTerminal: \(Accessibility.focusedApplicationIsSupportedTerminal())"
        Logger.log(message: "Ignoring keypress! \(conditions)",
                   subsystem: .keypress)

        return Unmanaged.passUnretained(event)
      }

      guard let window = AXWindowServer.shared.allowlistedWindow else {
        let message = "eventTap window of \(AXWindowServer.shared.allowlistedWindow?.bundleId ?? "<none>") is not allowlisted"
        Logger.log(message: message,
                   subsystem: .keypress)
        return Unmanaged.passUnretained(event)
      }

      var action = ""
      if event.type == .keyDown {
        action = "pressed"
      } else if event.type == .keyUp {
        action = "released"
      }

      let keyName = KeyboardLayout.humanReadableKeyName(event) ?? "?"
      // swiftlint:disable line_length
      Logger.log(message: "\(action) '\(keyName)' in \(window.bundleId ?? "<unknown>") [\(window.hash)], \(window.associatedShellContext?.ttyDescriptor ?? "???") (\(window.associatedShellContext?.executablePath ?? "???")) \(window.associatedShellContext?.processId ?? 0)", subsystem: .keypress)

      let handlers = KeypressProvider.shared.registeredHandlers + [ KeypressProvider.fallbackHandler ]
      // process handlers (order is important)
      for handler in handlers {
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
    let tapLocation = Settings.shared.getValue(forKey: Settings.eventTapLocation) as? String == "session"
      ? CGEventTapLocation.cgAnnotatedSessionEventTap
      : CGEventTapLocation.cghidEventTap
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
    // CFRunLoopRun()
    return eventTap
  }

  // MARK: - Notifications
  @objc func windowDidChange(_ notification: Notification) {
    guard let window = notification.object as? ExternalWindow else { return }

    let enabled = Integrations.bundleIsValidTerminal(window.bundleId)
    print("eventTap: \(enabled)")
    if let tap = self.tap {
      // turn off event tap for windows that belong to applications
      // which are not supported terminals
      CGEvent.tapEnable(tap: tap, enable: enabled)
    }

  }

  @objc func inputSourceChanged() {
    Autocomplete.position(makeVisibleImmediately: false)
  }

  @objc func accesibilityPermissionsUpdated(_ notification: Notification) {
    guard let granted = notification.object as? Bool else { return }

    if granted {
      self.registerKeystrokeHandler()
    } else {
      self.deregisterKeystrokeHandler()
    }
  }

  // MARK: - EventTapHandlers

  func setRedirectsEnabled(value: Bool) {
    KeypressProvider.shared.redirectsEnabled = value
  }

  func setGlobalKeystrokeInterceptsEnabled(value: Bool) {
    KeypressProvider.shared.globalKeystrokeInterceptsEnabled = value
  }

  static func fallbackHandler(event: CGEvent, in window: ExternalWindow) -> EventTapAction {
    // prevent redirects when typing in VSCode editor
    guard window.isFocusedTerminal else {
      return .forward
    }

    guard let context = window.associatedShellContext, context.isShell() else {
      return .forward
    }

    // Ensure we have a valid keybinding string that has a bound keybindings.
    guard let keybindingString = KeyboardLayout.humanReadableKeyName(event),
          let bindings = Settings.shared.getKeybindings(forKey: keybindingString) else {
      return .ignore
    }

    // Right now only handle autocomplete.keybindings
    guard let autocompleteBinding = bindings["autocomplete"] else {
      return .ignore
    }

    let autocompleteIsHidden = WindowManager.shared.autocomplete?.isHidden ?? true
    let action = autocompleteBinding.split(separator: " ").first
    let onlyShowOnTab = Settings.shared.getValue(forKey: Settings.onlyShowOnTabKey) as? Bool ?? false

    let isGlobalAction = KeypressProvider.shared.globalKeystrokeInterceptsEnabled && (
      keybindingString == "tab" && onlyShowOnTab ||
        autocompleteBinding.contains("--global") ||
        Autocomplete.globalActions.contains(String(action ?? ""))
    )

    guard isGlobalAction || !autocompleteIsHidden else {
      return .ignore
    }

    guard isGlobalAction || KeypressProvider.shared.redirectsEnabled else {
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
      if let action = action {
        $0.action = String(action)
      }

      if let event = NSEvent(cgEvent: event) {
        $0.keypress = event.fig_keyEvent
      }
    })

    return .consume
  }

  func handleKeystroke(event: NSEvent?, in window: ExternalWindow) {
    // handle keystrokes in VSCode editor
    guard window.isFocusedTerminal else {
      return
    }

    // trigger positioning updates for hotkeys, like cmd+w, cmd+t, cmd+n, or Spectacle
    if let event = event {
      if event.keyCode == KeyboardLayout.shared.keyCode(for: "w") && event.modifierFlags.contains(.command) {
        Autocomplete.hide()
      } else if event.modifierFlags.contains(.command) || event.modifierFlags.contains(.option) {
        Autocomplete.position()
      }
    }
  }
}
