//
//  Config.swift
//  fig
//
//  Created by Matt Schrage on 7/15/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class Config {
  static let shared = Config(configPath: URL(fileURLWithPath: "\(FileManager.default.homeDirectoryForCurrentUser)/.fig/user/config"))

  static let userExplictlyQuitApp = "APP_TERMINATED_BY_USER"
  static let userLoggedIn = "FIG_LOGGED_IN"
  private var userConfigPath: URL!
  
  init(configPath: URL) {
    userConfigPath = configPath
  }
  
  func set(value: String?, forKey key: String) {
    log("set '\(key)' := \(value ?? "nil")")
    updateConfig(key, value)
  }
  
  func getValue(forKey key: String) -> String? {
    let value = readConfig(forKey: key).value
    log("get '\(key)' = \(value ?? "nil")")
    return value
  }
  
  func readConfig(forKey key: String? = nil) -> (lines: [String], value: String?) {
    guard let config = try? String(contentsOf: userConfigPath, encoding: .utf8) else {
      log("could not read config file")
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
  
  func updateConfig(_ key: String, _ value: String?) {
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
      log("could not write updated config file")
    }
    
  }

}

extension Config {
  func log(_ message: String) {
    Logger.log(message: message, subsystem: .config)
  }
}

import FigAPIBindings
extension Config {
  
  func handleGetRequest(_ request: Fig_GetConfigPropertyRequest) throws -> Fig_GetConfigPropertyResponse {
    guard request.hasKey else {
      throw APIError.generic(message: "Must include key parameter")
    }
    
    guard let value = getValue(forKey: request.key) else {
      throw APIError.generic(message: "No value for key")
    }
    
    return Fig_GetConfigPropertyResponse.with { response in
      response.value = value
    }
  }
  
  @discardableResult
  func handleSetRequest(_ request: Fig_UpdateConfigPropertyRequest) throws -> Bool {
    guard request.hasKey else {
      throw APIError.generic(message: "Must include key parameter")
    }
    
    updateConfig(request.key, request.hasValue ? request.value : nil)
    
    return true
  }
  
}
