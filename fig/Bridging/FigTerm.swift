//
//  FigTerm.swift
//  fig
//
//  Created by Matt Schrage on 10/21/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class FigTerm {
  static let defaultPath = URL(fileURLWithPath: "/tmp/figterm-input.socket")
  
  static func path(for sessionId: SessionId) -> String {
    // We aren't using NSTemporaryDirectory because length of socket path is capped at 104 characters
    return "/tmp/figterm-\(sessionId).socket"
  }
  static func insert(_ text: String, into session: SessionId) throws {
    let socket = UnixSocketClient(path: path(for: session))
    guard socket.connect() else {
      return //throw
    }
    ShellInsertionProvider.insertLock()
    socket.send(message: text)
    ShellInsertionProvider.insertUnlock(with: text)
    socket.disconnect()
  }
  
}

import FigAPIBindings
extension FigTerm {
  static let insertedTextNotification: NSNotification.Name = Notification.Name("insertedTextNotification")

  static func handleInsertRequest(_ request: Fig_InsertTextRequest) throws -> Bool {
    switch request.type {
    case .text(let text):
      guard let window = AXWindowServer.shared.whitelistedWindow,
            let session = window.session else {
        return false
      }
      NotificationCenter.default.post(name: FigTerm.insertedTextNotification, object: nil)
      try FigTerm.insert(text, into: session)
    case .update(_):
      throw APIError.generic(message: "Not supported yet.")
    default:
      break
    }
    
    return false
  }
}
