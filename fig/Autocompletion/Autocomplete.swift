//
//  Autocomplete.swift
//  fig
//
//  Created by Matt Schrage on 2/2/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class Autocomplete {
  static let throttler = Throttler(minimumDelay: 0.05)
  static func update(with context: (String, Int)?, for windowHash: ExternalWindowHash) {
    let tty = ShellHookManager.shared.tty(for: windowHash)
    let ttyDescriptor = tty?.descriptor == nil ? "null" : "'\(tty!.descriptor)'"
    let cmd = tty?.cmd == nil ? "null" : "'\(tty!.cmd!)'"
    let cwd = tty?.cwd == nil ? "null" : "`\(tty!.cwd!.trimmingCharacters(in: .whitespacesAndNewlines))`"
    let prefix = tty?.runUsingPrefix == nil ? "null" : "`\(tty!.runUsingPrefix!)`"
    if let (buffer, index) = context, let b64 = buffer.data(using: .utf8)?.base64EncodedString() {
      // We aren't setting the tetheredWindow!
      print("fig.autocomplete = \(buffer)")
      print("fig.autocomplete(b64DecodeUnicode(`\(b64)`), \(index), '\(windowHash)', \(ttyDescriptor), \(cwd), \(cmd), \(prefix))")
      WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.autocomplete(b64DecodeUnicode(`\(b64)`), \(index), '\(windowHash)', \(ttyDescriptor), \(cwd), \(cmd), \(prefix)) } catch(e){} ", completionHandler: nil)
    } else {
      WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ fig.nocontext('\(windowHash)') } catch(e){} ", completionHandler: nil)
    }
    
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
        if let rect = KeypressProvider.shared.getTextRect(), !keybuffer.writeOnly {//, keybuffer.buffer?.count != 0 {
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
    }
  
    if (Defaults.insertUsingRightArrow) {
        KeypressProvider.shared.addRedirect(for: Keycode.rightArrow, in: window)
    }
  }
  
  static func removeAllRedirects(from window: ExternalWindow) {
    KeypressProvider.shared.resetRedirects(for: window)
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
