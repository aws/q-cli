//
//  File.swift
//  fig
//
//  Created by Matt Schrage on 9/21/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import FigAPIBindings


class FileSystem {
    static func readFile(_ request: Fig_ReadFileRequest) throws -> Fig_ReadFileResponse {
        
        let path = request.path.normalizedPath
       
        guard FileManager.default.fileExists(atPath: path) else {
            throw APIError.generic(message: "File does not exist.")
        }
        
        let data = try Data(contentsOf: URL(fileURLWithPath: path))
        
        return Fig_ReadFileResponse.with {
            $0.data = data
        }
    }
    
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
    
    static func contentsOfDirectory(_ request: Fig_ContentsOfDirectoryRequest) throws -> Fig_ContentsOfDirectoryResponse {
        
        let contents = try FileManager.default.contentsOfDirectory(atPath: request.directory.normalizedPath)
        
        return Fig_ContentsOfDirectoryResponse.with {
            $0.fileNames = contents
        }
    }
}

extension Fig_FilePath {
    var normalizedPath: String {
        get {
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
    }
}
