//
//  TerminalUsageTelemetry.swift
//  fig
//
//  Created by Matt Schrage on 10/19/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class TerminalUsageObserver {
    static let terminalApplicationLostFocusNotification: NSNotification.Name = Notification.Name("terminalApplicationLostKeyNotification")

    var observer: NSKeyValueObservation? = nil
    var start: Date = Date(timeIntervalSinceNow: 0)
    init() {
        
        self.observer = NSWorkspace.shared.observe(\.frontmostApplication, options: [.old]) { (workspace, delta) in
            print("old: ", delta.oldValue as Any)
            if let app = delta.oldValue {
                let now = Date(timeIntervalSinceNow: 0)
                let delta = now.timeIntervalSince(self.start)
                Logger.log(message: "Application changed from \(app?.bundleIdentifier ?? "<none>") after \(delta) seconds")
                if Integrations.nativeTerminals.contains(app?.bundleIdentifier ?? "")  {
                    NotificationCenter.default.post(name: TerminalUsageObserver.terminalApplicationLostFocusNotification, object: delta)
                }
                self.start = now
            }
        }
    }
}
