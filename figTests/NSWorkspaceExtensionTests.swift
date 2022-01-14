//
//  NSWorkspaceExtensionTests.swift
//  figTests
//
//  Created by Federico Ciardi on 13/11/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import XCTest
import FigAPIBindings
@testable import fig

private class TestableNSWorkspace: NSWorkspace {
  private var callback: ((URL) -> Void)!

  init(_ callback: @escaping (URL) -> Void) {
    super.init()
    self.callback = callback
  }

  override func open(_ url: URL) -> Bool {
    callback(url)
    return true
  }
}

class NSWorkspaceExtensionTests: XCTestCase {
  func createOpenInExternalApplicationRequest(path: String? = nil) -> Fig_OpenInExternalApplicationRequest {
    var request = Fig_OpenInExternalApplicationRequest()
    if let path = path {
      request.url = path
    }
    return request
  }

  func testOpenInExternalApplication() throws {
    let currentHomeDir = FileManager.default.homeDirectoryForCurrentUser
    let expectedPath = currentHomeDir.appendingPathComponent(".fig").path

    let workspace = TestableNSWorkspace { url in
      XCTAssertEqual(url, URL(string: expectedPath))
    }
    let request = createOpenInExternalApplicationRequest(path: expectedPath)
    _ = try workspace.handleOpenURLRequest(request)
  }

  func testOpenInExternalApplicationMissingURL() {
    let request = createOpenInExternalApplicationRequest()
    XCTAssertThrowsError(try NSWorkspace.shared.handleOpenURLRequest(request)) { error in
      XCTAssertEqual(error as? APIError, APIError.generic(message: "Missing 'url' parameter"))
    }
  }

  func testOpenInExternalApplicationWrongURL() {
    let request = createOpenInExternalApplicationRequest(path: "some wrong url")
    XCTAssertThrowsError(try NSWorkspace.shared.handleOpenURLRequest(request)) { error in
      XCTAssertEqual(error as? APIError, APIError.generic(message: "Could not parse '\(request.url)' as a URL"))
    }
  }
}
