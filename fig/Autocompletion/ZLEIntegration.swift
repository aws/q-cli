//
//  ZLEIntegration.swift
//  fig
//
//  Created by Matt Schrage on 2/2/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class ZLEIntegration {
  static let insertionLock = "\(NSHomeDirectory())/.fig/insertion-lock"

  static func insertLock() {
    // The existence of the insertion-lock file prevents latency in ZLE integration when inserting text
    // See the `self-insert` function in zle.sh
    FileManager.default.createFile(atPath: insertionLock, contents: nil, attributes: nil)

  }
  
  static func insertUnlock(with insertionText: String) {
      // remove lock after keystrokes have been processes (requires delay)
      Timer.delayWithSeconds(0.1) {
          try! FileManager.default.removeItem(atPath: insertionLock)
          Autocomplete.position()

      }
    
      // If ZLE, manually update keybuffer
      if let window = AXWindowServer.shared.whitelistedWindow,
         let context = KeypressProvider.shared.keyBuffer(for: window).insert(text: insertionText) {
          // trigger an update!
          print("update: \(context.0)")
          Autocomplete.update(with: context, for: window.hash)
          Autocomplete.position()
        
        
      }
  }
  
}
