//
//  DefaultsTest.swift
//  figTests
//
//  Created by Federico Ciardi on 14/11/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import XCTest
@testable import fig
import FigAPIBindings

class DefaultsTest: XCTestCase {
  let key = "some-key"
  let userDefaults = UserDefaults(suiteName: "testSuite")!
  lazy var defaults: Defaults = {
    return Defaults(userDefaults)
  }()

  func createGetDefaultsPropertyRequest(key: String? = nil) -> Fig_GetDefaultsPropertyRequest {
    var request = Fig_GetDefaultsPropertyRequest()
    if let key = key {
      request.key = key
    }

    return request
  }

  func testGetDefaultsPropertyNil() throws {
    userDefaults.set(nil, forKey: key)
    let request = createGetDefaultsPropertyRequest(key: key)
    let response = try defaults.handleGetRequest(request)
    XCTAssertEqual(response.value.type, .null(true))
  }

  func testGetDefaultsPropertyInt() throws {
    userDefaults.set(100, forKey: key)
    let request = createGetDefaultsPropertyRequest(key: key)
    let response = try defaults.handleGetRequest(request)
    XCTAssertEqual(response.value.type, .integer(100))
  }

  func testGetDefaultsPropertyIntTruthy() throws {
    userDefaults.set(1, forKey: key)
    let request = createGetDefaultsPropertyRequest(key: key)
    let response = try defaults.handleGetRequest(request)
    XCTAssertEqual(response.value.type, .integer(1))
  }

  func testGetDefaultsPropertyIntFalsy() throws {
    userDefaults.set(0, forKey: key)
    let request = createGetDefaultsPropertyRequest(key: key)
    let response = try defaults.handleGetRequest(request)
    XCTAssertEqual(response.value.type, .integer(0))
  }

  func testGetDefaultsPropertyTrue() throws {
    userDefaults.set(true, forKey: key)
    let request = createGetDefaultsPropertyRequest(key: key)
    let response = try defaults.handleGetRequest(request)
    XCTAssertEqual(response.value.type, .boolean(true))
  }

  func testGetDefaultsPropertyFalse() throws {
    userDefaults.set(false, forKey: key)
    let request = createGetDefaultsPropertyRequest(key: key)
    let response = try defaults.handleGetRequest(request)
    XCTAssertEqual(response.value.type, .boolean(false))
  }

  func testGetDefaultsPropertyStringValue() throws {
    userDefaults.set("some-value", forKey: key)
    let request = createGetDefaultsPropertyRequest(key: key)
    let response = try defaults.handleGetRequest(request)
    XCTAssertEqual(response.value.type, .string("some-value"))
  }

  func testGetDefaultsPropertyWrongType() {
    userDefaults.set(Date(), forKey: key)
    let request = createGetDefaultsPropertyRequest(key: key)
    XCTAssertThrowsError(try defaults.handleGetRequest(request)) { error in
      XCTAssertEqual(error as! APIError, APIError.generic(message: "Value is an unsupport type."))
    }
  }

  func testGetDefaultsPropertyMissingKey() {
    let request = createGetDefaultsPropertyRequest()
    XCTAssertThrowsError(try Defaults.shared.handleGetRequest(request)) { error in
      XCTAssertEqual(error as! APIError, APIError.generic(message: "No key provided."))
    }
  }

  func createSetDefaultsPropertyRequest(
    key: String?,
    type: Fig_DefaultsValue.OneOf_Type?
  ) -> Fig_UpdateDefaultsPropertyRequest {
    var request = Fig_UpdateDefaultsPropertyRequest()
    if let key = key {
      request.key = key
    }
    if let type = type {
      var value = Fig_DefaultsValue()
      value.type = type
      request.value = value
    }

    return request
  }

  func testSetDefaultsPropertyNull() throws {
    userDefaults.set("some-value", forKey: key)
    XCTAssertEqual(userDefaults.string(forKey: key), "some-value")
    let request = createSetDefaultsPropertyRequest(key: key, type: .null(true))
    try defaults.handleSetRequest(request)
    XCTAssertEqual(userDefaults.string(forKey: key), nil)
  }

  func testSetDefaultsPropertyNil() throws {
    userDefaults.set("some-value", forKey: key)
    XCTAssertEqual(userDefaults.string(forKey: key), "some-value")
    let request = createSetDefaultsPropertyRequest(key: key, type: nil)
    try defaults.handleSetRequest(request)
    XCTAssertEqual(userDefaults.string(forKey: key), nil)
  }

  func testSetDefaultsPropertyBool() throws {
    userDefaults.set(true, forKey: key)
    XCTAssertTrue(userDefaults.bool(forKey: key))
    let request = createSetDefaultsPropertyRequest(key: key, type: .boolean(false))
    try defaults.handleSetRequest(request)
    XCTAssertFalse(userDefaults.bool(forKey: key))
  }

  func testSetDefaultsPropertyInt() throws {
    userDefaults.set(100, forKey: key)
    XCTAssertEqual(userDefaults.integer(forKey: key), 100)
    let request = createSetDefaultsPropertyRequest(key: key, type: .integer(200))
    try defaults.handleSetRequest(request)
    XCTAssertEqual(userDefaults.integer(forKey: key), 200)
  }

  func testSetDefaultsPropertyString() throws {
    userDefaults.set("some-value", forKey: key)
    XCTAssertEqual(userDefaults.string(forKey: key), "some-value")
    let request = createSetDefaultsPropertyRequest(key: key, type: .string("some-updated-value"))
    try defaults.handleSetRequest(request)
    XCTAssertEqual(userDefaults.string(forKey: key), "some-updated-value")
  }

  func testSetDefaultsPropertyWrongType() {
    let request = createSetDefaultsPropertyRequest(key: key, type: .null(false))
    XCTAssertThrowsError(try defaults.handleSetRequest(request)) { error in
      XCTAssertEqual(error as! APIError, APIError.generic(message: "Value is an unsupport type."))
    }
  }

  func testSetDefaultsPropertyMissingKey() {
    let request = createSetDefaultsPropertyRequest(key: nil, type: nil)
    XCTAssertThrowsError(try Defaults.shared.handleSetRequest(request)) { error in
      XCTAssertEqual(error as! APIError, APIError.generic(message: "No key provided."))
    }
  }
}
