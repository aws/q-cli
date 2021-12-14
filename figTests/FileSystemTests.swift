//
//  FileSystemTests.swift
//  figTests
//
//  Created by Federico Ciardi on 07/11/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import XCTest
@testable import fig
import FigAPIBindings

class FileSystemTests: XCTestCase {
  var testBundle: Bundle {
    Bundle(for: Self.self)
  }

  func createReadFileRequest(path: String) -> Fig_ReadFileRequest {
    var filePath = Fig_FilePath()
    filePath.path = path
    var request = Fig_ReadFileRequest()
    request.path = filePath
    return request
  }

  func testReadFile() throws {
    let bundlePath = testBundle.path(forResource: "testable-file-to-read", ofType: "txt")
    let request = createReadFileRequest(path: bundlePath!)
    let response = try FileSystem.readFile(request)
    XCTAssertEqual(response.data, Data())
    XCTAssertEqual(response.text, "Hello!\n")
  }

  func testReadFileAsBinary() throws {
    let bundlePath = testBundle.path(forResource: "testable-file-to-read", ofType: "txt")
    var request = createReadFileRequest(path: bundlePath!)
    request.isBinaryFile = true
    let response = try FileSystem.readFile(request)
    XCTAssertEqual(response.text, "")
    XCTAssertEqual(response.data, "Hello!\n".data(using: .utf8))
  }

  func testReadMissingFile() {
    let path = testBundle.bundleURL.appendingPathComponent("some-missing-file.txt").path
    let request = createReadFileRequest(path: path)
    XCTAssertThrowsError(try FileSystem.readFile(request).text) { error in
      XCTAssertEqual(error as! APIError, APIError.generic(message: "File does not exist."))
    }
  }

  func createWriteFileRequest(path: String, content: Fig_WriteFileRequest.OneOf_Data?) -> Fig_WriteFileRequest {
    var filePath = Fig_FilePath()
    filePath.path = path
    var request = Fig_WriteFileRequest()
    request.path = filePath
    request.data = content
    return request
  }

  func testWriteTextToFile() throws {
    let fileURL = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    try "Hello!".write(to: fileURL, atomically: false, encoding: .utf8)
    XCTAssertEqual(try String(contentsOf: fileURL), "Hello!")

    let request = createWriteFileRequest(path: fileURL.path, content: .text("Updated hello!"))
    try FileSystem.writeFile(request)

    XCTAssertEqual(try String(contentsOf: fileURL), "Updated hello!")

    try FileManager.default.removeItem(at: fileURL)
  }

  func testWriteDataToFile() throws {
    let fileURL = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    try "Ciao!".write(to: fileURL, atomically: false, encoding: .utf8)
    XCTAssertEqual(try String(contentsOf: fileURL), "Ciao!")

    let request = createWriteFileRequest(path: fileURL.path, content: .binary("Updated ciao!".data(using: .utf8)!))
    try FileSystem.writeFile(request)

    XCTAssertEqual(try String(contentsOf: fileURL), "Updated ciao!")

    try FileManager.default.removeItem(at: fileURL)
  }

  func testWriteNoneToFile() throws {
    let fileURL = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    try "Hello!".write(to: fileURL, atomically: false, encoding: .utf8)
    XCTAssertEqual(try String(contentsOf: fileURL), "Hello!")

    let request = createWriteFileRequest(path: fileURL.path, content: .none)
    XCTAssertThrowsError(try FileSystem.writeFile(request)) { error in
      XCTAssertEqual(error as! APIError, APIError.generic(message: "No data to write"))
    }

    try FileManager.default.removeItem(at: fileURL)
  }

  func contentsOfDirectoryRequest(path: String) -> Fig_ContentsOfDirectoryRequest {
    var filePath = Fig_FilePath()
    filePath.path = path
    var request = Fig_ContentsOfDirectoryRequest()
    request.directory = filePath
    return request
  }

  func testContentsOfDirectory() throws {
    let path = testBundle.resourceURL?.appendingPathComponent("contents-of-this-folder").path
    let request = contentsOfDirectoryRequest(path: path!)
    let response = try FileSystem.contentsOfDirectory(request)
    XCTAssertEqual(response.fileNames.sorted(), ["file-1.txt", "file-2.json", "file-3.md"].sorted())
  }

  func createDestinationOfSymbolicLinkRequest(path: String?) -> Fig_DestinationOfSymbolicLinkRequest {
    var request = Fig_DestinationOfSymbolicLinkRequest()
    if let path = path {
      var filePath = Fig_FilePath()
      filePath.path = path
      request.path = filePath
    }
    return request
  }

  func testDestinationOfSymbolicLink() throws {
    let resourceURL = testBundle.url(forResource: "testable-file-to-read", withExtension: "txt")
    let symlinkURL = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)

    try FileManager.default.createSymbolicLink(at: symlinkURL, withDestinationURL: resourceURL!)
    let request = createDestinationOfSymbolicLinkRequest(path: symlinkURL.path)
    let response = try FileSystem.destinationOfSymbolicLink(request)
    XCTAssertEqual(response.destination.path, resourceURL!.path)
    try FileManager.default.removeItem(at: symlinkURL)
  }

  func testDestinationOfSymbolicLinkMissingPath() throws {
    let request = createDestinationOfSymbolicLinkRequest(path: nil)
    XCTAssertThrowsError(try FileSystem.destinationOfSymbolicLink(request)) { error in
      XCTAssertEqual(error as! APIError, APIError.generic(message: "Must specify a filepath"))
    }
  }

  func testDestinationOfSymbolicLinkIsNotSymbolicLink() throws {
    let path = testBundle.path(forResource: "testable-file-to-read", ofType: "txt")
    let request = createDestinationOfSymbolicLinkRequest(path: path!)
    XCTAssertThrowsError(try FileSystem.destinationOfSymbolicLink(request)) { error in
      XCTAssertEqual(error as! APIError, APIError.generic(message: "File at path is not a symbolic link"))
    }
  }

  func testDestinationOfSymbolicLinkMissingDestination() throws {
    // learn how to preserve symlinks after the build step

    //        let request = destinationOfSymbolicLinkRequest(path: url!.path)
    //        XCTAssertThrowsError(try FileSystem.destinationOfSymbolicLink(request)) { error in
    //            XCTAssertEqual(error as! APIError, APIError.generic(message: "No destination found for symbolic link"))
    //        }
  }
}
