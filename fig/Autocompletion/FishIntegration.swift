//
//  FishIntegration.swift
//  fig
//
//  Created by Matt Schrage on 4/5/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class FishIntegration: ShellIntegration {
  static let throttler = Throttler(minimumDelay: 0.005, queue: DispatchQueue(label: "com.withfig.fish"))
  static func enabledFor(_ tty: TTY) -> Bool {
    
    guard tty.name == "fish" else {
      return false
    }
    
    guard let version = tty.shellIntegrationVersion, version >= 2 else {
      return false
    }

    return true
  }
  
  // function fig_keybuffer --on-signal SIGUSR1
  //    fig bg:zsh-keybuffer (commandline -C) (commandline) 0 &
  // end
  static func handleKeystroke(event: NSEvent?, in window: ExternalWindow) -> Bool {
    guard let event = event else {
      return false
    }
    
    guard let tty = window.tty else {
      return false
    }
    
    guard enabledFor(tty) else {
      return false
    }
    
    guard let pid = tty.pid else {
      return false
    }
    
    let shouldReposition = ![ Keycode.enter, Keycode.upArrow, Keycode.downArrow ].contains(event.keyCode) && !(event.modifierFlags.contains(.command) || event.modifierFlags.contains(.control))
    
    // Only send signal on key up
    // because we don't want to run updates twice per keystroke
    // Don't send signal on enter key (avoids killing new process when execing and multiple phantom keypresses when inserting)
    if event.type == .keyUp, event.keyCode != Keycode.returnKey, !(event.keyCode == KeyboardLayout.shared.keyCode(for: "R") &&  event.modifierFlags.contains(.control))  {
      print("fish: Send signal SIGUSR1 to \(pid) on '\(event.characters ?? "?")' (\(event.keyCode))")
      requestUpdate(from: pid)
    } else if shouldReposition {
      // Reposition on keyDown to make motion less jerky
      // But not when modifier keys are pressed
      // or enter or up / down arrow keys
      Autocomplete.position(makeVisibleImmediately: false)
    }
    
    return true
  }
  
  static func requestUpdate(from pid: pid_t) {
    throttler.throttle {
      guard !inserting else { return }
      Darwin.kill(pid, SIGUSR1)
    }
  }
  
  static var inserting = false
  static func insertLock() {
    inserting = true
  }
  
  static func insertUnlock(with insertionText: String) {
    
    // Delay helps avoid jank (caused by positioning window on old cursor location)
    Timer.delayWithSeconds(0.1) {
      inserting = false
      
      if let window = AXWindowServer.shared.whitelistedWindow,
          KeypressProvider.shared.keyBuffer(for: window).backing == .fish,
         let context = KeypressProvider.shared.keyBuffer(for: window).insert(text: insertionText) {

          Autocomplete.update(with: context, for: window.hash)

      }
    }
  }
}
