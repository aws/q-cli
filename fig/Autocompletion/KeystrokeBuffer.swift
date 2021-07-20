//
//  Autocomplete.swift
//  fig
//
//  Created by Matt Schrage on 8/31/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class KeystrokeBuffer : NSObject {
  
  var currentState: (String, Int)? {
    guard !writeOnly else {
      return nil
    }
    
    if self.backedByShell {
      return (buffer ?? "", shellCursor)
    } else {
      guard buffer != nil, index != nil else { return nil }
      return (buffer!, index!.utf16Offset(in: buffer!))
    }
  }
  
  // whether a keybuffer starts in writeOnly mode (historically has been `false`)
  static var initialWritingMode: Bool {
    get {
      return (Settings.shared.getValue(forKey: Settings.onlyShowOnTabKey) as? Bool) ?? false
    }
  }
  var historyIndex = 0
  var index: String.Index?
  var stashedBuffer: String?
  var stashedIndex: String.Index?
  var writeOnly = KeystrokeBuffer.initialWritingMode // update buffer, but don't return it (prevents keypress events from being sent to autocomplete)
    {
    didSet {
      print("writeOnly: \(writeOnly)")
      if (writeOnly) {
        NotificationCenter.default.post(name: Self.contextLostInKeystrokeBufferNotification, object: nil)
      } else if self.buffer != nil {
        NotificationCenter.default.post(name: Self.contextRestoredInKeystrokeBufferNotification, object: nil)
      }
    }
  }
  // updates are recieved directly from ZLE when this is true,
  // so no need to process keypress events directly
  var backedByShell = false {
    didSet {
      if (!backedByShell) {
        buffer = ""
        shellHistoryNumber = nil
        backing = nil
      }
    }
  }
  
  var backing: Backing?
  var shellCursor: Int = 0
  var shellHistoryNumber: Int? {
    didSet {
      
      // reset writeOnly value when line number changes
      // so that even if escape has been pressed previous
      // the autocomplete window will reappear
      if (shellHistoryNumber != oldValue) {
        writeOnly = KeystrokeBuffer.initialWritingMode
      }
      
    }
  }

  static let contextRestoredInKeystrokeBufferNotification: NSNotification.Name = .init("contextRestoredInKeystrokeBufferNotification")
  static let lineResetInKeyStrokeBufferNotification: NSNotification.Name = .init("lineResetInKeyStrokeBufferNotification")
  static let contextLostInKeystrokeBufferNotification: NSNotification.Name = .init("contextLostInKeystrokeBufferNotification")
  static let lineAcceptedInKeystrokeBufferNotification: NSNotification.Name = .init("lineAcceptedInXTermBufferNotification")
  static let firstCharacterInKeystrokeBufferNotification: NSNotification.Name = .init("firstCharacterInKeystrokeBufferNotification")

  override init() {
    buffer = ""
    index = buffer!.startIndex
  }
  
  var hazy: Bool = true {
    didSet {
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
        NotificationCenter.default.post(name: Self.contextLostInKeystrokeBufferNotification, object: nil)

      } else if (buffer == "") {
        NotificationCenter.default.post(name: Self.lineResetInKeyStrokeBufferNotification, object: nil)
        index = buffer!.startIndex
        dropStash()
        writeOnly = KeystrokeBuffer.initialWritingMode
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
  
  func insert(text: String) -> (String, Int)? {
    guard backedByShell, buffer != nil else { return nil }
    self.index = buffer!.index(buffer!.startIndex, offsetBy: shellCursor, limitedBy: buffer!.endIndex) ?? buffer!.endIndex
    mutatingInsert(text: text)
    //buffer!.insert(contentsOf: text, at: index)
    //let updatedIndex = buffer!.index(index, offsetBy: text.count)
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
  
  func handleKeystroke(event: NSEvent) -> (String, Int)? {
    guard !backedByShell else {
      return writeOnly ? nil : (buffer ?? "", shellCursor)
    }
    let cleanedFlags = event.modifierFlags.intersection([.command, .control, .option, .shift])
    let keystroke = Keystroke(modifierFlags: cleanedFlags, keyCode: event.keyCode)
    switch KeyBindingsManager.keyBindings[keystroke] {
    case .paste:
      guard let pasteboard = NSPasteboard.general.string(forType: .string), buffer != nil, index != nil else { break }
      buffer!.insert(contentsOf: pasteboard, at: index!)
      index = buffer!.index(index!, offsetBy: pasteboard.count, limitedBy: buffer!.endIndex)
      print("xterm: Paste! (\(pasteboard))")
    case .backwardChar:
      guard buffer != nil, index != nil, index != buffer!.startIndex else { break }
      index = buffer!.index(before: index!)
      print("xterm: move cursor to the left by 1")
    case .forwardChar:
      guard buffer != nil, index != nil, index != buffer!.endIndex else { break }
      index = buffer!.index(after: index!)
      print("xterm: move cursor to the right by 1")
    case .backwardWord:
        guard buffer != nil, index != nil else { break }
        
        let prefix = String(buffer![..<index!])
        
        // strip leading whitespace
        let stripWhitespace = prefix.trimTrailingCharacters(in: CharacterSet.whitespaces)
        
        // strip all-nonwhitespace characters
        let notWhitespace = stripWhitespace.trimTrailingCharacters(in: CharacterSet.whitespaces.inverted)
        
        print("xterm: ", notWhitespace)
        //prefix.distance(from: notWhitespace.endIndex, to: prefix.endIndex)
        index = notWhitespace.endIndex
        print("xterm: move cursor to the left by 1 word")

    case .forwardWord:
        guard buffer != nil, index != nil
            else { break }
        
        let suffix = String(buffer!.suffix(from: index!))

        // strip all-nonwhitespace characters
        let notWhitespace = suffix.trimLeadingCharacters(in: CharacterSet.whitespaces.inverted)
        
        // strip leading whitespace
        let stripWhitespace = notWhitespace.trimLeadingCharacters(in: CharacterSet.whitespaces)
        
        print("xterm: ", stripWhitespace)
        // if the range can't be found, then we've reached the end of the string
        guard let range = suffix.range(of: stripWhitespace) else {
            index = buffer!.endIndex
            break
        }
        let dist = suffix.distance(from: suffix.startIndex, to: range.lowerBound)
        index = buffer!.index(index!, offsetBy: dist, limitedBy: buffer!.endIndex)
        print("xterm: move cursor to the right by 1 word")

    case .historySearchBackward:
      if (historyIndex ==  0) {
        stash()
      }
      historyIndex += 1
      print("xterm: previous history")
    case .historySearchForward:
      if (historyIndex >= 0) {
        historyIndex -= 1
      }
      if (historyIndex == 0 && buffer == nil) {
        restore()
      }
      
      if (historyIndex <= -1) {
        writeOnly = KeystrokeBuffer.initialWritingMode
        historyIndex = 0
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
    case .setMarkCommand:
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
      
      // some umapped command - cmdc shouldn't insert a c into buffer but shift P should
      guard keystroke.modifierFlags.intersection([.command, .control]).isEmpty else { break }
      
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
      
      if let characters = event.characters {
        mutatingInsert(text: characters)
      }
      break
    }
    
    if var logging = buffer, index != nil, logging.endIndex >= index!, !writeOnly {
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
    guard !writeOnly else {
      return "<hidden>"
    }
    
    guard !backedByShell else {
      if var logging = buffer {
        let index = logging.index(logging.startIndex, offsetBy: shellCursor, limitedBy: buffer!.endIndex) ?? buffer!.endIndex
        logging.insert("|", at: index)
        return logging
      }
      
      return "<no context>"
    }
    
    if var logging = buffer, index != nil, buffer!.endIndex >= index! {
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

  func trimLeadingCharacters(in characterSet : CharacterSet) -> String {
    if let range = rangeOfCharacter(from: characterSet, options: [.anchored]) {
        return String(self.suffix(from: range.upperBound)).trimLeadingCharacters(in: characterSet)
    }
    return self
  }
}

extension KeystrokeBuffer {
  enum Backing: String {
    case zle = "zle"
    case fish = "fish"
    case bash = "bash"
  }
}
