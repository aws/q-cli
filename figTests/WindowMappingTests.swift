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
    window.windowService = self
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

extension TestableWindowServer: WindowService2 {
  
  func lastTabId(for windowId: CGWindowID) -> String {
    return self.tabs[windowId] ?? ""
  }
  
}

class TerminalSessionLinkingTests: XCTestCase {

    override func setUp() {
        // Put setup code here. This method is called before the invocation of each test method in the class.
    }

    override func tearDown() {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }

    func testExample() throws {

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

}
