//
//  Onboarding.swift
//  fig
//
//  Created by Matt Schrage on 6/5/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

enum WindowCloseBehavior {
  case terminateApplicationWhenClosed
  case hideWindowWhenClosed
  case defaultBehavior
}
class WebViewWindow: NSWindow {
  let restoreAccessoryPolicyOnClose: Bool
  var behaviorOnClose: WindowCloseBehavior = .defaultBehavior
  init(viewController: NSViewController,
       shouldQuitAppOnClose: Bool = true,
       isLongRunningWindow: Bool = false,
       restoreAccessoryPolicyOnClose: Bool = false) {
    self.restoreAccessoryPolicyOnClose = restoreAccessoryPolicyOnClose

    if shouldQuitAppOnClose {
      self.behaviorOnClose = .terminateApplicationWhenClosed
    } else if isLongRunningWindow {
      self.behaviorOnClose = .hideWindowWhenClosed
    } else {
      self.behaviorOnClose = .defaultBehavior
    }

    super.init(
      contentRect: NSRect(x: 0, y: 0, width: 520, height: 350),
      styleMask: [.fullSizeContentView, .resizable, .titled, .miniaturizable, .closable],
      backing: .buffered, defer: false)
    self.center()
    self.titlebarAppearsTransparent = true

    self.isMovableByWindowBackground = true
    self.backgroundColor = NSColor.white
    self.level = .floating
    self.setFrameAutosaveName("Main Window")
    self.contentViewController = viewController
    self.makeKeyAndOrderFront(nil)
    self.delegate = self
    self.title = "Loading..."

    if let closeButton = self.standardWindowButton(.closeButton) {
      closeButton.target = self
      closeButton.action = #selector(closeWindow)
    }

  }

  @objc func closeWindow() {

    switch self.behaviorOnClose {
    case .terminateApplicationWhenClosed:
      self.close()
      Logger.log(message: "Close via button press!")
      if let delegate = NSApp.delegate as? AppDelegate {
        delegate.quit()
      }
    case .hideWindowWhenClosed:
      if self.restoreAccessoryPolicyOnClose {
        NSApp.setActivationPolicy(.accessory)
      }
      self.orderOut(nil)
    case .defaultBehavior:
      self.close()
    }
  }
}

extension WebViewWindow: NSWindowDelegate {
  func windowShouldClose(_ sender: NSWindow) -> Bool {
    self.contentViewController = nil
    return true
  }
}
