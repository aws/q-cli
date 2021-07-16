//
//  Config.swift
//  fig
//
//  Created by Matt Schrage on 7/15/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class Config {
  static let userExplictlyQuitApp = "APP_TERMINATED_BY_USER"
  static let userLoggedIn = "FIG_LOGGED_IN"
  fileprivate static let userConfigPath: URL = URL(fileURLWithPath: "\(NSHomeDirectory())/.fig/user/config")
  
  static func set(value: String?, forKey key: String) {
    Config.log("set '\(key)' := \(value ?? "nil")")
    updateConfig(key, value)
  }
  
  static func getValue(forKey key: String) -> String? {
    let value = readConfig(forKey: key).value
    Config.log("get '\(key)' = \(value ?? "nil")")
    return value
  }
  
  static func readConfig(forKey key: String? = nil) -> (lines: [String], value: String?) {
    guard let config = try? String(contentsOf: userConfigPath, encoding: .utf8) else {
      Config.log("could not read config file")
      return ([], nil)
    }
    
    var val: String? = nil
    let lines = config.split(separator: "\n").map{ String($0) }.filter { (line) -> Bool in
      let tokens = line.trimmingCharacters(in: .whitespaces).split(separator: "=")
      
      guard tokens.count == 2 else {
        // ignore nonconforming lines
        return true
      }
      
      let (k, v) = (String(tokens.first!), String(tokens.last!))
      
      if key == k {
        val = v
        return false
      }
      
      // Keep all keys except target
      return true
      
    }
    
    return (lines: lines, value: val)
    
  }
  
  static func updateConfig(_ key: String, _ value: String?) {
    var lines = readConfig(forKey: key).lines
    
    if let value = value {
      lines.append("\(key)=\(value)")
    }
    
    let newConfig = lines.joined(separator: "\n")
    
    do {
      try newConfig.write(to: userConfigPath,
                      atomically: true,
                      encoding: .utf8)
    } catch {
      Config.log("could not write updated config file")
    }
    
  }

}

extension Config {
  static func log(_ message: String) {
    Logger.log(message: message, subsystem: .config)
  }
}
