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
  
  init(windowService: WindowService) {
    self.windowService = windowService

  }
  
  func processEditbufferChangedHook(event: Local_EditBufferHook) throws {
    try self.linkWithFrontmostWindow(sessionId: event.context.hasSessionID ? event.context.sessionID : nil,
                                     isFocused: true)
  }

  func linkWithFrontmostWindow(sessionId: TerminalSessionId?, isFocused: Bool?) throws {

    guard let sessionId = sessionId else {
      throw LinkingError.noTerminalSessionId
    }
    guard let window = windowService.topmostWhitelistedWindow(), let bundleId = window.bundleId else {
      throw LinkingError.noWindowCandidateAvailable
    }
    
    // todo: handle focus id in WindowService
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

