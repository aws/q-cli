//
//  figTests.swift
//  figTests
//
//  Created by Matt Schrage on 4/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import XCTest
@testable import fig

class TestableApp: App {
  var bundleIdentifier: String?
  
  var localizedName: String?
  
  var processIdentifier: pid_t = 0
  
  init(bundleIdentifier: String) {
    self.bundleIdentifier = bundleIdentifier
  }

}

class TestableWindowServer {
  var windowCount = 0;
  var windows: [ExternalWindow] = []
  var tabs: [CGWindowID : String] = [:]
  
  func createNewWindow(for bundleId: String) -> CGWindowID {
    windowCount += 1
    
    let app = TestableApp(bundleIdentifier: bundleId)
    
    let window = ExternalWindow(.zero, CGWindowID(windowCount), app, nil)
    window.windowMetadataService = self
    self.windows.insert(window, at: 0)
    return window.windowId
  }
    
  func switchFocusInWindow(id: CGWindowID, focusId: String) {
    tabs[id] = focusId
  }
  
  func switchToWindow(id: CGWindowID) {
    var windowToMoveToFront: ExternalWindow? = nil
    for window in self.windows where window.windowId == id {
      let idx = self.windows.firstIndex(of: window)!
      self.windows.remove(at: idx)
      windowToMoveToFront = window
      break
    }
    
    if let window = windowToMoveToFront {
      self.windows.insert(window, at: 0)
    }
  }
  
  func switchToTab(_ focusId: String, in window: CGWindowID) {
    self.tabs[window] = focusId
  }
    
//    self.windows.re
  
  ///
  var isActivating: Bool = true
  
  var isDeactivating: Bool = true
  
}

extension TestableWindowServer: WindowService  {

  
  func topmostWhitelistedWindow() -> ExternalWindow? {
    return self.windows.first
  }
  
  func topmostWindow(for app: NSRunningApplication) -> ExternalWindow? { return nil }
  
  func previousFrontmostApplication() -> NSRunningApplication? { return nil }
  
  func currentApplicationIsWhitelisted() -> Bool { return true }
  
  func allWindows(onScreen: Bool) -> [ExternalWindow] { return [] }
  
  func allWhitelistedWindows(onScreen: Bool) -> [ExternalWindow] { return [] }
  
  func previousWhitelistedWindow() -> ExternalWindow? { return nil }
  
  func bringToFront(window: ExternalWindow) {}
  
  func takeFocus() {}
  
  func returnFocus() {}

}

extension TestableWindowServer: WindowMetadataService {
  func getMostRecentFocusId(for windowId: WindowId) -> FocusId? {
    return self.tabs[windowId]
  }
  
  func getAssociatedTTY(for windowId: WindowId) -> TTY? {
    return nil
  }
  
  func getTerminalSessionId(for windowId: WindowId) -> TerminalSessionId? {
    return nil
  }
  
  func getWindowHash(for windowId: WindowId) -> ExternalWindowHash {
    return String(windowId) + "/" + (self.tabs[windowId] ?? "") + "%"
  }
  
  func getMostRecentPaneId(for windowId: WindowId) -> String? {
    return nil
  }
}

class TerminalSessionLinkingTests: XCTestCase {

    override func setUp() {
        // Put setup code here. This method is called before the invocation of each test method in the class.
    }

    override func tearDown() {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }
  
    func testNoSessionUntilKeystroke() throws {
      let windowObserver = TestableWindowServer()
      let linker = TerminalSessionLinker(windowService: windowObserver)
      
      let a = windowObserver.createNewWindow(for: "iterm")
      XCTAssertEqual(linker.focusedTerminalSession(for: a), nil)
    }
  
    func testSessionOnceKeystroke() throws {
      let windowObserver = TestableWindowServer()
      let linker = TerminalSessionLinker(windowService: windowObserver)
      
      let a = windowObserver.createNewWindow(for: "iterm")
      try linker.linkWithFrontmostWindow(sessionId: "session-1", isFocused: true)
      XCTAssertEqual(linker.focusedTerminalSession(for: a), "session-1")

    }
  
    func testMultipleWindows() throws {

      let windowObserver = TestableWindowServer()
      let linker = TerminalSessionLinker(windowService: windowObserver)
      
      let a = windowObserver.createNewWindow(for: "iterm")
      let b = windowObserver.createNewWindow(for: "iterm")
      XCTAssertEqual(windowObserver.topmostWhitelistedWindow()?.windowId, 2)

      try linker.linkWithFrontmostWindow(sessionId: "2", isFocused: true)
      
      XCTAssertEqual(linker.focusedTerminalSession(for: b), "2")
      
      windowObserver.switchToWindow(id: a)
      
      XCTAssertEqual(linker.focusedTerminalSession(for: a), nil)
    }
  
    func testMultipleTabs() throws {

      let windowObserver = TestableWindowServer()
      let linker = TerminalSessionLinker(windowService: windowObserver)
      
      let a = windowObserver.createNewWindow(for: "iterm")

      try linker.linkWithFrontmostWindow(sessionId: "session-1", isFocused: true)
      windowObserver.switchToTab("tab-2", in: a)
      linker.resetFocusForAllSessions(in: a)
      XCTAssertEqual(linker.focusedTerminalSession(for: a), nil)
      
      try linker.linkWithFrontmostWindow(sessionId: "session-2", isFocused: true)
      XCTAssertEqual(linker.focusedTerminalSession(for: a), "session-2")

      windowObserver.switchToTab("tab-1", in: a)
      linker.resetFocusForAllSessions(in: a)
      XCTAssertEqual(linker.focusedTerminalSession(for: a), nil)
      
      try linker.linkWithFrontmostWindow(sessionId: "session-1", isFocused: true)
      XCTAssertEqual(linker.focusedTerminalSession(for: a), "session-1")
      
    }
  
  func testShellContext() throws {
    let windowObserver = TestableWindowServer()
    let linker = TerminalSessionLinker(windowService: windowObserver)
    
    let _ = windowObserver.createNewWindow(for: "iterm")
    try linker.linkWithFrontmostWindow(sessionId: "session-1", isFocused: true)
    
    let notification = Notification(name: IPC.Notifications.prompt.notification,
                                    object: Local_PromptHook.with { event in
                                        event.context = Local_ShellContext.with { context in
                                          context.pid = 0
                                          context.currentWorkingDirectory = "/usr/home"
                                          context.processName = "bash"
                                          context.ttys = "/dev/ttys001"
                                          context.sessionID = "session-1"
                                        }
                                    },
                                    userInfo: nil)
    linker.processPromptHook(notification: notification)
    let context = linker.getShellContext(for: "session-1")
    XCTAssertNotNil(context, "ShellContext should not be nil")
    XCTAssertEqual(context?.workingDirectory, "/usr/home")


    
  }

}
