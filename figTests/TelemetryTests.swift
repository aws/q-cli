//
//  TelemetryTests.swift
//  figTests
//
//  Created by Federico Ciardi on 17/11/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import XCTest
@testable import fig
import FigAPIBindings

private class TestableTelementryProvider: TelemetryProvider {
    private var callback: ((String, [String: String]) -> Void)!

    init(_ defaults: Defaults, _ callback: @escaping (String, [String: String]) -> Void) {
        super.init(defaults: defaults)
        self.callback = callback
    }

    override func upload(
        to endpoint: String,
        with body: [String: String],
        completion: ((Data?, URLResponse?, Error?) -> Void)? = nil
    ) {
        callback(endpoint, body)
    }
}

class TelemetryTests: XCTestCase {
    let defaults = Defaults(UserDefaults(suiteName: "testSuite")!)

    func createProperty(_ key: String, _ value: String) -> Fig_TelemetryProperty {
        var property = Fig_TelemetryProperty()
        property.key = key
        property.value = value
        return property
    }

    func createAliasRequest(userId: String? = nil) -> Fig_TelemetryAliasRequest {
        var request = Fig_TelemetryAliasRequest()
        if let userId = userId {
            request.userID = userId
        }
        return request
    }

    func testAliasRequest() throws {
        defaults.telemetryDisabled = false
        let request = createAliasRequest(userId: "bar")
        var updateCalls = 0
        let provider = TestableTelementryProvider(defaults) { endpoint, body in
            updateCalls += 1
            XCTAssertEqual(endpoint, "alias")
            XCTAssertEqual(body, ["previousId": self.defaults.uuid, "userId": "bar"])
        }
        try provider.handleAliasRequest(request)
        XCTAssertEqual(updateCalls, 1)
    }

    func testAliasRequestMissingUserId() {
        let request = createAliasRequest()
        XCTAssertThrowsError(try TelemetryProvider.shared.handleAliasRequest(request)) { error in
            XCTAssertEqual(error as? APIError, APIError.generic(message: "No user id specified."))
        }
    }

    func testAliasRequestTelemetryDisabled() throws {
        defaults.telemetryDisabled = true
        let request = createAliasRequest(userId: "foo")
        var updateCalls = 0
        let provider = TestableTelementryProvider(defaults) { _, _ in
            updateCalls += 1
        }
        try provider.handleAliasRequest(request)
        XCTAssertEqual(updateCalls, 0)
    }

    func createTrackRequest(
        event: String? = nil,
        properties: [Fig_TelemetryProperty] = []
    ) -> Fig_TelemetryTrackRequest {
        var request = Fig_TelemetryTrackRequest()
        if let event = event {
            request.event = event
        }
        request.properties = properties
        return request
    }

    func testTrackRequest() throws {
        defaults.telemetryDisabled = false
        let request = createTrackRequest(event: "some-event", properties: [createProperty("foo", "bar")])
        var updateCalls = 0
        let provider = TestableTelementryProvider(defaults) { endpoint, body in
            updateCalls += 1
            XCTAssertEqual(endpoint, "track")
            XCTAssertEqual(body["userId"], self.defaults.uuid)
            XCTAssertEqual(body["event"], "some-event")
            XCTAssertEqual(body["prop_foo"], "bar")
        }
        try provider.handleTrackRequest(request)
        XCTAssertEqual(updateCalls, 1)
    }

    func testTrackRequestMissingEvent() {
        let request = createTrackRequest()
        XCTAssertThrowsError(try TelemetryProvider.shared.handleTrackRequest(request)) { error in
            XCTAssertEqual(error as? APIError, APIError.generic(message: "No event specified."))
        }
    }

    func testTrackRequestTelemetryDisabled() throws {
        defaults.telemetryDisabled = true
        let request = createTrackRequest(event: "some-event")
        var updateCalls = 0
        let provider = TestableTelementryProvider(defaults) { _, _ in
            updateCalls += 1
        }
        try provider.handleTrackRequest(request)
        XCTAssertEqual(updateCalls, 0)
    }

    func testTrackRequestTelemetryJustEnabled() throws {
        defaults.telemetryDisabled = true
        let request = createTrackRequest(event: TelemetryEvent.telemetryToggled.rawValue)
        var updateCalls = 0
        let provider = TestableTelementryProvider(defaults) { endpoint, body in
            updateCalls += 1
            XCTAssertEqual(endpoint, "track")
            XCTAssertEqual(body["userId"], self.defaults.uuid)
            XCTAssertEqual(body["event"], TelemetryEvent.telemetryToggled.rawValue)
        }
        try provider.handleTrackRequest(request)
        XCTAssertEqual(updateCalls, 1)
    }

    func createIdentifyRequest(traits: [Fig_TelemetryProperty] = []) -> Fig_TelemetryIdentifyRequest {
        var request = Fig_TelemetryIdentifyRequest()
        request.traits = traits
        return request
    }

    func testIdentifyRequest() throws {
        defaults.telemetryDisabled = false
        let request = createIdentifyRequest(traits: [createProperty("foo", "bar")])
        var updateCalls = 0
        let provider = TestableTelementryProvider(defaults) { endpoint, body in
            updateCalls += 1
            XCTAssertEqual(endpoint, "identify")
            XCTAssertEqual(body["userId"], self.defaults.uuid)
            XCTAssertEqual(body["trait_foo"], "bar")
        }
        try provider.handleIdentifyRequest(request)
        XCTAssertEqual(updateCalls, 1)
    }

    func testIdentifyRequestTelemetryDisabled() throws {
        defaults.telemetryDisabled = true
        let request = createIdentifyRequest(traits: [createProperty("foo", "bar")])
        var updateCalls = 0
        let provider = TestableTelementryProvider(defaults) { _, _ in
            updateCalls += 1
        }
        try provider.handleIdentifyRequest(request)
        XCTAssertEqual(updateCalls, 0)
    }

}
