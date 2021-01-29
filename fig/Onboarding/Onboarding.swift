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
        
        DispatchQueue.global(qos: .userInitiated).async {
            let githubURL = URL(string: "https://raw.githubusercontent.com/withfig/config/main/tools/install_and_upgrade.sh")!
            let fallbackURL = Bundle.main.url(forResource: "install_and_upgrade_fallback", withExtension: "sh")!
            
            var script = try? String(contentsOf: githubURL)
            
            if (script == nil) {
              script = try? String(contentsOf: fallbackURL)
            }
          
            if let envSetupScript = script {
                let scriptsURL = FileManager.default.urls(for: .applicationScriptsDirectory, in: .userDomainMask)[0] as NSURL

                guard let folderPath = scriptsURL.path else {
                    Logger.log(message: "Folder path does not exist")
                    return
                }
                
                Logger.log(message: String(describing: scriptsURL.path))

                guard let script = scriptsURL.appendingPathComponent("install_and_upgrade.sh") else {
                    Logger.log(message: "Could not create PATH for install_and_upgrade.sh")
                    SentrySDK.capture(message: "Could not create PATH for install_and_upgrade.sh")
                    return
                }
                Logger.log(message: script.path)

                do {
                    try FileManager.default.createDirectory(atPath: folderPath, withIntermediateDirectories: true)
                    try envSetupScript.write(to: script, atomically: true, encoding: String.Encoding.utf8)
                } catch {
                    SentrySDK.capture(message: "Could not write to file.")
                    Logger.log(message: "Could not write to file.")

                    return
                }


                print("onboarding: ", script)
                
                guard let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String else {
                    Logger.log(message: "No version availible")
                    return
                }
                
                let out = "/bin/bash '\(script.path)' v\(version)".runAsCommand()
                
                guard !out.starts(with: "Error:") else {
                    Logger.log(message: out)
                    SentrySDK.capture(message: "Onboarding: \(out)")
                    return
                }
                
                Logger.log(message: "Successfully ran installation script!")
                Logger.log(message: "\(out)")
                SentrySDK.capture(message: "Script: \(out)")

                
            } else {
                Logger.log(message: "Could not download installation script")
                SentrySDK.capture(message: "Could not download installation script")
                // What should we do when this happens?
            }
        }
    }

    static func copyFigCLIExecutable(to path: String) {
        let fullPath = NSString(string: path).expandingTildeInPath
        let existingSymlink = try? FileManager.default.destinationOfSymbolicLink(atPath: fullPath)

        if let cliPath = Bundle.main.path(forAuxiliaryExecutable: "figcli"), existingSymlink != cliPath {
            do {
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
        // filter for native terminal windows (with hueristic to avoid menubar items + other window types)
        let nativeTerminals = NSWorkspace.shared.runningApplications.filter { Integrations.nativeTerminals.contains($0.bundleIdentifier ?? "")}
        
        let count = nativeTerminals.count
        guard count > 0 else {
            WindowManager.shared.newNativeTerminalSession(completion: completion)
            return
        }
        let iTermOpen = nativeTerminals.contains { $0.bundleId == "com.googlecode.iterm2" }
        let terminalAppOpen = nativeTerminals.contains { $0.bundleId == "com.apple.Terminal" }
        
        var emulators: [String] = []
        
        if (iTermOpen) {
            emulators.append("iTerm")
        }
        
        if (terminalAppOpen) {
            emulators.append("Terminal")
        }
                
        let restart = (NSApp.delegate as! AppDelegate).dialogOKCancel(question: "Fig will not work in existing terminal sessions", text: "Restart existing terminal sessions.\n", prompt: "Restart \(emulators.joined(separator: " and "))", noAction: false, icon: NSImage.init(imageLiteralResourceName: NSImage.applicationIconName), noActionTitle: "Open new terminal window")
        
        // only restart one of the terminals, so that shell onboarding doesn't appear twice
        if (restart) {
            TelemetryProvider.track(event: .restartForOnboarding, with: [:])

            guard !iTermOpen else {
                let iTerm = Restarter(with: "com.googlecode.iterm2")
                iTerm.restart(completion: completion)
                return
            }

            
            let terminalApp = Restarter(with: "com.apple.Terminal")
            terminalApp.restart(completion: completion)
        } else {
            TelemetryProvider.track(event: .newWindowForOnboarding, with: [:])
            // if the user doesn't want to restart their terminal, revert to previous approach of creating new window.
            WindowManager.shared.newNativeTerminalSession(completion: completion)
        }
    }
}
