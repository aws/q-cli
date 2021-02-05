//
//  BundleCache.swift
//  fig
//
//  Created by Matt Schrage on 2/3/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class BundleIdCache {
  
  fileprivate static var mapping: [pid_t: String?] = [:]
  static func getBundleId(for pid: pid_t) -> String? {
    if let cached = mapping[pid] {
      return cached
    }
    
    let app = NSRunningApplication(processIdentifier: pid)
    return getBundleId(for: app)
  }
  
  static func getBundleId(for app: NSRunningApplication?) -> String? {
    guard let app = app else { return nil }
    if let cached = mapping[app.processIdentifier] {
      return cached
    }
    
    let bundleId = app.bundleIdentifier
    mapping[app.processIdentifier] = bundleId
  
    return bundleId
  }
}

extension NSRunningApplication {
  var cachedBundleIdentifier: String? {
    return BundleIdCache.getBundleId(for: self)
  }
}
