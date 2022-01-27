//
//  File.swift
//  fig
//
//  Created by Matt Schrage on 9/21/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import FigAPIBindings

class FileSystem {
  static func readFile(_ request: Fig_ReadFileRequest) throws -> Fig_ReadFileResponse {

    let path = request.path.normalizedPath

    guard FileManager.default.fileExists(atPath: path) else {
      throw APIError.generic(message: "File does not exist.")
    }

    let isBinaryFile = request.hasIsBinaryFile ? request.isBinaryFile : false

    return try Fig_ReadFileResponse.with {
      let url = URL(fileURLWithPath: path)
      if isBinaryFile {
        $0.data = try Data(contentsOf: url)
      } else {
        $0.text = try String(contentsOf: URL(fileURLWithPath: path))
      }
    }
  }

  @discardableResult
  static func writeFile(_ request: Fig_WriteFileRequest) throws -> Bool {
    let url = URL(fileURLWithPath: request.path.normalizedPath)

    switch request.data {
    case .text(let string):
      try string.write(to: url,
                       atomically: true,
                       encoding: .utf8)
    case .binary(let data):
      try data.write(to: url)
    case .none:
      throw APIError.generic(message: "No data to write")
    }

    return true
  }

  @discardableResult
  static func appendToFile(_ request: Fig_AppendToFileRequest) throws -> Bool {
    let path = request.path.normalizedPath
    let data: Data

    switch request.data {
    case .text(let string):
      data = string.data(using: .utf8) ?? Data()
    case .binary(let _data):
      data = _data
    case .none:
      throw APIError.generic(message: "No data to append")
    }

    if !FileManager.default.fileExists(atPath: path) {
      if !FileManager.default.createFile(atPath: path, contents: data, attributes: nil) {
        throw APIError.generic(message: "Could not create a new file")
      }
      return true
    }

    // the below will never be nil because we are eventually creating a new file above if it does not exist
    guard let fileHandle = FileHandle(forWritingAtPath: path) else { throw APIError.generic(message: "No file exists at path") }
    if #available(macOS 10.15.4, *) {
      try fileHandle.seekToEnd()
      try fileHandle.write(contentsOf: data)
      try fileHandle.close()
    } else {
      fileHandle.seekToEndOfFile()
      fileHandle.write(data)
      fileHandle.closeFile()
    }

    return true
  }

  static func contentsOfDirectory(_ request: Fig_ContentsOfDirectoryRequest) throws -> Fig_ContentsOfDirectoryResponse {

    let contents = try FileManager.default.contentsOfDirectory(atPath: request.directory.normalizedPath)

    return Fig_ContentsOfDirectoryResponse.with {
      $0.fileNames = contents
    }
  }

  static func destinationOfSymbolicLink(_ request: Fig_DestinationOfSymbolicLinkRequest) throws -> Fig_DestinationOfSymbolicLinkResponse {
    guard request.hasPath else {
      throw APIError.generic(message: "Must specify a filepath")
    }

    let fileURL = request.path.normalizedFileURL

    let wrapper = try FileWrapper(url: fileURL, options: .immediate)

    guard wrapper.isSymbolicLink else {
      throw APIError.generic(message: "File at path is not a symbolic link")
    }

    guard let destinationURL = wrapper.symbolicLinkDestinationURL,
          let destination = destinationURL.fig_filepath else {
      throw APIError.generic(message: "No destination found for symbolic link")
    }

    return Fig_DestinationOfSymbolicLinkResponse.with { response in
      response.destination = destination
    }
  }
}

extension Fig_FilePath {
  var normalizedPath: String {
    let filePath = self

    var normalizedPath = filePath.path

    if filePath.hasExpandTildeInPath,
       filePath.expandTildeInPath {

      normalizedPath = NSString(string: normalizedPath).expandingTildeInPath
    }

    if filePath.hasRelativeTo {
      normalizedPath = URL(fileURLWithPath: path, relativeTo: URL(fileURLWithPath: filePath.relativeTo)).path
    }

    return normalizedPath
  }

  var normalizedFileURL: URL {
    return URL(fileURLWithPath: self.normalizedPath)
  }
}

extension URL {
  var fig_filepath: Fig_FilePath? {

    guard self.isFileURL else { return nil }
    return Fig_FilePath.with({filepath in
      filepath.path = self.path
    })
  }
}
