//
//  Restarter.swift
//  fig
//
//  Created by Matt Schrage on 1/20/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Restarter {
  let bundleIdentifier: String
  init(with bundleIdentifier: String) {
    self.bundleIdentifier = bundleIdentifier
  }

  var app: NSRunningApplication?
  var kvo: NSKeyValueObservation?

  func restart(launchingIfInactive: Bool = true, completion: (() -> Void)? = nil) {
    if let app = NSWorkspace.shared.runningApplications.filter({ return $0.bundleIdentifier == self.bundleIdentifier }).first {
      self.app = app
      self.app?.terminate()
      self.kvo = self.app!.observe(\.isTerminated, options: .new) { (app, terminated) in
        if terminated.newValue == true {
          NSWorkspace.shared.launchApplication(withBundleIdentifier: self.bundleIdentifier, options: [.default], additionalEventParamDescriptor: nil, launchIdentifier: nil)
          completion?()
          self.kvo!.invalidate()
          self.app = nil
        }
      }
    } else if launchingIfInactive {
      NSWorkspace.shared.launchApplication(withBundleIdentifier: self.bundleIdentifier, options: [.default], additionalEventParamDescriptor: nil, launchIdentifier: nil)
    }
  }
}
