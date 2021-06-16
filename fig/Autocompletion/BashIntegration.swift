//
//  BashIntegration.swift
//  fig
//
//  Created by Matt Schrage on 6/7/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class BashIntegration: ShellIntegration {
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
        // If Bash, manually update keybuffer
        if let window = AXWindowServer.shared.whitelistedWindow,
            KeypressProvider.shared.keyBuffer(for: window).backing == .bash,
           let context = KeypressProvider.shared.keyBuffer(for: window).insert(text: insertionText) {
            // trigger an update!
            print("update: \(context.0)")
            Autocomplete.update(with: context, for: window.hash)
          
        }
    }
  
    // Update position of window if backed by Bash
    if let window = AXWindowServer.shared.whitelistedWindow,
          KeypressProvider.shared.keyBuffer(for: window).backing == .bash {
      
      Autocomplete.position(makeVisibleImmediately: false, completion: nil)
    }
    
  }

  static func enabledFor(_ tty: TTY) -> Bool {
    
    guard tty.name == "bash" else {
      return false
    }
    
    guard let version = tty.shellIntegrationVersion, version >= 3 else {
      return false
    }

    return true
  }
}
