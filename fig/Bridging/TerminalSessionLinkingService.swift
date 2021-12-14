//
//  TerminalSessionLinkingService.swift
//  fig
//
//  Created by Matt Schrage on 11/29/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa

protocol WorkspaceService {
  var frontmostApplication: NSRunningApplication? { get }
  var runningApplications: [NSRunningApplication] { get }
}

protocol TerminalSessionLinkingService {

  func linkWithFrontmostWindow(sessionId: TerminalSessionId?, isFocused: Bool?) throws
  func link(windowId: WindowId,
            bundleId: String,
            terminalSessionId: TerminalSessionId,
            focusId: FocusId?,
            isFocused: Bool?)
  func focusedTerminalSession(for windowId: WindowId) -> TerminalSession?
  func getTerminalSession(for terminalSessionId: TerminalSessionId) -> TerminalSession?

}

typealias WindowId = CGWindowID
typealias TerminalSessionId = String
typealias FocusId = String

struct ShellContext {
  let processId: Int32
  let executablePath: String
  let ttyDescriptor: String
  let workingDirectory: String
  let integrationVersion: Int?
}

extension ShellContext {
  // todo(mschrage): this is for backwards compatiblity and can likely be removed
  func isShell() -> Bool {
    return ["zsh", "fish", "bash"].reduce(into: false) { (res, shell) in
      res = res || self.executablePath.contains(shell)
    }
  }
}

struct EditBuffer {
  var cursor: Int
  var text: String

  var representation: String {
    get {
      var bufferCopy = text
      let index = text.index(text.startIndex, offsetBy: cursor, limitedBy: text.endIndex) ?? text.endIndex
      bufferCopy.insert("|", at: index)
      return bufferCopy
    }
  }
}

struct TerminalSession {
  let windowId: WindowId
  let bundleId: String
  let terminalSessionId: TerminalSessionId

  var shellContext: ShellContext?
  var editBuffer: EditBuffer?
  let focusId: FocusId?
  var isFocused: Bool = false
}

// todo(mschrage): remove this!
extension TerminalSession {
  func generateLegacyWindowHash() -> ExternalWindowHash {
    return "\(self.windowId)/\(self.focusId ?? "")%"
  }
}

enum LinkingError: Error {
  case noTerminalSessionId

  case noWindowCandidateAvailable

  case couldNotDetermineFrontmostApplication
}

class TerminalSessionLinker: TerminalSessionLinkingService {
  // temporarily use a singleton
  static let shared = TerminalSessionLinker(windowService: AXWindowServer.shared)
  let windowService: WindowService
  let queue: DispatchQueue = DispatchQueue(label: "io.fig.session-linker")

  // `windows` is used to quickly index into `sessions` to locate TerminalSession for a given TerminalSessionId
  fileprivate var windows: [ TerminalSessionId: WindowId ] = [:]
  fileprivate var sessions: [WindowId : [ TerminalSessionId: TerminalSession ]] = [:]

  // MARK: - Lifecyle

  init(windowService: WindowService) {
    self.windowService = windowService

    NotificationCenter.default.addObserver(self,
                                           selector: #selector(processEditbufferHook),
                                           name: IPC.Notifications.editBuffer.notification,
                                           object: nil)

    NotificationCenter.default.addObserver(self,
                                           selector: #selector(processKeyboardFocusChangedHook),
                                           name: IPC.Notifications.keyboardFocusChanged.notification,
                                           object: nil)

    NotificationCenter.default.addObserver(self,
                                           selector: #selector(processPromptHook),
                                           name: IPC.Notifications.prompt.notification,
                                           object: nil)

  }

  deinit {
    NotificationCenter.default.removeObserver(self)
  }

  // MARK: - Notification
  @objc func processEditbufferHook(notification: Notification) {
    guard let event = notification.object as? Local_EditBufferHook else {
      return
    }

    do {
      let terminalSessionId = event.context.hasSessionID ? event.context.sessionID : nil

      try self.linkWithFrontmostWindow(sessionId: terminalSessionId,
                                       isFocused: true)

      if let sessionId = terminalSessionId,
         let shellContext = event.context.internalContext {
        self.setShellContext(for: sessionId, context: shellContext)
        self.setEditBuffer(for: sessionId, text: event.text, cursor: Int(event.cursor))
      }

    } catch let error {
      print(error)
    }
  }

  @objc func processKeyboardFocusChangedHook(notification: Notification) {
    guard let event = notification.object as? Local_KeyboardFocusChangedHook else {
      return
    }

    guard let window = windowService.topmostWhitelistedWindow() else {
      return
    }

    guard event.appIdentifier == window.bundleId else {
      return
    }

    // reset focus for all sessions associated with frontmost window
    // so that the sessionId of a new tab is `nil` until updated on keypress
    resetFocusForAllSessions(in: window.windowId)
  }

  @objc func processPromptHook(notification: Notification) {
    guard let event = notification.object as? Local_PromptHook else {
      return
    }

    guard event.context.hasSessionID,
          event.context.hasPid else {
      return
    }

    let workingDirectory = event.context.hasCurrentWorkingDirectory
      ? event.context.currentWorkingDirectory
      : ProcessStatus.workingDirectory(for: event.context.pid)

    let context = ShellContext(processId: event.context.pid,
                               executablePath: event.context.processName,
                               ttyDescriptor: event.context.ttys,
                               workingDirectory: workingDirectory,
                               integrationVersion: Int(event.context.integrationVersion))

    self.setShellContext(for: event.context.sessionID,
                         context: context)
  }

  // MARK: - Link Session with Window

  func resetFocusForAllSessions(in windowId: WindowId) {
    self.queue.sync { [weak self] in
      guard self != nil else { return }
      self!.sessions[windowId] =
        self!.sessions[windowId]?.mapValues({ session -> TerminalSession in
          var updatedSession = session
          updatedSession.isFocused = false
          return updatedSession
        })
    }
  }

  func linkWithFrontmostWindow(sessionId: TerminalSessionId?, isFocused: Bool?) throws {

    guard let sessionId = sessionId else {
      throw LinkingError.noTerminalSessionId
    }
    guard let window = windowService.topmostWhitelistedWindow(), let bundleId = window.bundleId else {
      throw LinkingError.noWindowCandidateAvailable
    }

    link(windowId: window.windowId,
         bundleId: bundleId,
         terminalSessionId: sessionId,
         focusId: window.lastTabId,
         isFocused: isFocused)

  }

  func link(windowId: WindowId,
            bundleId: String,
            terminalSessionId: TerminalSessionId,
            focusId: FocusId?,
            isFocused: Bool?) {

    // if focus state is not explictly passed attempt to use current state, if it exists.
    let isFocused = isFocused ?? self.sessions[windowId]?[terminalSessionId]?.isFocused ?? false

    let terminalSession = TerminalSession(windowId: windowId,
                                          bundleId: bundleId,
                                          terminalSessionId: terminalSessionId,
                                          focusId: focusId,
                                          isFocused: isFocused)

    // reset focus on all other sessions
    resetFocusForAllSessions(in: windowId)

    updateTerminalSessionForWindow(windowId, session: terminalSession)

  }

  // MARK: - Getters

  func focusedTerminalSession(for windowId: WindowId) -> TerminalSession? {
    guard let sessions = self.sessions[windowId]?.values else { return nil }

    var focusedSession: TerminalSession?
    for session in sessions {
      if session.isFocused {
        assert(focusedSession == nil, "There should only be one focused session per window.")
        focusedSession = session
      }
    }

    return focusedSession

  }

  func getTerminalSession(for terminalSessionId: TerminalSessionId) -> TerminalSession? {
    guard let windowId = self.windows[terminalSessionId],
          let sessions = self.sessions[windowId],
          let session = sessions[terminalSessionId] else {
      return nil
    }

    return session
  }

  // MARK: - Setters

  fileprivate func updateTerminalSessionForWindow(_ windowId: WindowId, session: TerminalSession) {
    // updates must be threadsafe
    queue.sync { [weak self] in
      guard self != nil else { return }

      var sessionsForWindow = self!.sessions[windowId] ?? [:]

      sessionsForWindow[session.terminalSessionId] = session
      self!.sessions[windowId] = sessionsForWindow
      self!.windows[session.terminalSessionId] = windowId
    }
  }

  fileprivate func setShellContext(for terminalSessionId: TerminalSessionId, context: ShellContext) {
    guard let session = self.getTerminalSession(for: terminalSessionId) else {
      return
    }

    var updatedSession = session
    updatedSession.shellContext = context

    self.updateTerminalSessionForWindow(session.windowId, session: updatedSession)
  }

  func setEditBuffer(for sessionId: TerminalSessionId, text: String, cursor: Int) {
    guard let session = self.getTerminalSession(for: sessionId) else {
      return
    }

    var updatedSession = session
    updatedSession.editBuffer = EditBuffer(cursor: cursor, text: text)

    Logger.log(message: "SET EDITBUFFER: \(updatedSession.editBuffer?.representation ?? "none")",
               subsystem: .autocomplete)
    self.updateTerminalSessionForWindow(updatedSession.windowId, session: updatedSession)

  }
}

extension Local_ShellContext {
  var internalContext: ShellContext? {
    get {

      guard self.hasSessionID,
            self.hasPid else {
        return nil
      }

      let workingDirectory = self.hasCurrentWorkingDirectory
        ? self.currentWorkingDirectory
        : ProcessStatus.workingDirectory(for: self.pid)

      let context = ShellContext(processId: self.pid,
                                 executablePath: self.processName,
                                 ttyDescriptor: self.ttys,
                                 workingDirectory: workingDirectory,
                                 integrationVersion: Int(self.integrationVersion))

      return context
    }
  }
}

extension ShellContext {
  var ipcContext: Local_ShellContext? {
    return Local_ShellContext.with { context in
      context.pid = self.processId
      context.processName = self.executablePath
      context.ttys = self.ttyDescriptor
      context.currentWorkingDirectory = self.workingDirectory
      if let integrationVersion = self.integrationVersion {
        context.integrationVersion = Int32(integrationVersion)
      }
    }
  }

}

import FigAPIBindings
extension TerminalSessionLinker {
  func handleRequest(_ request: Fig_TerminalSessionInfoRequest) throws -> Fig_TerminalSessionInfoResponse {

    let session = self.getTerminalSession(for: request.terminalSessionID)

    return Fig_TerminalSessionInfoResponse.with { response in
      if let context = session?.shellContext?.ipcContext {
        response.context = context
      }
    }
  }
}
