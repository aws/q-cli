//
//  ZLEIntegration.swift
//  fig
//
//  Created by Matt Schrage on 2/2/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class ZLEIntegration {
  static let insertionLock = "\(NSHomeDirectory())/.fig/insertion-lock"
  static let insertionFile = "\(NSHomeDirectory())/.fig/zle/insert"
  static let deletionFile = "\(NSHomeDirectory())/.fig/zle/delete"
  static let offsetFile = "\(NSHomeDirectory())/.fig/zle/offset"
  static let immediateFile = "\(NSHomeDirectory())/.fig/zle/immediate"

  static func insertLock() {
    // The existence of the insertion-lock file prevents latency in ZLE integration when inserting text
    // See the `self-insert` function in zle.sh
    FileManager.default.createFile(atPath: insertionLock, contents: nil, attributes: nil)

  }
  
  static func insertUnlock(with insertionText: String) {
      // remove lock after keystrokes have been processes
      // requires delay proportional to number of character inserted
      // unfortunately, we don't really know how long this will take - it varies significantly between native and Electron terminals.
      // We can probably be smarter about this and modulate delay based on terminal.
      let delay = min(0.01 * Double(insertionText.count), 0.5)
      Timer.delayWithSeconds(delay) {
          try? FileManager.default.removeItem(atPath: insertionLock)
          // If ZLE, manually update keybuffer
          if let window = AXWindowServer.shared.whitelistedWindow,
              KeypressProvider.shared.keyBuffer(for: window).backing == .zle,
             let context = KeypressProvider.shared.keyBuffer(for: window).insert(text: insertionText) {
              // trigger an update!
              print("update: \(context.0)")
              Autocomplete.update(with: context, for: window.hash)
            
          }
      }
    
      // Update position of window if backed by ZLE
      if let window = AXWindowServer.shared.whitelistedWindow,
            KeypressProvider.shared.keyBuffer(for: window).backing == .zle {
        
        Autocomplete.position(makeVisibleImmediately: false, completion: nil)
      }

  }
  
  static func paste() {
    if let window = AXWindowServer.shared.whitelistedWindow,
        let pastedText = NSPasteboard.general.string(forType: .string),
        let context = KeypressProvider.shared.keyBuffer(for: window).insert(text: pastedText) {
         print("ZLE: paste! (Hiding popup window)")
         Autocomplete.update(with: context, for: window.hash)
     }
  }
  
  static func insert(with insertionText: String, version: Int?) {
    let backspaceLiteral = Character("\u{8}")
    let cursorLeftLiteral = String("\u{1b}[D")
    let cursorLeftPlaceholder =  Character("\u{0}")
    
    var cleaned = insertionText
    
    // handle
    if insertionText.starts(with: " \u{8}") {
      cleaned = String(insertionText.dropFirst(2))
    }
    
    let isImmediate = insertionText.hasSuffix("\n")
    
    if isImmediate {
      cleaned = String(cleaned.dropLast())
    }

    
    // "\b\b\bText to insert^[[D^[[D"
    print("ZLE: inserting \(cleaned)")
    let (numberOfCharactersToRemove, _) = cleaned.reduce((0, true)) { (acc, char) -> (Int, Bool) in
      let (length, stillSearching) = acc
      
      guard stillSearching else {
        return acc
      }
      
      if char == backspaceLiteral {
        return (length + 1, true)
      } else {
        return (length, false)
      }
    }
    
    cleaned = String(cleaned.dropFirst(numberOfCharactersToRemove))
    
    let replacingMultiCharacterSequence = cleaned.replacingOccurrences(of: cursorLeftLiteral, with: String(cursorLeftPlaceholder))
    
    let (cursorOffset, _) = replacingMultiCharacterSequence.reversed().reduce((0, true)) { (acc, char) -> (Int, Bool) in
      let (length, stillSearching) = acc
      
      guard stillSearching else {
        return acc
      }
      
      if char == cursorLeftPlaceholder {
        return (length + 1, true)
      } else {
        return (length, false)
      }
    }
    
    cleaned = String(cleaned.dropLast(cursorOffset * cursorLeftLiteral.count))
    
    print("ZLE: delete \(numberOfCharactersToRemove)")
    print("ZLE: insert cleaned string '\(cleaned)'")
    print("ZLE: offset -\(cursorOffset)")
    
    FileManager.default.createFile(atPath: insertionFile, contents: cleaned.data(using: .utf8), attributes: nil)
    FileManager.default.createFile(atPath: deletionFile, contents: String(numberOfCharactersToRemove).data(using: .utf8), attributes: nil)
    FileManager.default.createFile(atPath: offsetFile, contents: String(cursorOffset).data(using: .utf8), attributes: nil)
    FileManager.default.createFile(atPath: immediateFile, contents: String(isImmediate ? 1 : 0).data(using: .utf8), attributes: nil)
  
      // Hide autocomplete to avoid jank
//      Autocomplete.hide()

    // Make sure this key is bound to a widget in fig.sh.
    // Use `read -r` to determine appropriate keycode.
      
      switch version {
        case nil:
          ShellBridge.injectUnicodeString(insertionText, delay: 0.01, completion: nil)
        default: // > 1
          ShellBridge.injectUnicodeString("◧")
      }
    
    
      // Delay helps avoid jank (caused by positioning window on old cursor location)
      Timer.delayWithSeconds(0.175) {
        
        if let window = AXWindowServer.shared.whitelistedWindow,
            KeypressProvider.shared.keyBuffer(for: window).backing == .zle {

            Autocomplete.position(makeVisibleImmediately: false)
         
//            WindowManager.shared.positionAutocompletePopover(textRect: KeypressProvider.shared.getTextRect(), makeVisibleImmediately: false, completion: nil)

        }
      }
    
  }

  
}
