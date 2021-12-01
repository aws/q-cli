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
  func focusedTerminalSession(for windowId: WindowId) -> TerminalSessionId?
  func associatedWindowId(for terminalSession: TerminalSessionId) -> WindowId?

}

typealias WindowId = CGWindowID
typealias TerminalSessionId = String
typealias FocusId = String

fileprivate struct TerminalSession {
  let windowId: WindowId
  let bundleId: String
  let terminalSessionId: TerminalSessionId


  let focusId: FocusId?
  var isFocused: Bool = false
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
  fileprivate var sessions: [WindowId : [ TerminalSessionId : TerminalSession ]] = [:]
    
  func focusedTerminalSession(for windowId: WindowId) -> TerminalSessionId? {
    guard let sessions = self.sessions[windowId]?.values else { return nil }
    
    var focusedSessionId: TerminalSessionId? = nil
    for session in sessions {
      if session.isFocused {
        assert(focusedSessionId == nil, "There should only be one focused session per window.")
        focusedSessionId = session.terminalSessionId
      }
    }
    
    return focusedSessionId

  }
  
  fileprivate func getTerminalSession(for terminalSessionId: TerminalSessionId) -> TerminalSession? {
    for sessions in self.sessions.values {
      if let targetSession = sessions[terminalSessionId] {
        return targetSession
      }
    }
    
    return nil
  }
  
  func associatedWindowId(for terminalSessionId: TerminalSessionId) -> WindowId? {
    guard let session = self.getTerminalSession(for: terminalSessionId) else {
      return nil
    }
    
    return session.windowId
  }
  
  func associatedWindowHash(for terminalSessionId: TerminalSessionId) -> ExternalWindowHash? {
    guard let session = self.getTerminalSession(for: terminalSessionId) else {
      return nil
    }
    
    return "\(session.windowId)/\(session.focusId ?? "")%"
  }
  
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

  }
  
  deinit {
    NotificationCenter.default.removeObserver(self)
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
  
  @objc func processEditbufferHook(notification: Notification) {
    guard let event = notification.object as? Local_EditBufferHook else {
      return
    }
  
    do {
      try self.linkWithFrontmostWindow(sessionId: event.context.hasSessionID ? event.context.sessionID : nil,
                                       isFocused: true)
    } catch {
      print(error)
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

    updateTerminalSessionForWindow(windowId, session: terminalSession)
  }
  
  fileprivate func updateTerminalSessionForWindow(_ windowId: WindowId, session: TerminalSession) {
    // updates must be threadsafe
    queue.sync { [weak self] in
        guard self != nil else { return }
      
        var sessionsForWindow = self!.sessions[windowId] ?? [:]
      
        // reset focus on all other sessions
        sessionsForWindow = sessionsForWindow.mapValues { session -> TerminalSession in
          var updatedSession = session
          updatedSession.isFocused = false
          return updatedSession
        }
      
        sessionsForWindow[session.terminalSessionId] = session
        self!.sessions[windowId] = sessionsForWindow
    }
  }
}

