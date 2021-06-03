//
//  Onboarding.swift
//  fig
//
//  Created by Matt Schrage on 6/5/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class WebViewWindow : NSWindow {
    init(viewController: NSViewController, shouldQuitAppOnClose: Bool = true) {
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
                
        if let closeButton = self.standardWindowButton(.closeButton), shouldQuitAppOnClose {
            closeButton.target = self
            closeButton.action = #selector(closeViaButton)
        }
        
    }
    
    @objc func closeViaButton() {
        self.close()
        Logger.log(message: "Close via button press!")
        if let delegate = NSApp.delegate as? AppDelegate {
            delegate.quit()
        }
    }
}

extension WebViewWindow: NSWindowDelegate {
  func windowShouldClose(_ sender: NSWindow) -> Bool {
    self.contentViewController = nil
    return true
  }
}
