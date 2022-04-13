//
//  LocalState.swift
//  fig
//
//  Created by Matt Schrage on 2/28/22.
//  Copyright © 2022 Matt Schrage. All rights reserved.
//

import Foundation

protocol JSONStore {
  func set(value: Any?, forKey key: String)
  func getValue(forKey key: String) -> Any?
  func jsonRepresentation() -> String?
}

protocol JSONStoreDelegate: AnyObject {
  func storeDidSet(key: String, value: Any?)
  func storeDidReload(previousStore: [String: Any])
  func defaultFor(key: String) -> Any?
}

class JSONStoreProvider: JSONStore {
  let backingFile: URL
  weak var delegate: JSONStoreDelegate?
  fileprivate var raw: [String: Any] = [:]

  init(backingFilePath: String) {
    self.backingFile = URL(fileURLWithPath: backingFilePath)
    self.raw = self.load()
    if !FileManager.default.fileExists(atPath: backingFilePath) {
      self.serialize()
    }
  }

  func set(value: Any?, forKey key: String) {
    if let value = value {
      self.raw[key] = value
    } else {
      self.raw.removeValue(forKey: key)
    }

    self.serialize()
  }

  func getValue(forKey key: String) -> Any? {
    return raw[key] ?? self.delegate?.defaultFor(key: key)
  }

  func keys() -> [String] {
    return Array(raw.keys)
  }

  func reload() {
    let previous = self.raw
    self.raw = load()
    self.delegate?.storeDidReload(previousStore: previous)
  }

  fileprivate func serialize() {
    do {
      let data = try JSONSerialization.data(withJSONObject: raw, options: [.prettyPrinted, .sortedKeys])
      try data.write(to: self.backingFile, options: .atomic)
    } catch {
      Logger.log(message: "failed to serialize data", priority: .trace)
    }
  }

  fileprivate func load() -> [String: Any] {
    let path = self.backingFile.path
    guard FileManager.default.fileExists(atPath: path) else {
      Logger.log(message: "file \(path) does not exist", priority: .trace)
      return [:]
    }

    guard let json = try? String(contentsOfFile: path), json.count > 0 else {
      Logger.log(message: "file \(path) is empty", priority: .trace)
      return [:]
    }

    return json.parseAsJSON() ?? [:]

  }

  func jsonRepresentation() -> String? {
    guard let data = try? JSONSerialization.data(
      withJSONObject: self.raw,
      options: .prettyPrinted
    ) else {
      return nil
    }

    return String(decoding: data, as: UTF8.self)

  }

}

class LocalState: JSONStore {
  static let localStateUpdatedNotification = Notification.Name("localStateUpdated")

  // Note: app will crash if anything is logged before LocalState.shared is initted
  static var canLogWithoutCrash = false

  static let shared = LocalState(fileURL:
                                  URL.dataDirectory.appendingPathComponent("state.json"))

  fileprivate let backing: JSONStoreProvider
  init(filePath: String) {
    self.backing = JSONStoreProvider(backingFilePath: filePath)
  }
  init(fileURL: URL) {
    self.backing = JSONStoreProvider(backingFilePath: fileURL.path)
    LocalState.canLogWithoutCrash = true
  }
  func set(value: Any?, forKey key: String) {
    self.backing.set(value: value, forKey: key)
  }

  func getValue(forKey key: String) -> Any? {
    return self.backing.getValue(forKey: key)
  }

  func addIfNotPresent(key: String, value: Any?) {
    guard getValue(forKey: key) == nil else { return }
    set(value: value, forKey: key)
  }

  func jsonRepresentation() -> String? {
    return self.backing.jsonRepresentation()
  }

  func localStateUpdated() {
    self.backing.reload()
    NotificationCenter.default.post(Notification(name: LocalState.localStateUpdatedNotification))

  }

}

extension LocalState {
  static let ptyPathKey = "pty.path"
  static let userShell = "userShell"
  static let hasSeenOnboarding = "user.onboarding"

  static let userExplictlyQuitApp = "APP_TERMINATED_BY_USER"
  static let userLoggedIn = "FIG_LOGGED_IN"

  static let logging = "developer.logging"
  static let loggingEnabledInternally = "developer.logging.internal"
  static let colorfulLogging = "developer.logging.color"

  static let inputMethodInstalled = "input-method.enabled"
  static let showIconInDock = "mission-control.showIconInDock"
}

import FigAPIBindings
extension LocalState {
  func handleGetRequest(_ request: Fig_GetLocalStateRequest) throws -> Fig_GetLocalStateResponse {
    let value: Any = try {
      if request.hasKey {
        if let value = Settings.shared.getValue(forKey: request.key) {
          return value
        } else {
          throw APIError.generic(message: "No value for key")
        }
      } else {
        return self.backing.raw
      }

    }()

    guard let data = try? JSONSerialization.data(withJSONObject: value,
                                                 options: [ .prettyPrinted,
                                                            .fragmentsAllowed]) else {
      throw APIError.generic(message: "Could not convert value for key to JSON")
    }

    return Fig_GetLocalStateResponse.with {
      $0.jsonBlob = String(decoding: data, as: UTF8.self)
    }
  }

  func handleSetRequest(_ request: Fig_UpdateLocalStateRequest) throws -> Bool {
    guard request.hasKey else {
      throw APIError.generic(message: "No key provided with request")
    }

    let value: Any? = {
      let valueString = request.hasValue ? request.value : nil
      guard let valueData = valueString?.data(using: .utf8) else {
        return nil
      }

      let value = try? JSONSerialization.jsonObject(with: valueData, options: .allowFragments)

      if value is NSNull {
        return nil
      }

      return value
    }()

    self.set(value: value, forKey: request.key)

    return true

  }
}
