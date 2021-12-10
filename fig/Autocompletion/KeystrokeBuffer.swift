//
//  Autocomplete.swift
//  fig
//
//  Created by Matt Schrage on 8/31/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class KeystrokeBuffer : NSObject {
  var index: String.Index?
  var backing: Backing?
  var shellCursor: Int = 0

  static let lineResetInKeyStrokeBufferNotification: NSNotification.Name = .init("lineResetInKeyStrokeBufferNotification")
  static let contextLostInKeystrokeBufferNotification: NSNotification.Name = .init("contextLostInKeystrokeBufferNotification")
  static let lineAcceptedInKeystrokeBufferNotification: NSNotification.Name = .init("lineAcceptedInXTermBufferNotification")
  static let firstCharacterInKeystrokeBufferNotification: NSNotification.Name = .init("firstCharacterInKeystrokeBufferNotification")

  override init() {
    buffer = ""
    index = buffer!.startIndex
  }
  
  var buffer: String? = nil {
    didSet {
      if buffer == nil {
        if (Defaults.shared.playSoundWhenContextIsLost) {
          NSSound.beep()
        }
        index = nil
        NotificationCenter.default.post(name: Self.contextLostInKeystrokeBufferNotification, object: nil)
      } else if (buffer == "") {
        NotificationCenter.default.post(name: Self.lineResetInKeyStrokeBufferNotification, object: nil)
        index = buffer!.startIndex
      } else if (buffer?.count == 1) {
        NotificationCenter.default.post(name: Self.firstCharacterInKeystrokeBufferNotification, object: nil)
      }
    }
  }
  
  func insert(text: String) -> (String, Int)? {
    guard buffer != nil else { return nil }
    self.index = buffer!.index(buffer!.startIndex, offsetBy: shellCursor, limitedBy: buffer!.endIndex) ?? buffer!.endIndex
    mutatingInsert(text: text)
    guard index != nil else { return nil }
    shellCursor = index!.utf16Offset(in: buffer!)
    return (buffer ?? "", shellCursor)
  }
  
  fileprivate func mutatingInsert(text: String) {
      var skip = 0
      for (idx, char) in text.enumerated() {
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
            if let direction = text.index(text.startIndex, offsetBy: idx + 2, limitedBy: text.endIndex) {
              let esc = text[direction]
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
            guard buffer != nil, index != nil, buffer!.endIndex >= index! else { return }
            buffer!.insert(char, at: index!)
            index = buffer!.index(index!, offsetBy: 1, limitedBy: buffer!.endIndex)
            print("xterm: insert! (\(char))")
          }
      }
  }
  
  var representation: String {
    if var logging = buffer {
      let index = logging.index(logging.startIndex, offsetBy: shellCursor, limitedBy: buffer!.endIndex) ?? buffer!.endIndex
      logging.insert("|", at: index)
      return logging
    }
    
    return "<no context>"
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

  func trimLeadingCharacters(in characterSet : CharacterSet) -> String {
    if let range = rangeOfCharacter(from: characterSet, options: [.anchored]) {
        return String(self.suffix(from: range.upperBound)).trimLeadingCharacters(in: characterSet)
    }
    return self
  }
}

extension KeystrokeBuffer {
  enum Backing: String {
    case zsh = "zsh"
    case fish = "fish"
    case bash = "bash"
  }
}
