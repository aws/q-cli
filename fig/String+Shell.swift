//
//  String+Shell.swift
//  fig
//
//  Created by Matt Schrage on 4/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
extension String {
    func runAsCommand(_ isVerbose: Bool = true) -> String {
        let pipe = Pipe()
        let task = Process()
        task.launchPath = "/bin/sh"
        task.arguments = ["-c", String(format:"%@", self)]
        task.standardOutput = pipe
        let outputHandler = pipe.fileHandleForReading
        outputHandler.waitForDataInBackgroundAndNotify()
        
        var output = ""
        var dataObserver: NSObjectProtocol!
        let notificationCenter = NotificationCenter.default
        let dataNotificationName = NSNotification.Name.NSFileHandleDataAvailable
        dataObserver = notificationCenter.addObserver(forName: dataNotificationName, object: outputHandler, queue: nil) {  notification in
            let data = outputHandler.availableData
            guard data.count > 0 else {
                notificationCenter.removeObserver(dataObserver!)
                return
            }
            if let line = String(data: data, encoding: .utf8) {
                if isVerbose {
                    print(line)
                }
                output = output + line + "\n"
            }
            outputHandler.waitForDataInBackgroundAndNotify()
        }
        
        task.launch()
        task.waitUntilExit()
        
        return output
//        if let result = NSString(data: file.readDataToEndOfFile(), encoding: String.Encoding.utf8.rawValue) {
//            return result as String
//        }
//        else {
//            return "--- Error running command - Unable to initialize string from file data ---"
//        }
    }
    
}
