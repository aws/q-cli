//
//  ScriptCommands.swift
//  figcli
//
//  Created by Matt Schrage on 8/29/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class ScriptCommand {
    static let cliFolder = NSHomeDirectory() + "/.fig/tools/cli/"
    static func matchesArguments(_ args: [String]) -> String? {

        guard args.count >= 2 else {
            return nil
        }
        
        let fileName = arguments[1].replacingOccurrences(of: ":", with: "-")
        let filePath = cliFolder + fileName + ".sh"

        guard FileManager.default.fileExists(atPath:filePath) else {
            return nil
        }
        
        return filePath
    }
}
