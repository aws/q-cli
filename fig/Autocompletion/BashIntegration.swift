//
//  BashIntegration.swift
//  fig
//
//  Created by Matt Schrage on 6/7/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class BashIntegration {

  static func enabledFor(_ tty: TTY) -> Bool {
    
    guard tty.name == "bash" else {
      return false
    }
    
    guard let version = tty.shellIntegrationVersion, version >= 2 else {
      return false
    }

    return true
  }

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
    
    let shouldReposition = ![ Keycode.enter, Keycode.upArrow, Keycode.downArrow ].contains(event.keyCode) && !(event.modifierFlags.contains(.command) || event.modifierFlags.contains(.control))
    
    // Only send signal on key up
    // because we don't want to run updates twice per keystroke
    // Don't send signal on enter key (avoids killing new process when execing and multiple phantom keypresses when inserting)
    if event.type == .keyUp, event.keyCode != Keycode.returnKey, !(event.keyCode == KeyboardLayout.shared.keyCode(for: "R") &&  event.modifierFlags.contains(.control))  {
    } else if shouldReposition {
      // Reposition on keyDown to make motion less jerky
      // But not when modifier keys are pressed
      // or enter or up / down arrow keys
      Autocomplete.position(makeVisibleImmediately: false)
    }
    
    return true
  }
}
