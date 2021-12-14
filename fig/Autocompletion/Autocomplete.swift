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

  static let throttler = Throttler(minimumDelay: 0.001)
  
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
//    let delay = 0.05// min(0.01 * Double(insertionText.count), 0.15)
//    Timer.delayWithSeconds(delay) {
        try? FileManager.default.removeItem(atPath: insertionLock)

      if let window = AXWindowServer.shared.whitelistedWindow,
         let sessionId = window.session,
         let editBuffer = window.associatedEditBuffer {
            
            var text = editBuffer.text
            var index = text.index(text.startIndex, offsetBy: editBuffer.cursor)

            var skip = 0
            for (idx, char) in insertionText.enumerated() {
                guard let value = char.asciiValue else { break }
                guard skip == 0 else { skip -= 1; break }
                switch value {
                case 8: //backspace literal
                    guard index != text.startIndex else { break }
                    index = text.index(before: index)
                    text.remove(at: index)
                case 27: // ESC
                    if let direction = insertionText.index(insertionText.startIndex, offsetBy: idx + 2, limitedBy: insertionText.endIndex) {
                    let esc = insertionText[direction]
                        if (esc == "D") {
                            guard index != text.startIndex else { break }
                            index = text.index(before: index)
                            skip = 2
                        } else if (esc == "C") { // forward one
                            guard index != text.endIndex else { break }
                            index = text.index(after: index)
                            skip = 2
                        }
                    }
                    break
                case 10: // newline literal
                    text = ""
                    index = text.startIndex
                    NotificationCenter.default.post(name: Self.lineAcceptedInKeystrokeBufferNotification, object: nil)
                default:
                    guard text.endIndex >= index else { return }
                    text.insert(char, at: index)
                  index = text.index(index, offsetBy: 1, limitedBy: text.endIndex) ?? text.endIndex
                }
            }
          
            let cursor = text.distance(from: text.startIndex, to: index)
          TerminalSessionLinker.shared.setEditBuffer(for: sessionId,
                                                      text: text,
                                                      cursor: cursor)
        
          API.notifications.editbufferChanged(buffer: text,
                                              cursor: cursor,
                                              session: sessionId,
                                              context: window.associatedShellContext?.ipcContext)
        
          Autocomplete.position()
//      }
    }
  }
}

extension String {
  
    func isAlphanumeric() -> Bool {
        return self.rangeOfCharacter(from: CharacterSet.alphanumerics.inverted) == nil && self != ""
    }
}
