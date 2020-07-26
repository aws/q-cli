//
//  Onboarding.swift
//  fig
//
//  Created by Matt Schrage on 6/5/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class OnboardingWindow : NSWindow {
     init(viewController: NSViewController) {
            super.init(
                contentRect: NSRect(x: 0, y: 0, width: 520, height: 350),
                styleMask: [.fullSizeContentView, .resizable, .titled, .miniaturizable, .closable],
                backing: .buffered, defer: false)
            self.center()
//            self.title = "Fig"
            self.titlebarAppearsTransparent = true

//            self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
            self.isMovableByWindowBackground = true
//            self.isOpaque = false
            self.backgroundColor = NSColor.white//.clear//NSColor.init(white: 1, alpha: 0.75)
//            self.level = .floating
            self.setFrameAutosaveName("Main Window")
            self.contentViewController = viewController //WebViewController()
            self.makeKeyAndOrderFront(nil)
    }
}
