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
    WindowManager.shared.positionAutocompletePopover(textRect: nil)
  }
  
  static func position(makeVisibleImmediately: Bool = true, completion:(() -> Void)? = nil) {
    guard let window = AXWindowServer.shared.whitelistedWindow else {
      completion?()
      return
    }
    
    throttler.throttle {
      DispatchQueue.main.async {
        let keybuffer = KeypressProvider.shared.keyBuffer(for: window)
        if let rect = window.cursor, !keybuffer.writeOnly {//, keybuffer.buffer?.count != 0 {
          WindowManager.shared.positionAutocompletePopover(textRect: rect, makeVisibleImmediately: makeVisibleImmediately, completion: completion)
        } else {
          completion?()
        }
      }
    }
  }

  static func handleShowOnTab(event:CGEvent, in window: ExternalWindow) -> EventTapAction {
    let keycode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
    guard Keycode.tab == keycode else {
      return .ignore
    }
    
    guard event.type == .keyDown else {
      return .ignore
    }
    
    // no modifier keys are pressed!
    guard !event.flags.containsKeyboardModifier else {
        return .ignore
    }
        
    let autocompleteIsNotVisible = WindowManager.shared.autocomplete?.isHidden ?? true

    let onlyShowOnTab = (Settings.shared.getValue(forKey: Settings.onlyShowOnTabKey) as? Bool) ?? false
    
    // if not enabled or if autocomplete is already visible, handle normally
    if !onlyShowOnTab || !autocompleteIsNotVisible {
      return .ignore
    }
    
    // Don't intercept tab when in VSCode editor
    guard window.isFocusedTerminal else {
      return .forward
    }
    
    // toggle autocomplete on and consume tab keypress
    Autocomplete.toggle(for: window)
    return .consume
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
        
        if let window = AXWindowServer.shared.whitelistedWindow,
           KeypressProvider.shared.keyBuffer(for: window).backing != nil,
           let context = KeypressProvider.shared.keyBuffer(for: window).insert(text: insertionText) {
            Autocomplete.update(with: context, for: window.hash)
        }
    }
  }
}

extension String {
  
    func isAlphanumeric() -> Bool {
        return self.rangeOfCharacter(from: CharacterSet.alphanumerics.inverted) == nil && self != ""
    }
}
