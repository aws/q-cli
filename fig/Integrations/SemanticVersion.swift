//
//  SemanticVersion.swift
//  fig
//
//  Created by Matt Schrage on 9/15/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class SemanticVersion {
  let major: Int
  let minor: Int
  let patch: Int
  let string: String

  convenience init?(version: String) {
    let semver = version.split(separator: ".").map { Int($0) }.filter { $0 != nil }
    guard semver.count == 3 else {
      return nil
    }

    guard let major = semver[0],
          let minor = semver[1],
          let patch = semver[2] else {
      return nil
    }

    self.init(major: major,
              minor: minor,
              patch: patch)
  }

  init(major: Int, minor: Int, patch: Int) {
    self.major = major
    self.minor = minor
    self.patch = patch

    self.string = "\(major).\(minor).\(patch)"
  }

}

// MARK: - Operators
extension SemanticVersion {
  static func compare(left: SemanticVersion, right: SemanticVersion, comparison: (Int, Int) -> Bool) -> Bool {
    guard comparison(left.major, right.major) else {
      return false
    }

    guard comparison(left.major, right.major) else {
      return false
    }

    guard comparison(left.major, right.major) else {
      return false
    }

    return true
  }

  static func > (left: SemanticVersion, right: SemanticVersion) -> Bool {
    return compare(left: left, right: right, comparison: >)
  }

  static func < (left: SemanticVersion, right: SemanticVersion) -> Bool {
    return compare(left: left, right: right, comparison: <)
  }

  static func == (left: SemanticVersion, right: SemanticVersion) -> Bool {
    return compare(left: left, right: right, comparison: ==)
  }

  static func <= (left: SemanticVersion, right: SemanticVersion) -> Bool {
    return compare(left: left, right: right, comparison: <=)
  }

  static func >= (left: SemanticVersion, right: SemanticVersion) -> Bool {
    return compare(left: left, right: right, comparison: >=)
  }
}
