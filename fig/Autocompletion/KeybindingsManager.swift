//
//  KeybindingsManager.swift
//  fig
//
//  Created by James Jackson on 1/7/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class KeybindingsManager {
  static let shared = KeybindingsManager()
  var keyBindings: [Keystroke: TextTransformation]
  
  private init() {
    self.keyBindings = [:]
    let bindingsFile = URL(fileURLWithPath: "\(NSHomeDirectory())/.fig/figkeymap.txt")
    do {
      let content = try NSString(contentsOf: bindingsFile, encoding: String.Encoding.utf8.rawValue) as String
      let lines = content.split(whereSeparator: \.isNewline)
      for line in lines {
        let entry = line.split(maxSplits: 1, whereSeparator: \.isWhitespace)
        let action = TextTransformation(rawValue: String(entry[0]))
        let userBindings = entry[1].split(separator: ",")
        for userBinding in userBindings {
          if let action = action, let keystroke = Self.parsedKeystroke(from: userBinding) {
            if (keyBindings[keystroke] == nil) {
              keyBindings[keystroke] = action
            } else {
              print("attempting rebind an already used key, avoiding to prevent crash")
            }
          }
        }
      }
    } catch {
      print("error parsing bindings")
    }
  }
  
  private static func parsedKeystroke(from userBinding: Substring.SubSequence) -> Keystroke? {
    var modifierFlags: NSEvent.ModifierFlags = []
    var keyCode: UInt16?
    for char in userBinding {
      // modifier
      if (char == "^") {
        modifierFlags.insert(.control)
      } else if (char == "⌥") {
        modifierFlags.insert(.option)
      } else if (char == "⌘") {
        modifierFlags.insert(.command)
      }
      // keycode
      if (char == " ") {
        continue
      } else if (char == "⌫") {
        keyCode = Keycode.delete
      } else if (char == "↹") {
        keyCode = Keycode.tab
      } else if (char == "↑") {
        keyCode = Keycode.upArrow
      } else if (char == "↓") {
        keyCode = Keycode.downArrow
      } else if (char == "←") {
        keyCode = Keycode.leftArrow
      } else if (char == "→") {
        keyCode = Keycode.rightArrow
      } else if (char == "↹") {
        keyCode = Keycode.tab
      } else if (char == "↩︎") {
        keyCode = Keycode.returnKey
      } else {
        keyCode = KeyboardLayout.shared.keyCode(for: String(char))
      }
    }
    if let keyCode = keyCode {
      return Keystroke(modifierFlags: modifierFlags, keyCode: keyCode)
    }
    return nil
  }
  
  // reset to defaults
  func resetToDefaults() {
    self.keyBindings = [
      // backward-word
      Keystroke(keyCode: Keycode.leftArrow): .backwardWord,
      Keystroke(modifierFlags: [.control], keyCode: Keycode.b) : .backwardWord,
      // forward-word
      Keystroke(modifierFlags: [.control], keyCode: Keycode.f) : .forwardWord,
      Keystroke(keyCode: Keycode.rightArrow) : .forwardWord,
      // history-search-backward
      Keystroke(modifierFlags: [.control], keyCode: Keycode.p) : .historySearchBackward,
      Keystroke(keyCode: Keycode.upArrow) : .historySearchBackward,
      // history-search-forward
      Keystroke(modifierFlags: [.control], keyCode: Keycode.n) : .historySearchForward,
      Keystroke(keyCode: Keycode.downArrow) : .historySearchForward,
      // beginning-of-line
      Keystroke(modifierFlags: [.control], keyCode: Keycode.a) : .beginningOfLine,
      // end-of-line
      Keystroke(modifierFlags: [.control], keyCode: Keycode.e) : .endOfLine,
      // history-incremental-search-backward
      Keystroke(modifierFlags: [.control], keyCode: Keycode.r) : .historyIncrementalSearchBackward,
      // history-incremental-search-forward
      Keystroke(modifierFlags: [.control], keyCode: Keycode.s) : .historyIncrementalSearchForward,
      // backward-delete-char
      Keystroke(modifierFlags: [.control], keyCode: Keycode.h) : .backwardDeleteChar,
      Keystroke(keyCode: Keycode.delete) : .backwardDeleteChar,
      // delete-char-or-list
      Keystroke(modifierFlags: [.control], keyCode: Keycode.d) : .deleteCharOrList,
      // transpose-chars
      Keystroke(modifierFlags: [.control], keyCode: Keycode.t) : .transposeChars,
      // kill-whole-line
      Keystroke(modifierFlags: [.control], keyCode: Keycode.u) : .killWholeLine,
      // backward-kill-word
      Keystroke(modifierFlags: [.control], keyCode: Keycode.w) : .backwardKillWord,
      // yank
      Keystroke(modifierFlags: [.control], keyCode: Keycode.y) : .yank,
      // send-break
      Keystroke(modifierFlags: [.control], keyCode: Keycode.g) : .sendBreak,
      // accept-line
      Keystroke(keyCode: Keycode.returnKey) : .acceptLine,
      Keystroke(modifierFlags: [.control], keyCode: Keycode.j) : .acceptLine,
      Keystroke(modifierFlags: [.control], keyCode: Keycode.m) : .acceptLine,
      // expand-or-complete
      Keystroke(keyCode: Keycode.tab) : .expandOrComplete,
      Keystroke(modifierFlags: [.control], keyCode: Keycode.i) : .expandOrComplete,
      // need more research, not present in bindkey
      Keystroke(modifierFlags: [.command], keyCode: Keycode.v) : .paste,
      Keystroke(modifierFlags: [.control], keyCode: Keycode.forwardSlash) : .forwardSlash,
      Keystroke(modifierFlags: [.control], keyCode: Keycode.c) : .killProcess,
      Keystroke(modifierFlags: [.control], keyCode: Keycode.two) : .ctrlTwo,
    ]
    
    let defaults = "backwardWord ←, ^B\nforwardWord →, ^F\nhistorySearchBackward ↑, ^P\nhistorySearchForward ↓, ^N\nbeginningOfLine ^A\nendOfLine ^E\nhistoryIncrementalSearchBackward ^R\nhistoryIncrementalSearchForward ^S\nbackwardDeleteChar ^H, ⌫\ndeleteCharOrList ^D\ntransposeChars ^T\nkillWholeLine ^U\nbackwardKillWord ^W\nyank ^Y\nsendBreak ^G\nacceptLine ↩︎, ^J, ^M\nexpandOrComplete ↹, ^I\npaste ⌘V\nforwardSlash ^/\nkillProcess ^C\nctrlTwo ^2"
    let bindingsFile = URL(fileURLWithPath: "\(NSHomeDirectory())/.fig/figkeymap.txt")
    do {
        try defaults.write(to: bindingsFile, atomically: true, encoding: String.Encoding.utf8)
    } catch {
        // failed to write file – bad permissions, bad filename, missing permissions, or more likely it can't be converted to the encoding
    }
  }
}
