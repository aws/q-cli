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
  // swiftlint:disable identifier_name
  static let lineAcceptedInKeystrokeBufferNotification: NSNotification.Name =
    .init("lineAcceptedInXTermBufferNotification")

  static let defaultPath = URL(fileURLWithPath: "/tmp/figterm-input.socket")

  static func path(for sessionId: SessionId) -> String {
    // We aren't using NSTemporaryDirectory because length of socket path is capped at 104 characters
    return "/tmp/figterm-\(sessionId).socket"
  }

  static func updateBuffer(_ update: Fig_TextUpdate,
                           into session: SessionId,
                           wrapWithFigMessage: Bool = true,
                           figtermManagesInsertionLock: Bool = true) throws {

    try connect(to: session) { socket in

      if !figtermManagesInsertionLock {
        ShellInsertionProvider.insertLock()
      }

      let figtermMessage = Figterm_FigtermMessage.with { msg in
        msg.insertTextCommand = Figterm_InsertTextCommand.with({ insert in
          insert.deletion = UInt64(update.deletion)
          insert.insertion = update.insertion
          insert.offset = update.offset
          insert.immediate = update.immediate
        })
      }

      let seralizedFigtermMessage = try figtermMessage.serializedData()

      var data = Data()
      if wrapWithFigMessage {
        data.append(contentsOf: "\u{001b}@fig-pbuf".utf8)
        data.append(contentsOf: Data(from: Int64(seralizedFigtermMessage.count).bigEndian))
      }
      data.append(contentsOf: seralizedFigtermMessage)

      socket.send(data: data)

      if !figtermManagesInsertionLock {
        ShellInsertionProvider.insertUnlock(deletion: Int(update.deletion),
                                            insertion: update.insertion,
                                            offset: Int(update.offset),
                                            immediate: update.immediate)
      } else {
        Defaults.shared.incrementKeystokesSaved(by: Int(update.deletion) + update.insertion.count)

        if update.immediate {
          NotificationCenter.default.post(name: Self.lineAcceptedInKeystrokeBufferNotification, object: nil)
        }
      }
    }
  }

  //
  static func insert(_ text: String,
                     into session: SessionId,
                     wrapWithFigMessage: Bool = true,
                     figtermManagesInsertionLock: Bool = true) throws {

    try updateBuffer(Fig_TextUpdate.with({ update in
      update.deletion = 0
      update.insertion = text
      update.offset = 0
      update.immediate = false
    }), into: session, wrapWithFigMessage: wrapWithFigMessage,
                     figtermManagesInsertionLock: figtermManagesInsertionLock)
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

extension FigTerm {
  static let insertedTextNotification: NSNotification.Name = Notification.Name("insertedTextNotification")

  fileprivate static let rustRewriteIncludedInVersion = 6
  fileprivate static let addedFigtermMessageInVersion = 7
  fileprivate static let figtermManagesInsertionLockInVersion = 8

  static func handleInsertRequest(_ request: Fig_InsertTextRequest) throws -> Bool {

    guard let window = AXWindowServer.shared.allowlistedWindow,
          let session = window.session else {
        throw APIError.generic(message: "Could not determine session associated with window.")
    }

    let integrationVersion = window.associatedShellContext?.integrationVersion ?? 0

    switch request.type {
    case .text(let text):

      switch integrationVersion {
      case 0..<rustRewriteIncludedInVersion:
        // if session is still using c-figterm, send raw text
        try FigTerm.legacyInsert(text, into: session)
      case rustRewriteIncludedInVersion:
        try FigTerm.insert(text, into: session, wrapWithFigMessage: false)
      case addedFigtermMessageInVersion:
        try FigTerm.insert(text, into: session, figtermManagesInsertionLock: false)
      case figtermManagesInsertionLockInVersion, _:
        try FigTerm.insert(text, into: session)
      }

    case .update:

      if integrationVersion >= figtermManagesInsertionLockInVersion {
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
