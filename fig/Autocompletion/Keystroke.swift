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
    
    return Keystroke(modifierFlags: modifierFlags, keyCode: keyCode)
  }
  
}

extension NSEvent.ModifierFlags: Hashable {
  public func hash(into hasher: inout Hasher) {
    hasher.combine(self.carbonFlags)
  }
}

enum TextTransformation: String {
  case backwardWord
  case forwardWord
  case historySearchBackward
  case historySearchForward
  case setMarkCommand
  case beginningOfLine
  case backwardChar
  case deleteCharOrList
  case endOfLine
  case forwardChar
  case sendBreak
  case backwardDeleteChar
  case expandOrComplete
  case acceptLine
  case killLine
  case clearScreen
  case downLineOrHistory
  case acceptLineAndDownHistory
  case upLineOrHistory
  case pushLine
  case historyIncrementalSearchBackward
  case historyIncrementalSearchForward
  case transposeChars
  case killWholeLine
  case quotedInsert
  case backwardKillWord
  case viMatchBracket
  case viFindNextchar
  case viJoin
  case killBuffer
  case inferNextHistory
  case overwriteMode
  case undo
  case viCmdMode
  case exchangePointAndMark
  case expandWord
  case whatCursorPosition
  case listExpand
  case yank
  case listChoices
  case selfInsertUnmeta
  case copyPrevWord
  case expandHistory
  case quoteRegion
  case spellWord
  case quoteLine
  case negArgument
  case insertLastWord
  case digitArgument
  case beginningOfBufferOrHistory
  case endFfBufferOrHistory
  case whichCommand
  case acceptAndHold
  case capitalizeWord
  case killWord
  case getLine
  case runHelp
  case downCaseWord
  case upLineOrSearch
  case downLineOrSearch
  case transposeWords
  case upCaseWord
  case copyRegionAsKill
  case bracketedPaste
  case deleteChar
  case executeNamedCmd
  case yankPop
  case executeLastNamedCmd
  case viGotoColumn
  case selfInsert
  
  // need more research, not present in bindkey
  case paste
  case forwardSlash
  case ctrlTwo
  case killProcess
}
