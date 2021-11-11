//
//  ConfigTests.swift
//  figTests
//
//  Created by Federico Ciardi on 11/11/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import XCTest
@testable import fig
import FigAPIBindings

enum ConfigTestsError: Error {
    case configFileCreation
}

class ConfigTests: XCTestCase {
    let defaultConfigContent = """
    SOME_KEY=1
    """
    let configPath = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    lazy var config: Config = {
       return Config(configPath: configPath)
    }()
    
    
    override func setUpWithError() throws {
        try super.setUpWithError()
        guard FileManager.default.createFile(atPath: configPath.path, contents: defaultConfigContent.data(using: .utf8), attributes: nil) else { throw ConfigTestsError.configFileCreation }
    }

    func createGetRequest(key: String? = nil) -> Fig_GetConfigPropertyRequest {
        var request = Fig_GetConfigPropertyRequest()
        if let key = key {
            request.key = key
        }
        return request
    }
    
    func testGetRequest() throws {
        // write file at path
        let request = createGetRequest(key: "SOME_KEY")
        let response = try config.handleGetRequest(request)
        XCTAssertEqual(response.value, "1")
    }
    
    func testGetRequestMissingKey() {
        let request = createGetRequest()
        XCTAssertThrowsError(try config.handleGetRequest(request)) { error in
            XCTAssertEqual(error as! APIError, APIError.generic(message: "Must include key parameter"))
        }
    }
    
    func testGetRequestMissingValue() {
        let request = createGetRequest(key: "MISSING_KEY")
        XCTAssertThrowsError(try config.handleGetRequest(request)) { error in
            XCTAssertEqual(error as! APIError, APIError.generic(message: "No value for key"))
        }
    }
    
    func createSetRequest(key: String?, value: String?) -> Fig_UpdateConfigPropertyRequest {
        var request = Fig_UpdateConfigPropertyRequest()
        if let key = key {
            request.key = key
        }
        if let value = value {
            request.value = value
        }
        return request
    }
    
    func testSetRequestCreateKey() throws {
        let key = "KEY_TO_ADD"
        // check that the key we want to update is missing
        let getRequest = createGetRequest(key: key)
        XCTAssertThrowsError(try config.handleGetRequest(getRequest)) { error in
            XCTAssertEqual(error as! APIError, APIError.generic(message: "No value for key"))
        }

        let setRequest = createSetRequest(key: key, value: "value")
        try config.handleSetRequest(setRequest)
        
        let getResponse = try config.handleGetRequest(getRequest)
        XCTAssertEqual(getResponse.value, "value")
    }
    
    func testSetRequestUpdateKey() throws {
        let key = "SOME_KEY"
        // check that the key we want to update contains the old value
        let getRequest = createGetRequest(key: key)
        var getResponse = try config.handleGetRequest(getRequest)
        XCTAssertEqual(getResponse.value, "1")

        let setRequest = createSetRequest(key: key, value: "2")
        try config.handleSetRequest(setRequest)
        
        getResponse = try config.handleGetRequest(getRequest)
        XCTAssertEqual(getResponse.value, "2")
    }
    
    func testSetRequestMissingKey() {
        let request = createSetRequest(key: nil, value: nil)
        XCTAssertThrowsError(try config.handleSetRequest(request)) { error in
            XCTAssertEqual(error as! APIError, APIError.generic(message: "Must include key parameter"))
        }
    }
}
