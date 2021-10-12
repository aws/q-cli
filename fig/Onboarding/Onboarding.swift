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
        
        
        "/bin/bash '\(path)'".runInBackground()
    }

    static func copyFigCLIExecutable(to path: String) {
        let fullPath = NSString(string: path).expandingTildeInPath
        let existingSymlink = try? FileManager.default.destinationOfSymbolicLink(atPath: fullPath)

        if let cliPath = Bundle.main.path(forAuxiliaryExecutable: "figcli"), existingSymlink != cliPath {
            do {
                try? FileManager.default.removeItem(atPath: fullPath)
                let fullURL = URL(fileURLWithPath: fullPath)
                try? FileManager.default.createDirectory(at: fullURL.deletingLastPathComponent(), withIntermediateDirectories: true, attributes: [:])
                try FileManager.default.createSymbolicLink(at: fullURL, withDestinationURL: URL(fileURLWithPath: cliPath))
            } catch {
                Logger.log(message: "Could not download copy CLI to ~/.fig/bin")
                SentrySDK.capture(message: "Could not download copy CLI to ~/.fig/bin")
            }
        }
        
    }
    
    static func setupTerminalsForShellOnboarding(completion: (()->Void)? = nil) {
        WindowManager.shared.newNativeTerminalSession(completion: completion)
        
        // filter for native terminal windows (with hueristic to avoid menubar items + other window types)
//        let nativeTerminals = NSWorkspace.shared.runningApplications.filter { Integrations.nativeTerminals.contains($0.bundleIdentifier ?? "")}
//
//        let count = nativeTerminals.count
//        guard count > 0 else {
//            WindowManager.shared.newNativeTerminalSession(completion: completion)
//            return
//        }
//        let iTermOpen = nativeTerminals.contains { $0.bundleIdentifier == "com.googlecode.iterm2" }
//        let terminalAppOpen = nativeTerminals.contains { $0.bundleIdentifier == "com.apple.Terminal" }
//
//        var emulators: [String] = []
//
//        if (iTermOpen) {
//            emulators.append("iTerm")
//        }
//
//        if (terminalAppOpen) {
//            emulators.append("Terminal")
//        }
//
//        let restart = (NSApp.delegate as! AppDelegate).dialogOKCancel(question: "Fig will not work in existing terminal sessions", text: "Restart existing terminal sessions.\n", prompt: "Restart \(emulators.joined(separator: " and "))", noAction: false, icon: NSImage.init(imageLiteralResourceName: NSImage.applicationIconName), noActionTitle: "Open new terminal window")
//
//        // only restart one of the terminals, so that shell onboarding doesn't appear twice
//        if (restart) {
//            TelemetryProvider.track(event: .restartForOnboarding, with: [:])
//
//            guard !iTermOpen else {
//                let iTerm = Restarter(with: "com.googlecode.iterm2")
//                iTerm.restart(completion: completion)
//                return
//            }
//
//
//            let terminalApp = Restarter(with: "com.apple.Terminal")
//            terminalApp.restart(completion: completion)
//        } else {
//            TelemetryProvider.track(event: .newWindowForOnboarding, with: [:])
//            // if the user doesn't want to restart their terminal, revert to previous approach of creating new window.
//            WindowManager.shared.newNativeTerminalSession(completion: completion)
//        }
    }
}
