//
//  Autocomplete.swift
//  fig
//
//  Created by Matt Schrage on 2/2/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Autocomplete {
  // todo: load global actions from ~/.fig/apps/autocomplete/actions.json
  static let globalActions = ["toggleAutocomplete", "showAutocomplete"]

  static let throttler = Throttler(minimumDelay: 0.05)
  
  static func runJavascript(_ command: String) {
    WindowManager.shared.autocomplete?.webView?.evaluateJavaScript("try{ \(command) } catch(e) { console.log(e) }", completionHandler: nil)
  }
  static func redirect(keyCode: UInt16, event: CGEvent, for windowHash: ExternalWindowHash) {
    
    guard let event =  NSEvent(cgEvent: event) else { return }
    let characters = event.characters ?? ""
      
    WindowManager.shared.autocomplete?.webView?.evaluateJavaScript(
      """
      try{
        fig.keypress(\"\(keyCode)\", \"\(windowHash)\", {
            command: \(event.modifierFlags.contains(.command)),
            control: \(event.modifierFlags.contains(.control)),
              shift: \(event.modifierFlags.contains(.shift)),
           isRepeat: \(event.isARepeat),
         characters: \"\(characters.isAlphanumeric() ? characters : "")\" })
      } catch(e) {}
      """, completionHandler: nil)
  }
  
  static func hide() {  
    WindowManager.shared.positionAutocompletePopover(textRect: nil)
  }
  
  static func position(makeVisibleImmediately: Bool = true) {
    guard let window = AXWindowServer.shared.whitelistedWindow else {
      return
    }
    
    throttler.throttle {
      DispatchQueue.main.async {
        if let rect = window.cursor {
          WindowManager.shared.positionAutocompletePopover(textRect: rect, makeVisibleImmediately: makeVisibleImmediately, completion: nil)
        }
      }
    }
  }
}

class ShellInsertionProvider {
  static let insertionLock = "\(NSHomeDirectory())/.fig/insertion-lock"

  static let lineAcceptedInKeystrokeBufferNotification: NSNotification.Name = .init("lineAcceptedInXTermBufferNotification")

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

        if let window = AXWindowServer.shared.whitelistedWindow, window.bufferInfo.backing != nil {
            var text = window.bufferInfo.text
            var cursor = text.index(text.startIndex, offsetBy: window.bufferInfo.cursor)

            var skip = 0
            for (idx, char) in insertionText.enumerated() {
                guard let value = char.asciiValue else { break }
                guard skip == 0 else { skip -= 1; break }
                switch value {
                case 8: //backspace literal
                    guard cursor != text.startIndex else { break }
                    cursor = text.index(before: cursor)
                    text.remove(at: cursor)
                case 27: // ESC
                    if let direction = insertionText.index(insertionText.startIndex, offsetBy: idx + 2, limitedBy: insertionText.endIndex) {
                    let esc = insertionText[direction]
                        if (esc == "D") {
                            guard cursor != text.startIndex else { break }
                            cursor = text.index(before: cursor)
                            skip = 2
                        } else if (esc == "C") { // forward one
                            guard cursor != text.endIndex else { break }
                            cursor = text.index(after: cursor)
                            skip = 2
                        }
                    }
                    break
                case 10: // newline literal
                    text = ""
                    cursor = text.startIndex
                    NotificationCenter.default.post(name: Self.lineAcceptedInKeystrokeBufferNotification, object: nil)
                default:
                    guard text.endIndex >= cursor else { return }
                    text.insert(char, at: cursor)
                  cursor = text.index(cursor, offsetBy: 1, limitedBy: text.endIndex) ?? text.endIndex
                }
            }
            
            window.bufferInfo = KeystrokeBuffer(
              backing: window.bufferInfo.backing,
              cursor: text.distance(from: text.startIndex, to: cursor),
              text: text)
        }
    }
  }
}

extension String {
  
    func isAlphanumeric() -> Bool {
        return self.rangeOfCharacter(from: CharacterSet.alphanumerics.inverted) == nil && self != ""
    }
}
