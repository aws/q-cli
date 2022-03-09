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

  static let throttler = Throttler(minimumDelay: 0.01)

  static func runJavascript(_ command: String) {
    WindowManager.shared.autocomplete?.webView?.evaluateJavaScript(
      "try{ \(command) } catch(e) { console.log(e) }",
      completionHandler: nil
    )
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
    guard let window = AXWindowServer.shared.allowlistedWindow else {
      return
    }

    throttler.throttle {
      DispatchQueue.main.async {
        if let rect = window.cursor {
          WindowManager.shared.positionAutocompletePopover(
            textRect: rect,
            makeVisibleImmediately: makeVisibleImmediately,
            completion: nil
          )
        }
      }
    }
  }
}

// This is legacy code and should be removed after the transition
// to locks internal locks in figterm
class ShellInsertionProvider {
  static let insertionLock = "\(NSHomeDirectory())/.fig/insertion-lock"

  // swiftlint:disable identifier_name
  static let lineAcceptedInKeystrokeBufferNotification: NSNotification.Name =
    .init("lineAcceptedInXTermBufferNotification")

  static func insertLock() {
    NotificationCenter.default.post(name: FigTerm.insertedTextNotification, object: nil)
    FileManager.default.createFile(atPath: insertionLock, contents: nil, attributes: nil)
  }

  static func insertUnlock(with insertionText: String) {
    insertUnlock { text, index in

        var skip = 0
        for (idx, char) in insertionText.enumerated() {
          guard let value = char.asciiValue else { break }
          guard skip == 0 else { skip -= 1; break }
          switch value {
          case 8: // backspace literal
            guard index != text.startIndex else { break }
            index = text.index(before: index)
            text.remove(at: index)
          case 27: // ESC
            if let direction = insertionText.index(
              insertionText.startIndex,
              offsetBy: idx + 2,
              limitedBy: insertionText.endIndex
            ) {
              let esc = insertionText[direction]
              if esc == "D" {
                guard index != text.startIndex else { break }
                index = text.index(before: index)
                skip = 2
              } else if esc == "C" { // forward one
                guard index != text.endIndex else { break }
                index = text.index(after: index)
                skip = 2
              }
            }

          case 10: // newline literal
            text = ""
            index = text.startIndex
            NotificationCenter.default.post(name: Self.lineAcceptedInKeystrokeBufferNotification, object: nil)
          default:
            guard text.endIndex >= index else { break }
            text.insert(char, at: index)
            index = text.index(index, offsetBy: 1, limitedBy: text.endIndex) ?? text.endIndex
          }
        }
    }

    Defaults.shared.incrementKeystokesSaved(by: insertionText.count)

  }

  static func insertUnlock(deletion: Int, insertion: String, offset: Int, immediate: Bool) {

    insertUnlock { text, index in
      let startOfDeletion = text.index(index,
                                       offsetBy: -deletion,
                                       limitedBy: text.startIndex) ?? index

      if deletion > 0 {
        text.removeSubrange(startOfDeletion...index)
      }

      text.insert(contentsOf: insertion, at: startOfDeletion)

      let endOfInsertion = text.index(startOfDeletion, offsetBy: insertion.count)

      index = text.index(endOfInsertion, offsetBy: offset)

      if immediate {
        text = ""
        index = text.startIndex
        NotificationCenter.default.post(name: FigTerm.lineAcceptedInKeystrokeBufferNotification, object: nil)
      }
    }

    Defaults.shared.incrementKeystokesSaved(by: deletion + insertion.count)

  }

  fileprivate static func insertUnlock(textUpdateCallback: @escaping (inout String, inout String.Index) -> Void) {
    // remove lock after keystrokes have been processes
    // requires delay proportional to number of character inserted
    // unfortunately, we don't really know how long this will take
    // - it varies significantly between native and Electron terminals.
    // We can probably be smarter about this and modulate delay based on terminal.

      // remove lock after keystrokes have been processes

      if let window = AXWindowServer.shared.allowlistedWindow,
         let sessionId = window.session,
         let editBuffer = window.associatedEditBuffer {

        var text = editBuffer.text
        var index = text.index(text.startIndex, offsetBy: editBuffer.cursor)

        textUpdateCallback(&text, &index)

        let cursor = text.distance(from: text.startIndex, to: index)
        TerminalSessionLinker.shared.setEditBuffer(for: sessionId,
                                                   text: text,
                                                   cursor: cursor)

        API.notifications.editbufferChanged(buffer: text,
                                            cursor: cursor,
                                            session: sessionId,
                                            context: window.associatedShellContext?.ipcContext)

        Autocomplete.position()

    }
  }

}

extension String {

  func isAlphanumeric() -> Bool {
    return self.rangeOfCharacter(from: CharacterSet.alphanumerics.inverted) == nil && self != ""
  }
}
