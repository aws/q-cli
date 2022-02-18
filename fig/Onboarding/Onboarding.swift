//
//  Onboarding.swift
//  fig
//
//  Created by Matt Schrage on 8/19/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Sentry

class Onboarding {

  static let loginURL: URL = Remote.baseURL.appendingPathComponent("login", isDirectory: true)

  static func setUpEnviroment(completion:( () -> Void)? = nil) {

    if !Diagnostic.isRunningOnReadOnlyVolume {
      SentrySDK.capture(message: "Currently running on read only volume! App is translocated!")
    }

    guard let path = Bundle.main.path(forAuxiliaryExecutable: "dotfilesd-darwin-universal") else {
      return Logger.log(message: "Could not locate install script!")
    }

    let configDirectory = Bundle.main.resourceURL?.appendingPathComponent("config", isDirectory: true).path

    "\(path) app install".runInBackground(cwd: configDirectory,
                                                with: [ "FIG_BUNDLE_EXECUTABLES":
                                                    Bundle.main.url(forAuxiliaryExecutable: "")!.path ],
                                                completion: { _ in
                                                  // Install launch agent that watches for Fig.app being trashed
                                                  LaunchAgent.uninstallWatcher.addIfNotPresent()

                                                  Onboarding.symlinkBundleExecutable("figterm",
                                                                                     to: "~/.fig/bin/figterm")
                                                  Onboarding.symlinkBundleExecutable("fig_get_shell",
                                                                                     to: "~/.fig/bin/fig_get_shell")
                                                  Onboarding.symlinkBundleExecutable("fig_callback",
                                                                                     to: "~/.fig/bin/fig_callback")
                                                  Onboarding.symlinkBundleExecutable("dotfilesd-darwin-universal",
                                                                                     to: "~/.local/bin/fig")
                                                  completion?()
                                                })
  }

  static func symlinkBundleExecutable(_ executable: String, to path: String) {
    let fullPath = NSString(string: path).expandingTildeInPath
    let existingSymlink = try? FileManager.default.destinationOfSymbolicLink(atPath: fullPath)

    if let cliPath = Bundle.main.path(forAuxiliaryExecutable: executable), existingSymlink != cliPath {
      do {
        try? FileManager.default.removeItem(atPath: fullPath)
        let fullURL = URL(fileURLWithPath: fullPath)
        try? FileManager.default.createDirectory(
          at: fullURL.deletingLastPathComponent(),
          withIntermediateDirectories: true,
          attributes: [:]
        )
        try FileManager.default.createSymbolicLink(at: fullURL, withDestinationURL: URL(fileURLWithPath: cliPath))
      } catch {
        Logger.log(message: "Could not symlink executable '\(executable)' to '\(path)'")
        SentrySDK.capture(message: "Could not symlink executable '\(executable)' to '\(path)'")
      }
    }

  }

  static func copyFigCLIExecutable(to path: String) {
    symlinkBundleExecutable("figcli", to: path)
  }

  static func setupTerminalsForShellOnboarding(completion: (() -> Void)? = nil) {
    WindowManager.shared.newNativeTerminalSession(completion: completion)
  }
}

import FigAPIBindings
import WebKit
extension Onboarding {
  static func handleRequest(
    _ request: Fig_OnboardingRequest,
    in webView: WKWebView,
    callback: @escaping ((Bool) -> Void)
  ) {

    switch request.action {
    case .installationScript:
      Onboarding.setUpEnviroment {
        callback(true)
      }
    case .promptForAccessibilityPermission:
      Accessibility.promptForPermission { _ in
        callback(true)
      }
    case .closeAccessibilityPromptWindow:
      Accessibility.closeUI()
    case .launchShellOnboarding:
      callback(true)
      webView.window?.close()
      Defaults.shared.loggedIn = true

      Onboarding.setupTerminalsForShellOnboarding {
        SecureKeyboardInput.notifyIfEnabled()
      }

      NSApp.appDelegate.setupCompanionWindow()
    case .uninstall:
      NSApp.appDelegate.uninstall()
    case .UNRECOGNIZED:
      Logger.log(message: "Unrecognized Onboarding Action!", subsystem: .api)
      callback(false)
    }
  }
}
