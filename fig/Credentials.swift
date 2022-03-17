//
//  Credentials.swift
//  fig
//
//  Created by Matt Schrage on 3/16/22.
//  Copyright © 2022 Matt Schrage. All rights reserved.
//

import Foundation

class Credentials {
  static let shared = Credentials(fileURL:
                                  URL.dataDirectory.appendingPathComponent("credentials.json"))

  fileprivate let backing: JSONStoreProvider
  fileprivate init(filePath: String) {
    self.backing = JSONStoreProvider(backingFilePath: filePath, createIfNotExists: true)
  }
  fileprivate init(fileURL: URL) {
    self.backing = JSONStoreProvider(backingFilePath: fileURL.path, createIfNotExists: true)
  }

  func migrate() {
    guard let defaultsCredentials = Defaults.shared.credentialsToMigrate else {
      return
    }

    for (key, value) in defaultsCredentials where value != nil {
      self.backing.set(value: value, forKey: key)
    }
  }

  func getEmail() -> String? {
    return self.backing.getValue(forKey: "email") as? String
  }

  func isLoggedIn() -> Bool {
    return getEmail() != nil
  }
}
