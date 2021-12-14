//
//  PathHelpers.swift
//  fig
//
//  Created by Matt Schrage on 11/10/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class PathHelper {

  // https://scriptingosx.com/2017/05/where-paths-come-from/
  static let defaultMacOSLocations: Set<String> = ["/usr/local/bin", "/usr/bin", "/bin", "/usr/sbin", "/sbin"]
  static let homebrewIntel = "/usr/local/bin"
  static let homebrewAppleSilicon = "/opt/homebrew/bin"

  static var wellKnownLocations: Set<String> {
    return defaultMacOSLocations.union( ProcessInfo.processInfo.isAppleSilicon ? [homebrewAppleSilicon]
                                          : [homebrewIntel])
  }

  static var defaultMacOSPath: String {
    return PathHelper.path(from: defaultMacOSLocations)
  }

  static var defaultPath: String {
    return PathHelper.path(from: wellKnownLocations)
  }

  static func locations(for path: String) -> Set<String> {
    return Set<String>(path.split(separator: ":").map { String($0) })
  }

  static func path(from locations: Set<String>) -> String {
    return locations.joined(separator: ":")
  }

  static func pathByPrependingMissingWellKnownLocations(_ path: String) -> String {
    let locations = locations(for: path)
    let missingLocations = wellKnownLocations.subtracting(locations)

    if missingLocations.count > 0 {
      return PathHelper.path(from: missingLocations) + ":" + path
    } else {
      return path
    }

  }
}

// https://developer.apple.com/forums/thread/652667
extension ProcessInfo {
  /// Returns a `String` representing the machine hardware name or nil if there was an error invoking `uname(_:)` or
  /// decoding the response.
  ///
  /// Return value is the equivalent to running `$ uname -m` in shell.
  var machineHardwareName: String? {
    var sysinfo = utsname()
    let result = uname(&sysinfo)
    guard result == EXIT_SUCCESS else { return nil }
    let data = Data(bytes: &sysinfo.machine, count: Int(_SYS_NAMELEN))
    guard let identifier = String(bytes: data, encoding: .ascii) else { return nil }
    return identifier.trimmingCharacters(in: .controlCharacters)
  }

  var isAppleSilicon: Bool {
    return self.machineHardwareName == "arm64"
  }
}
