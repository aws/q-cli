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
    
    // check current hash with ...
    static let commitHashForVersion = ""//["1.0.24" : "hi"]
    
    static func setUpEnviroment(completion:( () -> Void)? = nil) {
      
        if !Diagnostic.isRunningOnReadOnlyVolume {
            SentrySDK.capture(message: "Currently running on read only volume! App is translocated!")
        }
        
        guard let path = Bundle.main.path(forResource: "install_and_upgrade", ofType: "sh", inDirectory: "config/tools") else {
            return Logger.log(message: "Could not locate install script!")
        }
        
        let configDirectory = Bundle.main.resourceURL?.appendingPathComponent("config", isDirectory: true).path
        
      "/bin/bash '\(path)' local".runInBackground(cwd: configDirectory,
                                                  with: [ "FIG_BUNDLE_EXECUTABLES" : Bundle.main.url(forAuxiliaryExecutable: "")!.path ],
                                                  completion:  { transcript in
            Onboarding.symlinkBundleExecutable("figcli", to: "~/.fig/bin/fig")
            Onboarding.symlinkBundleExecutable("figterm", to: "~/.fig/bin/figterm")
            Onboarding.symlinkBundleExecutable("fig_get_shell", to: "~/.fig/bin/fig_get_shell")
            Onboarding.symlinkBundleExecutable("fig_callback", to: "~/.fig/bin/fig_callback")
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
                try? FileManager.default.createDirectory(at: fullURL.deletingLastPathComponent(), withIntermediateDirectories: true, attributes: [:])
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
    
    static func setupTerminalsForShellOnboarding(completion: (()->Void)? = nil) {
        WindowManager.shared.newNativeTerminalSession(completion: completion)
    }
}

import FigAPIBindings
import WebKit
extension Onboarding {
  static func handleRequest(_ request: Fig_OnboardingRequest, in webView: WKWebView, callback: @escaping ((Bool) -> Void)) {
    
    switch request.action {
      case .installationScript:
        Onboarding.setUpEnviroment {
          callback(true)
        }
      case .promptForAccessibilityPermission:
        Accessibility.promptForPermission { status in
          callback(true)
        }
      case .launchShellOnboarding:
        callback(true)
        webView.window?.close()
        Defaults.loggedIn = true

        Onboarding.setupTerminalsForShellOnboarding {
          SecureKeyboardInput.notifyIfEnabled()
        }
    
        NSApp.appDelegate.setupCompanionWindow()
      case .uninstall:
        NSApp.appDelegate.uninstall()
      case .UNRECOGNIZED(_):
        Logger.log(message: "Unrecognized Onboarding Action!", subsystem: .api)
        callback(false)
    }
  }
}
