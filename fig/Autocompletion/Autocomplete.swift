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
  
  static func hide() {
    guard let window = AXWindowServer.shared.whitelistedWindow else { return }
    
    WindowManager.shared.positionAutocompletePopover(textRect: nil)
    
    KeypressProvider.shared.removeRedirect(for: Keycode.upArrow, in: window)
    KeypressProvider.shared.removeRedirect(for: Keycode.downArrow, in: window)
    KeypressProvider.shared.removeRedirect(for: Keycode.tab, in: window)
    KeypressProvider.shared.removeRedirect(for: Keycode.returnKey, in: window)
    KeypressProvider.shared.removeRedirect(for: Keystroke(modifierFlags: [.control], keyCode: Keycode.n), in: window)
    KeypressProvider.shared.removeRedirect(for: Keystroke(modifierFlags: [.control], keyCode: Keycode.p), in: window)
  }
  
  static func position(makeVisibleImmediately: Bool = true, completion:(() -> Void)? = nil) {
    guard let window = AXWindowServer.shared.whitelistedWindow else {
      completion?()
      return
    }
    
    throttler.throttle {
      let keybuffer = KeypressProvider.shared.keyBuffer(for: window)
      if let rect = KeypressProvider.shared.getTextRect(), !keybuffer.writeOnly {//, keybuffer.buffer?.count != 0 {
        WindowManager.shared.positionAutocompletePopover(textRect: rect, makeVisibleImmediately: makeVisibleImmediately, completion: completion)
      } else {
        KeypressProvider.shared.removeRedirect(for: Keycode.upArrow, in: window)
        KeypressProvider.shared.removeRedirect(for: Keycode.downArrow, in: window)
        KeypressProvider.shared.removeRedirect(for: Keycode.tab, in: window)
        KeypressProvider.shared.removeRedirect(for: Keycode.returnKey, in: window)
        KeypressProvider.shared.removeRedirect(for: Keystroke(modifierFlags: [.control], keyCode: Keycode.n), in: window)
        KeypressProvider.shared.removeRedirect(for: Keystroke(modifierFlags: [.control], keyCode: Keycode.p), in: window)
        
        DispatchQueue.main.async {
          completion?()
        }
      }
    }
  }
}
