//
//  MissionControl.swift
//  fig
//
//  Created by Matt Schrage on 1/12/22.
//  Copyright © 2022 Matt Schrage. All rights reserved.
//

import Cocoa
import SwiftUI

class MissionControl {
  static let shared = MissionControl()

  fileprivate var window: WebViewWindow?

  @objc enum Tab: Int {
    case home = 0
    case settings = 1
    func endpoint() -> String {
      switch self {
      case .settings:
        return "settings"
      case .home:
        return ""
      }
    }
  }

  @objc class func openUI(_ tab: Tab = .home) {
    Logger.log(message: "Open MissionControl UI")

    let url: URL = {

      // Use value specified by developer.mission-control.host if it exists
      if let urlString = Settings.shared.getValue(forKey: Settings.missionControlURL) as? String,
         let url = URL(string: urlString) {
        return url.appendingPathComponent(tab.endpoint())
      }

      // otherwise use fallback
      return Remote.missionControlURL.appendingPathComponent(tab.endpoint())
    }()

    if let window = MissionControl.shared.window {

      if window.contentViewController != nil {
        window.makeKeyAndOrderFront(nil)
        window.orderFrontRegardless()
        NSApp.activate(ignoringOtherApps: true)

        if let vc = window.contentViewController as? WebViewController {
          vc.webView?.loadRemoteApp(at: url)
        }
        return
      } else {
        MissionControl.shared.window?.contentViewController = nil
        MissionControl.shared.window = nil
      }
    }

    let viewController = WebViewController()
    viewController.webView?.defaultURL = nil
    viewController.webView?.loadRemoteApp(at: url)
    viewController.webView?.dragShouldRepositionWindow = true

    let missionControl = WebViewWindow(viewController: viewController, shouldQuitAppOnClose: false)
    missionControl.setFrame(NSRect(x: 0, y: 0, width: 830, height: 570), display: true, animate: false)
    missionControl.center()
    missionControl.makeKeyAndOrderFront(self)

    // Set color to match background of mission-control app to avoid flicker while loading
    missionControl.backgroundColor = NSColor(hex: "#ffffff")

    missionControl.delegate = missionControl
    missionControl.isReleasedWhenClosed = false
    missionControl.level = .normal

    if NSApp.activationPolicy() == .accessory {
        NSApp.setActivationPolicy(.regular)
    }

    MissionControl.shared.window = missionControl
    NSApp.activate(ignoringOtherApps: true)
  }

}
