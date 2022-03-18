//
//  WindowObserver.swift
//  fig
//
//  Created by Matt Schrage on 1/29/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class WindowObserver {
  let bundleIdentifier: String
  init?(with bundleIdentifier: String) {

    // ensure app is installed
    guard NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleIdentifier) != nil else {
      return nil
    }

    self.bundleIdentifier = bundleIdentifier
  }

  var completion: (() -> Void)?
  var timer: DispatchWorkItem?
  func windowDidAppear(timeoutAfter interval: TimeInterval? = nil, completion: @escaping (() -> Void)) {
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(windowDidChange(_ :)),
                                           name: AXWindowServer.windowDidChangeNotification,
                                           object: nil)
    self.completion = completion

    if let timeout = interval {
      timer = Timer.cancellableDelayWithSeconds(timeout) {
        completion()
        // swiftlint:disable notification_center_detachment
        NotificationCenter.default.removeObserver(self)
      }
    }

  }

  @objc func windowDidChange(_ notification: Notification) {
    guard let window = notification.object as? ExternalWindow else { return }

    if self.bundleIdentifier == window.bundleId {
      timer?.cancel()
      completion?()
      // swiftlint:disable notification_center_detachment
      NotificationCenter.default.removeObserver(self)
    }
  }

  deinit {
    NotificationCenter.default.removeObserver(self)
  }

}
