//
//  Credentials.swift
//  fig
//
//  Created by Matt Schrage on 3/16/22.
//  Copyright © 2022 Matt Schrage. All rights reserved.
//

import Foundation

enum CredentialsError: Error {
    case authorizationError(String)
}

class Credentials {
  static let shared = Credentials(fileURL:
                                  URL.dataDirectory.appendingPathComponent("credentials.json"))

  fileprivate let backing: JSONStoreProvider
  fileprivate init(filePath: String) {
    self.backing = JSONStoreProvider(backingFilePath: filePath)
  }
  fileprivate init(fileURL: URL) {
    self.backing = JSONStoreProvider(backingFilePath: fileURL.path)
  }

  func migrate() {
    guard let defaultsCredentials = Defaults.shared.credentialsToMigrate else {
      return
    }

    for (key, value) in defaultsCredentials where value != nil {
      self.backing.set(value: value, forKey: key)
    }

    Defaults.shared.credentialsHaveMigrated = true
  }

  func getEmail() -> String? {
    return self.backing.getValue(forKey: "email") as? String
  }

  func isLoggedIn() -> Bool {
    return getEmail() != nil
  }

  func authorizeRequest(request: inout URLRequest) throws {
    // Try to refresh access/id token.
    let cli = Bundle.main.path(forAuxiliaryExecutable: "fig-darwin-universal")!

    Process.run(command: cli, args: ["login", "-r"])
    self.backing.reload()
    let accessToken = self.backing.getValue(forKey: "access_token") as? String
    let idToken = self.backing.getValue(forKey: "id_token") as? String

    if accessToken != nil, idToken != nil {
      if let authToken = (try? JSONSerialization.data(withJSONObject: [
        "idToken": idToken,
        "accessToken": accessToken
      ])) {
        request.setValue(
          "Bearer \(authToken.base64EncodedString())",
          forHTTPHeaderField: "Authorization"
        )
        return
      }
    }
    throw CredentialsError.authorizationError("Could not authorize request.")
  }
}
