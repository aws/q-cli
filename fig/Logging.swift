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
  
    static var defaultLocation: URL = URL(fileURLWithPath: "\(NSHomeDirectory())/.fig/logs")

    enum Subsystem: String, CaseIterable {
        case global = "global"
        case telemetry = "telemetry"
        case windowServer = "windowserver"
        case keypress = "keypress"
        case xterm = "xterm"
        case javascript = "javascript"
        case tty = "tty-link"
        case iterm = "iterm"
        case docker = "docker"
        case ssh = "ssh"
        case pty = "pty"
        case cli = "cli"
        case shellhooks = "shellhooks"
        case windowEvents = "window-events"
        case buffer = "buffer"
        case autocomplete = "autocomplete"
        case cursor = "cursor"
        case xtermCursor = "xterm-cursor"
        case settings = "settings"
        case fish = "fish"
        case tmux = "tmux"
        case unix = "unix"
        case updater = "updater"
        case config = "config"
        func pathToLogFile() -> URL {
          return Logger.defaultLocation
                 .appendingPathComponent(self.rawValue, isDirectory: false)
                 .appendingPathExtension("log")
        }
      
        func ansiColor() -> String {
          return Subsystem.colorOverridesTable[self] ?? Subsystem.colorTable[self]!
        }
      
        private static let colorOverridesTable: [Subsystem : String] =
          [ .autocomplete : "[36m"
          , .xtermCursor  : "[35;1m"
          , .windowEvents : "[46;1m"
          ]

      
        private static let colorTable: [Subsystem : String] = {
          var table: [Subsystem : String] = [:]
          for subsystem in Subsystem.allCases {
            table[subsystem] = "[38;5;\((subsystem.rawValue.djb2hash % 256))m"

          }
          
          return table
        }()
        
        static let maxLength: Int = {
          return Subsystem.allCases.max { (a, b) -> Bool in
            return a.rawValue.count > b.rawValue.count
          }?.rawValue.count ?? 15
        }()
    }
    
    static func log(message: String, priority: Priority = .info, subsystem: Subsystem = .global) {
      var line = Logger.format(message, priority, subsystem)
      
      guard Settings.canLogWithoutCrash else {
        return
      }
    
      if Settings.shared.getValue(forKey: Settings.loggingEnabledInternally) as? Bool ?? true {
        print(line)
      }
            
      guard let loggingEnabled = Settings.shared.getValue(forKey: Settings.logging) as? Bool, loggingEnabled else {
        return
      }
      
      if Settings.shared.getValue(forKey: Settings.colorfulLogging) as? Bool ?? true {
        line = Logger.format(message, priority, subsystem, colorful: true)
      }
      
      appendToLog(line, subsystem: subsystem)


    }
  
    static func resetLogs() {
      try? FileManager.default.removeItem(at: Logger.defaultLocation)
      try? FileManager.default.createDirectory(at: Logger.defaultLocation,
                                               withIntermediateDirectories: true,
                                               attributes: nil)
      // Create all log files so that they can be tailed
      // even if no events have been logged yet
      for system in Subsystem.allCases {
        FileManager.default.createFile(atPath: system.pathToLogFile().path,
                                       contents: nil,
                                       attributes: nil)
      }
    }
    
    fileprivate static func appendToLog(_ line: String, subsystem: Subsystem = .global) {
        let filepath = subsystem.pathToLogFile()
        if let file = try? FileHandle(forUpdating: filepath) {
            file.seekToEndOfFile()
    
            file.write(line.data(using: .utf8)!)
            file.closeFile()
        } else {
//            FileManager.default.createFile(atPath: filepath.absoluteString, contents: nil, attributes: nil)
            do {
                try line.write(to: filepath, atomically: true, encoding: String.Encoding.utf8)
            } catch {
              print("\(filepath.absoluteString) does not exist and could not be created. Logs will not be written.")
            }
        }
    }
    
    static func format(_ message: String,_ priority: Priority,_ subsystem: Subsystem, colorful: Bool = false) -> String {
        var prefix = "\(subsystem.rawValue): "
      
        if colorful {
          prefix = "\u{001b}\(subsystem.ansiColor())" + prefix.trimmingCharacters(in: .whitespaces) + "\u{001b}[0m "
        }
      
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


fileprivate extension String {
  var djb2hash: Int {
      let unicodeScalars = self.unicodeScalars.map { $0.value }
      return unicodeScalars.reduce(5381) {
          ($0 << 5) &+ $0 &+ Int($1)
      }
  }
  
//  var sdbmhash: Int {
//      let unicodeScalars = self.unicodeScalars.map { $0.value }
//      return unicodeScalars.reduce(0) {
//          Int($1) &+ ($0 << 6) &+ ($0 << 16) - $0
//      }
//  }
}
