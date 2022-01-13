//
//  MissionControl.swift
//  fig
//
//  Created by Matt Schrage on 1/12/22.
//  Copyright © 2022 Matt Schrage. All rights reserved.
//

import Cocoa

class MissionControl {
  static let shared = MissionControl()

  fileprivate var window: WebViewWindow?
  @objc class func openUI() {
    Logger.log(message: "Open MissionControl UI")

    if let window = MissionControl.shared.window {

      if window.contentViewController != nil {
        window.makeKeyAndOrderFront(nil)
        window.orderFrontRegardless()
        NSApp.activate(ignoringOtherApps: true)

        return
      } else {
        MissionControl.shared.window?.contentViewController = nil
        MissionControl.shared.window = nil
      }
    }

    let url: URL = {

      // Use value specified by developer.mission-control.host if it exists
      if let urlString = Settings.shared.getValue(forKey: Settings.missionControlURL) as? String,
         let url = URL(string: urlString) {
        return url
      }

      // otherwise use fallback
      return Remote.baseURL.appendingPathComponent("mission-control", isDirectory: true)
    }()

    let viewController = WebViewController()
    viewController.webView?.defaultURL = nil
    viewController.webView?.loadRemoteApp(at: url)
    viewController.webView?.dragShouldRepositionWindow = true

    let missionControl = WebViewWindow(viewController: viewController, shouldQuitAppOnClose: false)
    missionControl.setFrame(NSRect(x: 0, y: 0, width: 770, height: 520), display: true, animate: false)
    missionControl.center()
    missionControl.makeKeyAndOrderFront(self)

    // Set color to match background of mission-control app to avoid flicker while loading
    missionControl.backgroundColor = NSColor(hex: "#ffffff")

    missionControl.delegate = missionControl
    missionControl.isReleasedWhenClosed = false
    missionControl.level = .normal

    MissionControl.shared.window = missionControl
  }

}
