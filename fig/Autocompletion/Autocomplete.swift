//
//  Autocomplete.swift
//  fig
//
//  Created by Matt Schrage on 2/2/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Autocomplete {
  static func log(_ buffer: String, _ cursor: Int) {
    var logging = buffer
    let index = logging.index(logging.startIndex, offsetBy: cursor, limitedBy: buffer.endIndex) ?? buffer.endIndex
    logging.insert("|", at: index)
    Logger.log(message: logging, subsystem: .autocomplete)
  }
  
  static let throttler = Throttler(minimumDelay: 0.05)
  static func update(with context: (String, Int)?, for windowHash: ExternalWindowHash) {
    let tty = ShellHookManager.shared.tty(for: windowHash)
    let ttyDescriptor = tty?.descriptor == nil ? "null" : "'\(tty!.descriptor)'"
    let cmd = tty?.cmd == nil ? "null" : "'\(tty!.cmd!)'"
    let cwd = tty?.cwd == nil ? "null" : "`\(tty!.cwd!.trimmingCharacters(in: .whitespacesAndNewlines))`"
    let prefix = tty?.runUsingPrefix == nil ? "null" : "`\(tty!.runUsingPrefix!)`"
    if let (buffer, index) = context, let b64 = buffer.data(using: .utf8)?.base64EncodedString() {
      // We aren't setting the tetheredWindow!
      
      Autocomplete.log(buffer, index)
      
      print("fig.autocomplete = \(buffer)")
      print("fig.autocomplete(b64DecodeUnicode(`\(b64)`), \(index), '\(windowHash)', \(ttyDescriptor), \(cwd), \(cmd), \(prefix))")
      WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.autocomplete(b64DecodeUnicode(`\(b64)`), \(index), '\(windowHash)', \(ttyDescriptor), \(cwd), \(cmd), \(prefix)) } catch(e){} ", completionHandler: nil)
    } else {
      WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.nocontext('\(windowHash)') } catch(e){} ", completionHandler: nil)
    }
    
  }
  
  static func runJavascript(_ command: String) {
    WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ \(command) } catch(e) { console.log(e) }", completionHandler: nil)
  }
  static func redirect(keyCode: UInt16, event: CGEvent, for windowHash: ExternalWindowHash) {
    
    guard let event =  NSEvent(cgEvent: event) else { return }
    let characters = event.characters ?? ""
      
    WindowManager.shared.autocomplete?.webView?.evaluateJavaScript(
      """
      try{
        fig.keypress(\"\(keyCode)\", \"\(windowHash)\", {
            command: \(event.modifierFlags.contains(.command)),
            control: \(event.modifierFlags.contains(.control)),
              shift: \(event.modifierFlags.contains(.shift)),
           isRepeat: \(event.isARepeat),
         characters: \"\(characters.isAlphanumeric() ? characters : "")\" })
      } catch(e) {}
      """, completionHandler: nil)
  }
  
  static func toggle(for window: ExternalWindow) {
    let buffer = KeypressProvider.shared.keyBuffer(for: window)

    buffer.writeOnly = !buffer.writeOnly

    if buffer.writeOnly {
        Autocomplete.hide()
        WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.keypress(\"\(Keycode.escape)\", \"\(window.hash)\") } catch(e) {}", completionHandler: nil)
    } else {
        Autocomplete.update(with: buffer.currentState, for: window.hash)
        Autocomplete.position()
    }
  }
  
  static func hide() {
    guard let window = AXWindowServer.shared.whitelistedWindow else { return }
    
    WindowManager.shared.positionAutocompletePopover(textRect: nil)
    
    Autocomplete.removeAllRedirects(from: window)

  }
  
  static func position(makeVisibleImmediately: Bool = true, completion:(() -> Void)? = nil) {
    guard let window = AXWindowServer.shared.whitelistedWindow else {
      completion?()
      return
    }
    
    throttler.throttle {
      DispatchQueue.main.async {
        let keybuffer = KeypressProvider.shared.keyBuffer(for: window)
        if let rect = Accessibility.getTextRect(), !keybuffer.writeOnly {//, keybuffer.buffer?.count != 0 {
          WindowManager.shared.positionAutocompletePopover(textRect: rect, makeVisibleImmediately: makeVisibleImmediately, completion: completion)
        } else {
          Autocomplete.removeAllRedirects(from: window)
          completion?()
        }
      }
    }
  }
  
  static func interceptKeystrokes(in window: ExternalWindow) {
    let nKeycode = KeyboardLayout.shared.keyCode(for: "N") ?? Keycode.n
    let pKeycode = KeyboardLayout.shared.keyCode(for: "P") ?? Keycode.p
    let jKeycode = KeyboardLayout.shared.keyCode(for: "J") ?? Keycode.j
    let kKeycode = KeyboardLayout.shared.keyCode(for: "K") ?? Keycode.k
    let iKeycode = KeyboardLayout.shared.keyCode(for: "I") ?? Keycode.i
    
    KeypressProvider.shared.addRedirect(for: Keystroke(modifierFlags: [.command], keyCode: iKeycode), in: window)
    KeypressProvider.shared.addRedirect(for: Keycode.upArrow, in: window)
    KeypressProvider.shared.addRedirect(for: Keycode.downArrow, in: window)
    KeypressProvider.shared.addRedirect(for: Keycode.tab, in: window)
    KeypressProvider.shared.addRedirect(for:  Keystroke(modifierFlags: [.shift], keyCode: Keycode.tab), in: window)
    if (!Defaults.onlyInsertOnTab) {
        KeypressProvider.shared.addRedirect(for: Keycode.returnKey, in: window)
    }
    
    if (Settings.shared.getValue(forKey: Settings.allowAlternateNavigationKeys) as? Bool ?? true) {
        KeypressProvider.shared.addRedirect(for: Keystroke(modifierFlags: [.control], keyCode: nKeycode), in: window)
        KeypressProvider.shared.addRedirect(for: Keystroke(modifierFlags: [.control], keyCode: pKeycode), in: window)
        
        KeypressProvider.shared.addRedirect(for: Keystroke(modifierFlags: [.control], keyCode: jKeycode), in: window)
        KeypressProvider.shared.addRedirect(for: Keystroke(modifierFlags: [.control], keyCode: kKeycode), in: window)
    }
  
    if (Settings.shared.getValue(forKey: Settings.disablePopoutDescriptions) as? Bool ?? false) {
        KeypressProvider.shared.addRedirect(for: Keystroke(modifierFlags: [.command], keyCode: iKeycode), in: window)
    }
    
    if (Defaults.insertUsingRightArrow) {
        KeypressProvider.shared.addRedirect(for: Keycode.rightArrow, in: window)
    }
  }
  
  static func removeAllRedirects(from window: ExternalWindow) {
    KeypressProvider.shared.resetRedirects(for: window)
  }
  
  static func handleCommandIKey(event:CGEvent, in window: ExternalWindow) -> EventTapAction {
    let keycode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
    guard KeyboardLayout.shared.keyCode(for: "I") ?? Keycode.i == keycode else {
      return .ignore
    }
    
    guard let event = NSEvent(cgEvent: event), event.modifierFlags.contains(.command) else {
      return .ignore
    }
    
    // Don't intercept command+I when in VSCode editor
    if Integrations.electronTerminals.contains(window.bundleId ?? "") &&
        Accessibility.findXTermCursorInElectronWindow(window) == nil {
      return .forward
    }
        
    let autocompleteIsNotVisible = WindowManager.shared.autocomplete?.isHidden ?? true

    // Allow to be intercepted by autocomplete app if visible
    // otherwise prevent keypress from propogating
    return autocompleteIsNotVisible ? .consume : .ignore
  }
  
  static func handleTabKey(event:CGEvent, in window: ExternalWindow) -> EventTapAction {
    let keycode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
    guard Keycode.tab == keycode else {
      return .ignore
    }
    
    guard [.keyDown].contains(event.type) else {
      return .ignore
    }
    
    let autocompleteIsNotVisible = WindowManager.shared.autocomplete?.isHidden ?? true

    let onlyShowOnTab = (Settings.shared.getValue(forKey: Settings.onlyShowOnTabKey) as? Bool) ?? false
    
    // if not enabled or if autocomplete is already visible, handle normally
    if !onlyShowOnTab || !autocompleteIsNotVisible {
      return .ignore
    }
    
    // toggle autocomplete on and consume tab keypress
    Autocomplete.toggle(for: window)
    return .consume
    
  }
  
  static func handleEscapeKey(event:CGEvent, in window: ExternalWindow) -> EventTapAction {
    let keycode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
    guard Keycode.escape == keycode else {
      return .ignore
    }
    
    guard [.keyDown].contains(event.type) else {
      return .ignore
    }
    
    let autocompleteIsNotVisible = WindowManager.shared.autocomplete?.isHidden ?? true
        
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
    
    return autocompleteIsNotVisible ? .forward : .consume

  }
}

protocol ShellIntegration {
  static func insertLock()
  static func insertUnlock(with insertionText: String)
}

class GenericShellIntegration: ShellIntegration {
  static let insertionLock = "\(NSHomeDirectory())/.fig/insertion-lock"

  static func insertLock() {
    FileManager.default.createFile(atPath: insertionLock, contents: nil, attributes: nil)
  }
  
  static func insertUnlock(with insertionText: String) {
    // remove lock after keystrokes have been processes
    // requires delay proportional to number of character inserted
    // unfortunately, we don't really know how long this will take - it varies significantly between native and Electron terminals.
    // We can probably be smarter about this and modulate delay based on terminal.
    let delay = min(0.01 * Double(insertionText.count), 0.15)
    Timer.delayWithSeconds(delay) {
        try? FileManager.default.removeItem(atPath: insertionLock)
      
    }
    
  }
}

extension String {
  
    func isAlphanumeric() -> Bool {
        return self.rangeOfCharacter(from: CharacterSet.alphanumerics.inverted) == nil && self != ""
    }
}
