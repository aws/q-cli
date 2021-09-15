//
//  KittyIntegration.swift
//  fig
//
//  Created by Matt Schrage on 9/13/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class KittyIntegration: InputMethodDependentTerminalIntegrationProvider {
    static let `default` = KittyIntegration(bundleIdentifier: Integrations.Kitty)
    
    static let configDirectory: URL = URL(fileURLWithPath: NSHomeDirectory() + "/.config/kitty")
    // https://sw.kovidgoyal.net/kitty/faq/#how-do-i-specify-command-line-options-for-kitty-on-macos
    static let cmdlineFilename = "macos-launch-services-cmdline"
    static let cmdlineFilepath = configDirectory.appendingPathComponent(cmdlineFilename)
    static let pythonScriptPath = Bundle.main.path(forResource: "kitty-integration", ofType: "py")!//"~/.fig/tools/kitty-integration.py"
    static let commandLineArguments = "--watcher \(pythonScriptPath)"
}

extension KittyIntegration: IntegrationProvider {
    func verifyInstallation() -> InstallationStatus {
        
        guard FileManager.default.fileExists(atPath: KittyIntegration.cmdlineFilepath.path) else {
            return .failed(error: "'\(KittyIntegration.cmdlineFilepath.path)' file does not exist")
        }
        
        guard let kittyCommandLine = try? String(contentsOf: KittyIntegration.cmdlineFilepath) else {
            return .failed(error: "Could not read '\(KittyIntegration.cmdlineFilepath.path)'")
        }
        
        guard kittyCommandLine.contains(KittyIntegration.commandLineArguments) else {
            return .failed(error: "\(KittyIntegration.cmdlineFilename) does not contains --watcher")
        }
        
        let inputMethodStatus = InputMethod.default.verifyInstallation()
        guard inputMethodStatus == .installed else {
            return .pending(event: .inputMethodActivation)
        }
        
        return .installed
    }
    
    func install() -> InstallationStatus {
        guard self.applicationIsInstalled else {
            return .applicationNotInstalled
        }
        
        if FileManager.default.fileExists(atPath: KittyIntegration.cmdlineFilepath.path) {
            
            guard let kittyCommandLine = try? String(contentsOf: KittyIntegration.cmdlineFilepath) else {
                return .failed(error: "Could not read '\(KittyIntegration.cmdlineFilepath.path)'")
            }
            
            guard kittyCommandLine.contains(KittyIntegration.commandLineArguments) else {
                // todo(mschrage): we should have a support page for this case
                return .failed(error: "\(KittyIntegration.cmdlineFilename) already exists and contains user-specified configuration.", supportURL: nil)
            }
        
        } else {
            do {
                try KittyIntegration.commandLineArguments.write(toFile: KittyIntegration.cmdlineFilepath.path,
                                                                atomically: true,
                                                                encoding: .utf8)
            } catch {
                return .failed(error: "Could not write to \(KittyIntegration.cmdlineFilename)")
            }
        }
        
        // what is the minimum version where the integration works?
        
        return .pending(event: .applicationRestart)
    }
}

extension KittyIntegration : TerminalIntegration {
    func getCursorRect(in window: ExternalWindow) -> NSRect? {
        return InputMethod.getCursorRect()
    }
    
    func terminalIsFocused(in window: ExternalWindow) -> Bool {
        return true
    }
    
    
}
