//
//  FigTerm.swift
//  fig
//
//  Created by Matt Schrage on 10/21/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import FigAPIBindings

class FigTerm {
  static let defaultPath = URL(fileURLWithPath: "/tmp/figterm-input.socket")

  static func path(for sessionId: SessionId) -> String {
    // We aren't using NSTemporaryDirectory because length of socket path is capped at 104 characters
    return "/tmp/figterm-\(sessionId).socket"
  }

  static func updateBuffer(_ update: Fig_TextUpdate, into session: SessionId) throws {

    try connect(to: session) { socket in

      ShellInsertionProvider.insertLock()

      let msg = Figterm_FigtermMessage.with { msg in
        msg.insertTextCommand = Figterm_InsertTextCommand.with({ insert in
          insert.deletion = UInt64(update.deletion)
          insert.insertion = update.insertion
          insert.offset = update.offset
          insert.immediate = update.immediate
        })
      }

      socket.send(data: try msg.serializedData())

      ShellInsertionProvider.insertUnlock(deletion: Int(update.deletion),
                                          insertion: update.insertion,
                                          offset: Int(update.offset),
                                          immediate: update.immediate)

    }
  }

  //
  static func insert(_ text: String, into session: SessionId) throws {

    try updateBuffer(Fig_TextUpdate.with({ update in
      update.deletion = 0
      update.insertion = text
      update.offset = 0
      update.immediate = false
    }), into: session)
  }

  // `legacyInsert` is used to write text to the C-implementation of figterm.
  // `text` is sent directly to the socket with no framing protocol

  static func legacyInsert(_ text: String, into session: SessionId) throws {

    try connect(to: session) { socket in

      ShellInsertionProvider.insertLock()

      socket.send(message: text)

      ShellInsertionProvider.insertUnlock(with: text)

    }
  }

  fileprivate static func connect(to session: SessionId, connectCallback: ((UnixSocketClient) throws -> Void)) throws {

    let socket = UnixSocketClient(path: path(for: session))
    guard socket.connect() else {
      let error = String(utf8String: strerror(errno)) ?? "Unknown error code"
      throw APIError.generic(message: "Could connected to \(path(for: session)). Error \(errno): \(error)")
    }

    try connectCallback(socket)

    socket.disconnect()
  }

}

import FigAPIBindings
extension FigTerm {
  static let insertedTextNotification: NSNotification.Name = Notification.Name("insertedTextNotification")

  fileprivate static let rustRewriteIncludedInVersion = 6

  static func handleInsertRequest(_ request: Fig_InsertTextRequest) throws -> Bool {

    guard let window = AXWindowServer.shared.allowlistedWindow,
          let session = window.session else {
        throw APIError.generic(message: "Could not determine session associated with window.")
    }

    let integrationVersion = window.associatedShellContext?.integrationVersion ?? 0

    switch request.type {
    case .text(let text):

      // if session is still using c-figterm, send raw text
      if integrationVersion >= rustRewriteIncludedInVersion {
        try FigTerm.insert(text, into: session)
      } else {
        try FigTerm.legacyInsert(text, into: session)
      }
    case .update:

      if integrationVersion >= rustRewriteIncludedInVersion {
        try FigTerm.updateBuffer(request.update, into: session)
      } else {
        throw APIError.generic(message: "Not supported yet.")
      }
    default:
      break
    }

    return false
  }
}
