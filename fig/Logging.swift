//
//  Logging.swift
//  fig
//
//  Created by Matt Schrage on 5/27/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

class Logger {
    static var defaultLocation: URL = URL(fileURLWithPath: "\(NSHomeDirectory())/figjs.log")
    static func log(message: String) {
        if let file = try? FileHandle(forUpdating: Logger.defaultLocation) {
                file.seekToEndOfFile()
                file.write(message.data(using: .utf8)!)
                file.closeFile()
            } else {
                print("figjs.log does not exist. JS console logs will not be written.")
            }
        
    }
}


