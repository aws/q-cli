//
//  Settings.swift
//  figcli
//
//  Created by Matt Schrage on 3/16/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Settings {
  static let filePath = NSHomeDirectory() + "/.fig/settings.json"
  static func loadFromFile() ->  [String: Any]? {
    guard FileManager.default.fileExists(atPath: Settings.filePath) else {
      print("Settings: settings file does not exist")
      return nil
    }
    
    guard let settings = try? String(contentsOfFile: Settings.filePath) else {
      print("Settings: settings file is empty")
      return nil
    }
    
    guard settings.count > 0 else {
      return nil
    }
    
    guard let json = settings.jsonStringToDict() else {
      return nil
    }
    
    return json
  }
  
  static func serialize(settings: [String: Any]) {
      do {
        let data = try JSONSerialization.data(withJSONObject: settings, options: [.sortedKeys, .prettyPrinted])
        try data.write(to: URL(fileURLWithPath: Settings.filePath), options: .atomic)
      } catch {
        print("Settings: failed to serialize data")
      }
    }
}

extension String {
    func jsonStringToDict() -> [String: Any]? {
        if let data = self.data(using: .utf8) {
            do {
                return try JSONSerialization.jsonObject(with: data, options: []) as? [String: Any]
            } catch {
                
            }
        }
        return nil
    }
  
}



