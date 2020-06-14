//
//  String+Shell.swift
//  fig
//
//  Created by Matt Schrage on 4/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
extension String {
    func runAsCommand(_ isVerbose: Bool = true, cwd: String? = nil, with env: Dictionary<String, String>? = nil) -> String {
        let pipe = Pipe()
        let stderr = Pipe()
        let task = Process()
        task.arguments = ["-c", String(format:"%@", self)]
        task.standardOutput = pipe
        task.standardError = stderr

        if let cwd = cwd {
            task.currentDirectoryPath = cwd
        }
        
        if let env = env {
            task.environment = env
        }
        
        if let env = env, let shell = env["SHELL"] {
            task.launchPath = shell
        } else {
            task.launchPath = "/bin/sh"
        }
        
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
        
        let errorHandler = stderr.fileHandleForReading
        errorHandler.waitForDataInBackgroundAndNotify()
        var errorObserver: NSObjectProtocol!
        errorObserver = notificationCenter.addObserver(forName: dataNotificationName, object: errorHandler, queue: nil) {  notification in
            let data = errorHandler.availableData
            guard data.count > 0 else {
                notificationCenter.removeObserver(errorObserver!)
                return
            }
            if let line = String(data: data, encoding: .utf8) {
                if isVerbose {
                    print(line)
                }
                output = output + line + "\n"
            }
            errorHandler.waitForDataInBackgroundAndNotify()
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
    
    fileprivate func addListener(_ listener: @escaping ((String) -> Void), to pipe: Pipe) {
        let handler = pipe.fileHandleForReading
        handler.waitForDataInBackgroundAndNotify()
        var observer: NSObjectProtocol!
        observer = NotificationCenter.default.addObserver(forName: .NSFileHandleDataAvailable, object: handler, queue: nil) {  notification in
                  let data = handler.availableData
                  guard data.count > 0 else {
                      NotificationCenter.default.removeObserver(observer!)
                      return
                  }
                  if let line = String(data: data, encoding: .utf8) {
                      listener(line)
                  }
                  handler.waitForDataInBackgroundAndNotify()
              }
    }
    
    func runInBackground(cwd: String? = nil, with env: Dictionary<String, String>? = nil, updateHandler: ((String, Process) -> Void)? = nil, completion: (() -> Void)? = nil) -> Process {
        
        let stdin = Pipe()
        let stderr = Pipe()
        
        let task = Process()
        task.standardOutput = stdin
        task.standardError = stderr
        
        if let cwd = cwd {
            task.currentDirectoryPath = cwd
        }
               
        if let env = env {
            task.environment = env
        }
        
        if let env = env, let shell = env["SHELL"] {
            task.launchPath = shell
        } else {
            task.launchPath = "/bin/sh"
        }
        
        if let updateHandler = updateHandler {
            addListener({ (line) in
                updateHandler(line, task)
            }, to: stdin)
            
            addListener({ (line) in
                updateHandler(line, task)
            }, to: stderr)
        }
       
        
        task.arguments = ["-c", self]
        task.launch()
        DispatchQueue.global(qos: .background).async {
            task.waitUntilExit()
            if let completion = completion {
                completion()
            }
        }
        
        return task
    }
    
}
