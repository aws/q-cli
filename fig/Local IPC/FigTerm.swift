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
  static let insertedTextNotification: NSNotification.Name = Notification.Name("insertedTextNotification")
  fileprivate static let figtermManagesInsertionLockInVersion = 8

  // swiftlint:disable identifier_name
  static let lineAcceptedInKeystrokeBufferNotification: NSNotification.Name =
    .init("lineAcceptedInXTermBufferNotification")

  static func path(for sessionId: SessionId) -> String {
    // We aren't using NSTemporaryDirectory because length of socket path is capped at 104 characters
    return "/tmp/figterm-\(sessionId).socket"
  }

  static func updateBuffer(_ update: Fig_TextUpdate, into session: SessionId) throws {
    let insertRequest = Figterm_InsertTextRequest.with({ insert in
      // Optional proto fields are not Swift optionals: https://github.com/apple/swift-protobuf/issues/644
      if update.hasDeletion {
        insert.deletion = UInt64(update.deletion)
      }
      if update.hasInsertion {
        insert.insertion = update.insertion
      }
      if update.hasOffset {
        insert.offset = update.offset
      }
      if update.hasImmediate {
        insert.immediate = update.immediate
      }
      if update.hasInsertionBuffer {
        insert.insertionBuffer = update.insertionBuffer
      }
    })

    // Try secureIPC first.
    if (try? SecureIPC.shared.makeInsertTextRequest(for: session, with: insertRequest)) == nil {
      let figtermMessage = Figterm_FigtermRequestMessage.with { msg in
        msg.insertText = insertRequest
      }

      let socket = UnixSocketClient(path: path(for: session))
      guard socket.connect() else {
        let error = String(utf8String: strerror(errno)) ?? "Unknown error code"
        throw APIError.generic(message: "Could connected to \(path(for: session)). Error \(errno): \(error)")
      }

      try socket.send(message: figtermMessage)

      socket.disconnect()
    }

    Defaults.shared.incrementKeystrokesSaved(by: Int(update.deletion) + update.insertion.count)

    if update.immediate {
      NotificationCenter.default.post(name: Self.lineAcceptedInKeystrokeBufferNotification, object: nil)
    }
  }

  static func handleInsertRequest(_ request: Fig_InsertTextRequest) throws -> Bool {

    guard let window = AXWindowServer.shared.allowlistedWindow,
          let session = window.session else {
        throw APIError.generic(message: "Could not determine session associated with window.")
    }

    let integrationVersion = window.associatedShellContext?.integrationVersion ?? 0

    guard integrationVersion >= figtermManagesInsertionLockInVersion else {
      throw APIError.generic(message: "Outdated figterm version.")
    }

    switch request.type {
    case .text(let text):
      try FigTerm.updateBuffer(Fig_TextUpdate.with({ update in
        update.deletion = 0
        update.insertion = text
        update.offset = 0
        update.immediate = false
        update.clearInsertionBuffer()
      }), into: session)
    case .update:
      try FigTerm.updateBuffer(request.update, into: session)
    default:
      break
    }

    return false
  }
}
