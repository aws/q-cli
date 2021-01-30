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
  
  var completion: (()-> Void)?
  func windowDidAppear(completion: @escaping (()-> Void)) {
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(windowDidChange(_ :)),
                                           name: AXWindowServer.windowDidChangeNotification,
                                           object: nil)
    self.completion = completion
    
  }
  
  @objc func windowDidChange(_ notification: Notification) {
    guard let window = notification.object as? ExternalWindow else { return }
    
    if (self.bundleIdentifier == window.bundleId) {
      completion?()
      NotificationCenter.default.removeObserver(self)
    }
  }
  
  deinit {
    NotificationCenter.default.removeObserver(self)
  }
  
}
