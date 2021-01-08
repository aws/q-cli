//
//  KeyBindingsManager.swift
//  fig
//
//  Created by James Jackson on 1/7/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class KeyBindingsManager {
  static let keymapFilePath = URL(fileURLWithPath: "\(NSHomeDirectory())/.fig/figkeymap.txt")
  static var keyBindings: [Keystroke: TextTransformation] = parseKeyBindings()
  
  private static func parseKeyBindings() -> [Keystroke: TextTransformation] {
    print("xterm: reparsing keybindings")
    var keyBindings: [Keystroke: TextTransformation] = [:]
    do {
      let content = try NSString(contentsOf: Self.keymapFilePath, encoding: String.Encoding.utf8.rawValue) as String
      let lines = content.split(whereSeparator: \.isNewline)
      for line in lines {
        let entry = line.split(whereSeparator: \.isWhitespace)
        if let action = TextTransformation(rawValue: String(entry[0])), let keystroke = Self.parsedKeystroke(from: entry[1]) {
          if (keyBindings[keystroke] == nil) {
            keyBindings[keystroke] = action
          } else {
            print("xterm: attempting rebind an already used key, avoiding to prevent crash")
          }
        }
      }
      // add in non-configurable bindings, will add as found... :(
      keyBindings[Keystroke( keyCode: Keycode.tab)] = .expandOrComplete
      keyBindings[Keystroke( keyCode: Keycode.returnKey)] = .acceptLine
      keyBindings[Keystroke( keyCode: Keycode.delete)] = .backwardDeleteChar
      keyBindings[Keystroke(modifierFlags: [.command], keyCode: Keycode.v)] = .paste
      keyBindings[Keystroke(modifierFlags: [.control], keyCode: Keycode.c)] = .killProcess
      keyBindings[Keystroke(modifierFlags: [.control], keyCode: Keycode.two)] = .setMarkCommand // unknown
      keyBindings[Keystroke(modifierFlags: [.control], keyCode: Keycode.forwardSlash)] = .forwardSlash // some very strange iterm command
    } catch {
      // recreate defaults file
      let defaultKeybindingsContent = "backwardWord ←\nbackwardWord ⌃B\nforwardWord →\nforwardWord ⌃F\nhistorySearchBackward ↑\nhistorySearchBackward ⌃P\nhistorySearchForward ↓\nhistorySearchForward ⌃N\nbeginningOfLine ⌃A\nendOfLine ⌃E\nhistoryIncrementalSearchBackward ⌃R\nhistoryIncrementalSearchForward ⌃S\nbackwardDeleteChar ⌃H\ndeleteCharOrList ⌃D\ntransposeChars ⌃T\nkillWholeLine ⌃U\nbackwardKillWord ⌃W\nyank ⌃Y\nsendBreak ⌃G\nacceptLine ⌃J\nacceptLine ⌃M\nexpandOrComplete ⌃I"
      do {
        print("xterm: keybindings file default not found recreating")
        try defaultKeybindingsContent.write(to: Self.keymapFilePath, atomically: true, encoding: String.Encoding.utf8)
        return parseKeyBindings()
      } catch {
        print("xterm: failed to write file")
      }
    }
    return keyBindings
  }
  
  private static func parsedKeystroke(from userBinding: Substring.SubSequence) -> Keystroke? {
    var modifierFlags: NSEvent.ModifierFlags = []
    var keyCode: UInt16?
    for char in userBinding {
      // modifier
      if (char == "⌃" || char == "^") {
        modifierFlags.insert(.control)
      } else if (char == "⌥") {
        modifierFlags.insert(.option)
      } else if (char == "⌘") {
        modifierFlags.insert(.command)
      }
      // keycode
      if (char == "↑") {
        keyCode = Keycode.upArrow
      } else if (char == "↓") {
        keyCode = Keycode.downArrow
      } else if (char == "←") {
        keyCode = Keycode.leftArrow
      } else if (char == "→") {
        keyCode = Keycode.rightArrow
      } else {
        // converts to US-ANSI Keyboard Positions, case-insensitive
        keyCode = KeyboardLayout.shared.keyCode(for: String(char))
      }
    }
    if let keyCode = keyCode {
      return Keystroke(modifierFlags: modifierFlags, keyCode: keyCode)
    }
    return nil
  }
}
