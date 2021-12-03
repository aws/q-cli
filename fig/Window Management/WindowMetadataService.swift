//
//  WindowMetadataService.swift
//  fig
//
//  Created by Matt Schrage on 11/29/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

protocol WindowMetadataService {
  func getMostRecentFocusId(for windowId: WindowId) -> FocusId?
  func getAssociatedShellContext(for windowId: WindowId) -> ShellContext?
  func getTerminalSessionId(for windowId: WindowId) -> TerminalSessionId?
  func getWindowHash(for windowId: WindowId) -> ExternalWindowHash
  
  @available(*, deprecated, message: "TTY should be phased out in favor of ShellContext")
  func getAssociatedTTY(for windowId: WindowId) -> TTY?
  
  @available(*, deprecated, message: "PaneId should be phased out in favor of FocusId")
  func getMostRecentPaneId(for windowId: WindowId) -> String?

}


// This should be temporary.
extension ShellHookManager: WindowMetadataService {
  func getAssociatedShellContext(for windowId: WindowId) -> ShellContext? {
    return nil
  }
  
  func getMostRecentFocusId(for windowId: WindowId) -> FocusId? {
    return self.tab(for: windowId)
  }
  
  func getAssociatedTTY(for windowId: WindowId) -> TTY? {
    return self.tty(for: self.getWindowHash(for: windowId))
  }
  
  func getTerminalSessionId(for windowId: WindowId) -> TerminalSessionId? {
    return self.getSessionId(for: self.getWindowHash(for: windowId))
  }
  
  func getWindowHash(for windowId: WindowId) -> ExternalWindowHash {
    return self.hashFor(windowId)
  }
  
  func getMostRecentPaneId(for windowId: WindowId) -> String? {
    return self.pane(for: "\(windowId)/\(self.getMostRecentFocusId(for: windowId) ?? "")")
  }
}

extension TerminalSessionLinker: WindowMetadataService {
  func getAssociatedShellContext(for windowId: WindowId) -> ShellContext? {
    guard let session = self.focusedTerminalSession(for: windowId) else {
      return nil
    }
    
    return session.shellContext
  }
  
  func getMostRecentFocusId(for windowId: WindowId) -> FocusId? {
    guard let session = self.focusedTerminalSession(for: windowId) else {
      return nil
    }
    
    return session.focusId
  }
  
  func getAssociatedTTY(for windowId: WindowId) -> TTY? {
    return nil//ShellHookManager.shared.tty(for: self.getWindowHash(for: windowId))
  }
  
  func getTerminalSessionId(for windowId: WindowId) -> TerminalSessionId? {
    return self.focusedTerminalSession(for: windowId)?.terminalSessionId
  }
  
  func getWindowHash(for windowId: WindowId) -> ExternalWindowHash {
    guard let session = self.focusedTerminalSession(for: windowId) else {
      return "\(windowId)/%"
    }
   
    return "\(session.windowId)/\(session.focusId ?? "")%"
  }
  
  func getMostRecentPaneId(for windowId: WindowId) -> String? {
    return nil //ShellHookManager.shared.pane(for: "\(windowId)/\(self.getMostRecentFocusId(for: windowId) ?? "")")
  }
}

