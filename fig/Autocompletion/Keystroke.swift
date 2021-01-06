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
}

extension NSEvent.ModifierFlags: Hashable {
  public func hash(into hasher: inout Hasher) {
    hasher.combine(self.carbonFlags)
  }
}

var keyBindings: [Keystroke: TextTransformation] = [
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

enum TextTransformation {
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
