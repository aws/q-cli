//
//  Autocomplete.swift
//  fig
//
//  Created by Matt Schrage on 8/31/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class KeystrokeBuffer : NSObject {
  var cursor: Int = 0
  var historyIndex = 0
  var index: String.Index?
  var stashedBuffer: String?
  var stashedIndex: String.Index?
  static let shared = KeystrokeBuffer()
  static let lineResetInKeyStrokeBufferNotification: NSNotification.Name = .init("lineResetInKeyStrokeBufferNotification")
  static let lineAcceptedInKeystrokeBufferNotification: NSNotification.Name = .init("lineAcceptedInXTermBufferNotification")
  static let firstCharacterInKeystrokeBufferNotification: NSNotification.Name = .init("firstCharacterInKeystrokeBufferNotification")
  
  override init() {
    buffer = ""
    index = buffer!.startIndex
  }
  
  var hazy: Bool = true {
    didSet {
      cursor = 0
      index = nil
    }
  }
  
  var buffer: String? = nil {
    didSet {
      if buffer == nil {
        if (Defaults.playSoundWhenContextIsLost) {
          NSSound.beep()
        }
        index = nil
      } else if (buffer == "") {
        NotificationCenter.default.post(name: Self.lineResetInKeyStrokeBufferNotification, object: nil)
        index = buffer!.startIndex
        dropStash()
      } else if (buffer?.count == 1) {
        NotificationCenter.default.post(name: Self.firstCharacterInKeystrokeBufferNotification, object: nil)
      }
    }
  }
  
  func stash() {
    print("xterm: stash")
    stashedBuffer = buffer
    stashedIndex = index
    buffer = nil
  }
  
  func restore() {
    print("xterm: restore")
    buffer = stashedBuffer
    index = stashedIndex
    historyIndex = 0
  }
  
  func dropStash() {
    print("xterm: dropStash")
    stashedBuffer = nil
    stashedIndex = nil
    historyIndex = 0
  }
  
  func handleKeystroke(event: NSEvent) -> (String, Int)? {
    let cleanedFlags = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
    let keystroke = Keystroke(modifierFlags: cleanedFlags, keyCode: event.keyCode)
    switch keyBindings[keystroke] {
    case .paste:
      guard let pasteboard = NSPasteboard.general.string(forType: .string), buffer != nil, index != nil else { break }
      buffer!.insert(contentsOf: pasteboard, at: index!)
      index = buffer!.index(index!, offsetBy: pasteboard.count, limitedBy: buffer!.endIndex)
      print("xterm: Paste! (\(pasteboard))")
    case .backwardWord:
      guard buffer != nil, index != nil, index != buffer!.startIndex else { break }
      index = buffer!.index(before: index!)
      print("xterm: move cursor to the left by 1")
    case .forwardWord:
      // handles zsh greyed out text
      if Defaults.deferToShellAutosuggestions && buffer != nil && index == buffer!.endIndex {
        buffer = nil
        print("xterm: possible zsh autosuggest, break context")
      }
      guard buffer != nil, index != nil, index != buffer!.endIndex else { break }
      index = buffer!.index(after: index!)
      print("xterm: move cursor to the right by 1")
    case .historySearchBackward:
      if (historyIndex ==  0) {
        stash()
      }
      historyIndex += 1
      buffer = nil
      print("xterm: previous history")
    case .historySearchForward:
      if (historyIndex >= 0) {
        historyIndex -= 1
      }
      if (historyIndex == -1 && buffer == nil) {
        restore()
      }
      print("xterm: next history")
    case .beginningOfLine:
      guard buffer != nil, index != nil else { break }
      index = buffer!.startIndex
      print("xterm: move to start of current line")
    case .endOfLine:
      guard buffer != nil, index != nil else { break }
      index = buffer!.endIndex
      print("xterm: move to end of current line")
    case .historyIncrementalSearchBackward:
      buffer = nil // lost context
      print("xterm: reverse search history")
    case .historyIncrementalSearchForward:
      buffer = nil // lost context
      print("xterm: forwards search history")
    case .backwardDeleteChar:
      guard buffer != nil, index != nil, index != buffer!.startIndex else { break }
      index = buffer!.index(before: index!)
      buffer!.remove(at: index!)
      print("xterm: delete character")
    case .deleteCharOrList:
      guard buffer != nil, index != nil, index != buffer!.endIndex else { break }
      buffer!.remove(at: index!)
      print("xterm: delete following character")
    case .forwardSlash:
      // this is some iterm specific binding
      guard buffer != nil, index != nil, buffer!.count > 0 else { break }
      buffer!.remove(at: buffer!.index(before: buffer!.endIndex))
      index = buffer!.endIndex
      print("xterm: delete character from end")
    case .transposeChars:
      guard buffer != nil, index != nil, buffer!.count >= 2 else { break }
      if (buffer!.count == 2) {
        index = buffer!.endIndex
      }
      let second = buffer!.index(before: index!)
      let first = buffer!.index(before: second)
      let a = buffer![first]
      let b = buffer![second]
      buffer!.replaceSubrange(first...second, with: "\(b)\(a)")
      print("xterm: transpose")
    case .killWholeLine:
      buffer = ""
      index = buffer!.startIndex
      print("xterm: kill line")
    case .backwardKillWord:
      guard buffer != nil, index != nil else { break }
      let prefix = String(buffer![..<index!])
      var allowed = CharacterSet()
      allowed.formUnion(.whitespaces)
      allowed.insert("\"")
      allowed.insert("\\")
      allowed.insert(",")
      allowed.insert("'")
      allowed.insert(":")
      allowed.insert("`")
      allowed.insert("@")
      print("xterm:", prefix.trimTrailingCharacters(in: allowed).split(separator: " "))
      var killed = prefix.trimTrailingCharacters(in: allowed)
      killed = killed.split(separator: " ", maxSplits: Int.max, omittingEmptySubsequences: false).dropLast().joined(separator: " ") + " "
      // If deleting the first word, don't add an additional space
      if (killed == " ") {
        killed = ""
      }
      buffer!.replaceSubrange(buffer!.startIndex..<index!, with: killed)
      index = killed.endIndex
      print("xterm: kill the word behind point")
    case .yank:
      buffer = nil
      print("xterm: yank from kill ring") // lost context
    case .sendBreak:
      buffer = ""
      index = buffer!.startIndex
      print("xterm: abort") // clear buffer
    case .acceptLine:
      if buffer != nil, index != nil, buffer!.suffix(1) == "\\" {
        buffer = nil
        print("xterm: accept-line w/ newline")
      } else {
        buffer = ""
        NotificationCenter.default.post(name: Self.lineAcceptedInKeystrokeBufferNotification, object: nil)
        print("xterm: accept-line") //clear buffer
      }
    case .ctrlTwo:
      buffer = nil
      print("xterm: set-mark") //lost context
    case .expandOrComplete:
      buffer = nil
      print("xterm: complete")
    case .killProcess:
      buffer = ""
      print("xterm: kill process")
    default:
      guard buffer != nil, index != nil else { break }
      if let characters = event.characters {
        var skip = 0
        for (idx, char) in characters.enumerated() {
          guard char.asciiValue != nil else { break }
          guard skip == 0 else { skip -= 1; break}
          print("char:",char, char.utf16, char.asciiValue ?? 0)
          switch char.asciiValue! {
          case 8: //backspace literal
            guard buffer != nil, index != nil, index != buffer!.startIndex else { break }
            index = buffer!.index(before: index!)
            buffer!.remove(at: index!)
            print("xterm: delete character")
          case 27: // ESC
            if let direction = characters.index(characters.startIndex, offsetBy: idx + 2, limitedBy: characters.endIndex) {
              let esc = characters[direction]
              print("char:",esc)
              if (esc == "D") { //backward one
                guard buffer != nil, index != nil,  index != buffer!.startIndex else { break }
                index = buffer!.index(before: index!)
                print("xterm: move cursor to the left by 1 (ESC[D)")
                skip = 2
              } else if (esc == "C") { // forward one
                guard buffer != nil, index != nil, index != buffer!.endIndex else { break }
                index = buffer!.index(after: index!)
                print("xterm: move cursor to the right by 1 (ESC[C)")
                skip = 2
              }
            }
            break
          case 10: // newline literal
            if buffer != nil, index != nil, buffer!.suffix(1) == "\\" {
              buffer = nil
              print("xterm: accept-line w/ newline")
            } else {
              buffer = ""
              print("xterm: accept-line") //clear buffer
              NotificationCenter.default.post(name: Self.lineAcceptedInKeystrokeBufferNotification, object: nil)
            }
          default:
            
            // some umapped command - cmdc shouldn't insert a c into buffer
            let isFigInsertion = keystroke.keyCode == 0
            if (!isFigInsertion && !keystroke.modifierFlags.isEmpty) {
              return nil
            }
            
            if (event.modifierFlags.contains(.option)) {
              var char = UniChar()
              var length = 0
              event.cgEvent?.keyboardGetUnicodeString(maxStringLength: 1, actualStringLength: &length, unicodeString: &char)
              let string = String(UnicodeScalar(char)!)
              print("unichar: \(string)")
              // lost context
              if (string.count == 0) {
                buffer = nil
                hazy = true
                return nil
              }
            }
            
            buffer!.insert(char, at: index!)
            index = buffer!.index(index!, offsetBy: 1)
            print("xterm: insert! (\(char))")
          }
        }
      }
      break
    }
    
    if var logging = buffer, index != nil {
      // todo: check if index is within bounds
      logging.insert("|", at: index!)
      print("xterm-out: \(logging) ")
      return (buffer!, index!.utf16Offset(in: buffer!))
    } else {
      print("xterm-out: <no context> ")
      return nil
    }
  }
  
  var representation: String {
    if var logging = buffer, index != nil {
      // todo: check if index is within bounds
      logging.insert("|", at: index!)
      return logging
    } else {
      return "<no context>"
    }
  }
}

extension String {
  mutating func swap(at index: String.Index, to character: Character) {
    let endIndex = self.index(after: index)
    let range = index ..< endIndex
    assert(indices.contains(index) && indices.contains(endIndex))
    replaceSubrange(range, with: String(character))
  }
  
  func trimTrailingCharacters(in characterSet : CharacterSet) -> String {
    if let range = rangeOfCharacter(from: characterSet, options: [.anchored, .backwards]) {
      return String(self[..<range.lowerBound]).trimTrailingCharacters(in: characterSet)
    }
    return self
  }
}
