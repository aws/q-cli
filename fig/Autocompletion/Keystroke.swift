//
//  Keystroke.swift
//  fig
//
//  Created by James Jackson on 12/22/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

struct Keystroke: Hashable {
  var modifierFlags: NSEvent.ModifierFlags = []
  var keyCode: UInt16
  
  
  func hash(into hasher: inout Hasher) {
    hasher.combine(keyCode)
    hasher.combine(modifierFlags)
  }
  
  static func == (lhs: Keystroke, rhs: Keystroke) -> Bool {
    return lhs.keyCode == rhs.keyCode && lhs.modifierFlags == rhs.modifierFlags
  }
  
  static func from(event: CGEvent) -> Keystroke {
    let keyCode = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
    var modifierFlags: NSEvent.ModifierFlags = []
    
    if (event.flags.contains(.maskCommand)) {
      modifierFlags.insert(.command)

    }
    
    if (event.flags.contains(.maskControl)) {
      modifierFlags.insert(.control)
    }
    
    if (event.flags.contains(.maskShift)) {
        modifierFlags.insert(.shift)
    }
    
    if (event.flags.contains(.maskAlternate)) {
        modifierFlags.insert(.option)
    }
    
    return Keystroke(modifierFlags: modifierFlags, keyCode: keyCode)
  }
  
}

extension NSEvent.ModifierFlags: Hashable {
  public func hash(into hasher: inout Hasher) {
    hasher.combine(self.rawValue)
  }
}
