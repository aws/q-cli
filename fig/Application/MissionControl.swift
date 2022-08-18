//
//  MissionControl.swift
//  fig
//
//  Created by Matt Schrage on 1/12/22.
//  Copyright © 2022 Matt Schrage. All rights reserved.
//

import Cocoa
import FigAPIBindings

class MissionControl {
  static let shared = MissionControl()

  fileprivate var window: WebViewWindow?

  // swiftlint:disable type_name
  @objc enum Tab: Int {
    case home = 0
    case settings = 1
    case plugins = 2
    case onboarding = 3

    func endpoint() -> String {
      switch self {
      case .settings:
        return "settings"
      case .plugins:
        return "plugins"
      case .home:
        return ""
      case .onboarding:
        return "onboarding/welcome"
      }
    }
  }

  static func launchOnboarding() {
    MissionControl.openUI(.onboarding)

    self.shared.window?.setFrame(NSRect(x: 0, y: 0, width: 590, height: 480), display: true, animate: false)
    self.shared.window?.center()
    self.shared.window?.behaviorOnClose = .terminateApplicationWhenClosed
  }

  init() {
    NotificationCenter.default.addObserver(self, selector: #selector(windowDidChange(_:)),
                                           name: AXWindowServer.windowDidChangeNotification,
                                           object: nil)

    NSWorkspace.shared.notificationCenter.addObserver(
      self,
      selector: #selector(windowDidChange(_:)),
      name: NSWorkspace.activeSpaceDidChangeNotification,
      object: nil
    )
  }

  @objc func windowDidChange(_ notification: Notification) {
    if let window = notification.object as? ExternalWindow,
           window.isFullScreen ?? false == true {
      // Enable autocomplete to show up in full-screen applications
      NSApp.setActivationPolicy(.accessory)
      return
    }

    if self.window?.isVisible ?? false {
      // If Dashboard window exists, show Fig icon in dock
      NSApp.setActivationPolicy(.regular)
    } else {
      // If Dashboard is closed, remove Fig icon from dock
      NSApp.setActivationPolicy(.accessory)
    }
  }

  @objc static func openDashboard() {
    MissionControl.openUI(.home)
  }

  @objc class func openUI(_ tab: Tab = .home, additionalPathComponent: String? = nil) {
    Logger.log(message: "Open MissionControl UI")

    let url: URL = {

      // Use value specified by developer.mission-control.host if it exists
      if let urlString = Settings.shared.getValue(forKey: Settings.missionControlURL) as? String,
         let url = URL(string: urlString) {
        return url.appendingPathComponent(tab.endpoint()).appendingPathComponent(additionalPathComponent ?? "")
      }

      // otherwise use fallback
      return Remote.missionControlURL.appendingPathComponent(tab.endpoint())
                                     .appendingPathComponent(additionalPathComponent ?? "")
    }()

    if let window = MissionControl.shared.window {

      if window.contentViewController != nil {
        if self.shouldShowIconInDock {
          if NSApp.activationPolicy() == .accessory {
            NSApp.setActivationPolicy(.regular)
          }
        }

        window.makeKeyAndOrderFront(nil)
        window.orderFrontRegardless()
        NSApp.activate(ignoringOtherApps: true)

        if let controller = window.contentViewController as? WebViewController,
           let currentURL = controller.webView?.url,
           currentURL != url {

          // If host is the same, use event to trigger page change without full refresh
          if currentURL.host == url.host {
            API.notifications.post(Fig_EventNotification.with({ event in
              event.eventName = "mission-control.navigate"
              event.payload = "{ \"path\": \"\(url.path)\" }"
            }))
          } else {
            controller.webView?.navigate(to: url)
          }

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

    let missionControl = WebViewWindow(viewController: viewController,
                                       shouldQuitAppOnClose: false,
                                       isLongRunningWindow: true,
                                       restoreAccessoryPolicyOnClose:
                                        self.shouldShowIconInDock)
    missionControl.setFrame(NSRect(x: 0, y: 0, width: 1030, height: 720), display: true, animate: false)
    missionControl.center()
    missionControl.makeKeyAndOrderFront(self)

    // Set color to match background of mission-control app to avoid flicker while loading
    let mode = UserDefaults.standard.string(forKey: "AppleInterfaceStyle")

    missionControl.backgroundColor = mode == "Dark" ? NSColor(hex: "#000000") : NSColor(hex: "#ffffff")

    missionControl.delegate = missionControl
    missionControl.isReleasedWhenClosed = false
    missionControl.level = .normal

    if self.shouldShowIconInDock {
      if NSApp.activationPolicy() == .accessory {
        NSApp.setActivationPolicy(.regular)
      }
    }

    MissionControl.shared.window = missionControl
    NSApp.activate(ignoringOtherApps: true)
  }

  static var shouldShowIconInDock: Bool {
    return true // LocalState.shared.getValue(forKey: LocalState.showIconInDock) as? Bool ??  false
  }
}
