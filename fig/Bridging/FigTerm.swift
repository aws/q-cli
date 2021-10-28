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
  
  // todo: update path depending on sessionId
  static func path(for sessionId: SessionId) -> String {
    return "/tmp/figterm-\(sessionId).socket"
      //FileManager.default.temporaryDirectory.appendingPathComponent("figterm-" + sessionId + ".socket").path
  }
  static func insert(_ text: String, into session: SessionId) throws {
    let socket = UnixSocketClient(path: path(for: session))
    guard socket.connect() else {
      return //throw
    }
    GenericShellIntegration.insertLock()
    socket.send(message: text)
    GenericShellIntegration.insertUnlock(with: text)
    socket.disconnect()
  }
  
}

import FigAPIBindings
extension FigTerm {
  static func handleInsertRequest(_ request: Fig_InsertTextRequest) throws -> Bool {
    switch request.type {
    case .text(let text):
      guard let window = AXWindowServer.shared.whitelistedWindow,
            let session = window.session else {
        return false
      }
      
      try FigTerm.insert(text, into: session)
    case .update(_):
      throw APIError.generic(message: "Not supported yet.")
    default:
      break
    }
    
    return false
  }
}
