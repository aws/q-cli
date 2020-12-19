//
//  Logging.swift
//  fig
//
//  Created by Matt Schrage on 5/27/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

class Logger {
    enum Priority {
        case info
        case notify
    }
    
    enum Subsystem: String {
        case global = "global"
        case telemetry = "telemetry"
        case windowServer = "windowserver"
        case keypress = "keypress"
        case xterm = "xterm"
        case javascript = "javascript"
        case tty = "tty"

    }
    
    static var defaultLocation: URL = URL(fileURLWithPath: "\(NSHomeDirectory())/.fig/debug.log")
    static func log(message: String, priority: Priority = .info, subsystem: Subsystem = .global) {
        print(message)
        
        let line = Logger.format(message, priority, subsystem)
        appendToLog(line)
        
        //|| Defaults.broadcastLogsForSubsystem == subsystem
        if Defaults.broadcastLogs {
            let notification = NSUserNotification()
            notification.title = "Logging: \(subsystem.rawValue)"
            notification.subtitle = message
            NSUserNotificationCenter.default.deliver(notification)
        } 

    }
    
    fileprivate static func appendToLog(_ line: String) {
        if let file = try? FileHandle(forUpdating: Logger.defaultLocation) {
            file.seekToEndOfFile()
    
            file.write(line.data(using: .utf8)!)
            file.closeFile()
        } else {
            do {
                try line.write(to: defaultLocation, atomically: true, encoding: String.Encoding.utf8)
            } catch {
                print("debug.log does not exist and could not be created. Logs will not be written.")
            }
        }
    }
    
    static func format(_ message: String,_ priority: Priority,_ subsystem: Subsystem) ->String {
        let prefix = "\(subsystem.rawValue) | \(Logger.now): "
        return message.split(separator: "\n").map { return prefix + $0 }.joined(separator: "\n") + "\n"
    }
    
    static var now: String {
            let now = Date()

           let formatter = DateFormatter()
           formatter.timeZone = TimeZone.current
           formatter.dateFormat = "yyyy-MM-dd HH:mm"
        
           return formatter.string(from: now)
    }
}


import os.log

extension OSLog {
    private static var subsystem = Bundle.main.bundleIdentifier!

    /// Logs the view cycles like viewDidLoad.
    static let socketServer = OSLog(subsystem: subsystem, category: "socketServer")
}
