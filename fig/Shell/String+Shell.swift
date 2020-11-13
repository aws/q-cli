//
//  String+Shell.swift
//  fig
//
//  Created by Matt Schrage on 4/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

extension NSAppleScript {
    static func run(path: String) {
        let task = Process()
        task.launchPath = "/usr/bin/osascript"
        task.arguments = [path]
         
        task.launch()
    }
}

extension String {
    func runWithElevatedPrivileges() -> String? {
        let myAppleScript = "do shell script \"\(self)\" with administrator privileges"
        var error: NSDictionary?
        guard let scriptObject = NSAppleScript(source: myAppleScript) else { return nil }
    
        let output: NSAppleEventDescriptor = scriptObject.executeAndReturnError(&error)
        if (error != nil) {
            print("error: \(error ?? [:])")
            return nil
        }
        
        return output.stringValue
        
    }
    
    func runWithElevatedPriviledgesFromAppleScript(completion: (()-> Void)? = nil) {
        "cmd=\"do shell script \\\"\(self)\\\" with administrator privileges\" && osascript -e \"$cmd\"".runInBackground(completion: { (out) in
            if let completion = completion {
                completion()
            }
        })

    }
    
    func runAsCommand(_ isVerbose: Bool = false, cwd: String? = nil, with env: Dictionary<String, String>? = nil) -> String {
        
        
        let pipe = Pipe()
        let stderr = Pipe()
        let task = Process()
        //add "-li" to get closer to terminal behavior
        task.arguments = ["-c", String(format:"%@", self)]
        task.standardOutput = pipe
        task.standardError = stderr

//        if let cwd = cwd {
//            task.currentDirectoryPath = cwd
//        }
        task.currentDirectoryPath = cwd ?? NSHomeDirectory()

        
        if let env = env {
            task.environment = env
        }
        
        task.environment?["HOME"] = NSHomeDirectory()

        
        if let env = env, let shell = env["SHELL"] {
            task.launchPath = shell
        } else {
            task.launchPath = "/bin/sh"
        }
        
//        let data = pipe.fileHandleForReading.readDataToEndOfFile()
//        let output = String(data: data, encoding: String.Encoding.utf8) ?? ""
        
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
                outputHandler.closeFile()
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
                errorHandler.closeFile()
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
//        task.terminationHandler = { (process) in
//            notificationCenter.removeObserver(dataObserver!)
//            notificationCenter.removeObserver(errorObserver!)
//            outputHandler.closeFile()
//            errorHandler.closeFile()
//        }
            
        task.launch()
        task.waitUntilExit()
//
//        print("TerminationStatus:", task.terminationStatus)
//        print("TerminationReason:", task.terminationReason)

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
                      handler.closeFile()
                      return
                  }
                  if let line = String(data: data, encoding: .utf8) {
                      listener(line)
                  }
                  handler.waitForDataInBackgroundAndNotify()
              }
    }
    
    func runInBackground(cwd: String? = nil, with env: Dictionary<String, String>? = nil, updateHandler: ((String, Process) -> Void)? = nil, completion: ((String) -> Void)? = nil) -> Process {
        
        let stdin = Pipe()
        let stderr = Pipe()
        
        let task = Process()
        task.standardOutput = stdin
        task.standardError = stderr
        
        task.currentDirectoryPath = cwd ?? NSHomeDirectory()

               
        if let env = env {
            task.environment = env
        }
        
        task.environment?["HOME"] = NSHomeDirectory()
        
        if let env = env, let shell = env["SHELL"] {
            task.launchPath = shell
        } else {
            task.launchPath = "/bin/sh"
        }
        var out: String = ""
        if let updateHandler = updateHandler {
            addListener({ (line) in
                updateHandler(line, task)
                out += line
            }, to: stdin)
            
            addListener({ (line) in
                updateHandler(line, task)
                out += line
            }, to: stderr)
        }
       
        //add "-li" to get closer to terminal behavior
        task.arguments = ["-c", self]
        task.launch()
        DispatchQueue.global(qos: .background).async {
            task.waitUntilExit()
            if let completion = completion {
                completion(out)
            }
        }
        
        return task
    }
    
}
