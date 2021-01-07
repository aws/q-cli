//
//  NativeCLI.swift
//  fig
//
//  Created by Matt Schrage on 1/6/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import KituraWebSocket

class NativeCLI {
    typealias Scope = (ShellMessage, WebSocketConnection)
    static let index = Command.openMenuBar.rawValue
    enum Command: String {
        case help = "--help"
        case h = "-h"
        case version = "--version"
        case accesibility = "util:axprompt"
        case logout = "util:logout"
        case restart = "util:restart"
        case build = "util:build"
        case feedback = "feedback"
        case invite = "invite"
        case docs = "docs"
        case update = "update"
        case source = "source"
        case resetCache = "util:reset-cache"
        case list = "list"
        case onboarding = "onboarding"
        case star = "star"
        case tweet = "tweet"
        case share = "share"
        case slack = "slack"
        case community = "community"
        case contribute = "contribute"
        case report = "issue"
        case openMenuBar = " _fig" // leading space means this can never be run directly
        case uninstall = "uninstall"

        var isUtility: Bool {
            get {
                let utilities: Set<Command> = [.resetCache, .build, .logout, .restart, .accesibility]
               return utilities.contains(self)
            }
        }
        
        var implementatedNatively: Bool {
            get {
                let implementatedNatively: Set<Command> = [.resetCache,
                                                           .build,
                                                           .logout,
                                                           .restart,
                                                           .accesibility,
                                                           .openMenuBar,
                                                           .onboarding,
                                                           .version]
               return implementatedNatively.contains(self)
            }
        }
        

        func run(_ scope: Scope) {
            guard self.implementatedNatively else {
                Logger.log(message: "CLI function '\(self.rawValue)' not implemented natively")
                return
            }
            switch self {
            case .version:
                NativeCLI.versionCommand(scope)
            case .accesibility:
                NativeCLI.accesibilityCommand(scope)
            case .restart:
                NativeCLI.restartCommand(scope)
            case .resetCache:
                NativeCLI.resetCacheCommand(scope)
            case .logout:
                NativeCLI.logoutCommand(scope)
            case .openMenuBar:
                NativeCLI.openMenuBarCommand(scope)
            case .build:
                NativeCLI.buildCommand(scope)
            case .onboarding:
                NativeCLI.onboardingCommand(scope)
            default:
                break;
            }
        }
        
        func runFromScript(_ scope: Scope) {
            guard !self.implementatedNatively else {
                Logger.log(message: "CLI function '\(self.rawValue)' is implemented natively")
                return
            }
            
            var scriptName: String? = nil
            
            // map between raw CLI command and script name
            switch self {
            case .h, .help:
                scriptName = "help"
            case .uninstall:
                scriptName = "uninstall_spec"
            case .star:
                scriptName = "contribute"
            default:
                break;
            }
            
            let script = scriptName ?? self.rawValue
            if let scriptPath = Bundle.main.path(forResource: script,
                                                 ofType: "sh") {//,
                                                 //inDirectory: "CLI Scripts") {
                NativeCLI.runShellScriptInTerminal(scriptPath, with: scope)
                print("CLI: ", scriptPath)
            } else {
                print("CLI: Failed to find script")
                NativeCLI.printError("Command does not exist.",
                           details: "Could not find the associated script for '\(script)'.")
            }
        }
    }
    
    static func route(_ command: Command, with message: ShellMessage, from connection: WebSocketConnection) {
        let scope = (message, connection)
        
        DispatchQueue.main.async {
            if command.implementatedNatively {
                command.run(scope)
            } else {
                command.runFromScript(scope)
            }
            
            connection.send(message: "disconnect")
        }
        
        trackCommandEvent(scope)
    }
}


// CLI command functions go here
extension NativeCLI {
    static func versionCommand(_ scope: Scope)  {
        let (_, connection) = scope
        let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "-1"
        NativeCLI.printInTerminal(version, using: connection)
    }
    
    static func logoutCommand(_ scope: Scope) {
        let (_, connection) = scope
        let domain = Bundle.main.bundleIdentifier!
        UserDefaults.standard.removePersistentDomain(forName: domain)
        UserDefaults.standard.synchronize()
        WebView.deleteCache()
        
        printInTerminal("→ Logging out of Fig...", using: connection)

        if let delegate = NSApp.delegate as? AppDelegate {
            delegate.restart()
        }
        
    }
    
    static func restartCommand(_ scope: Scope) {
        let (_, connection) = scope

        printInTerminal("→ Restarting Fig...", using: connection)

        if let delegate = NSApp.delegate as? AppDelegate {
            delegate.restart()
        }
        
    }
    
    static func accesibilityCommand(_ scope: Scope) {
        ShellBridge.promptForAccesibilityAccess();
    }
    
    static func resetCacheCommand(_ scope: Scope) {
        let (_, connection) = scope

        WebView.deleteCache()
        NativeCLI.printInTerminal("→ Resetting WebKit Cache...", using: connection)
    }
    
    static func openMenuBarCommand(_ scope: Scope) {
        if let delegate = NSApp.delegate as? AppDelegate {
            delegate.openMenu()
        }
    }
    
    static func onboardingCommand(_ scope: Scope) {
        let (_, connection) = scope

        if let path = NSURL(fileURLWithPath: NSString("~/.fig/tools/drip/fig_onboarding.sh").expandingTildeInPath).resourceSpecifier {
            NativeCLI.runInTerminal(command: path, over: connection)
        }
    }
    
    static func buildCommand(_ scope: Scope) {
        let (message, connection) = scope
        if let buildMode = Build(rawValue: message.arguments.first ?? "") {
            let msg = "→ Setting build to \(buildMode.rawValue)"
            NativeCLI.printInTerminal(msg, using: connection)
            Defaults.build = buildMode
        } else {
            let msg = "→ Current build is '\( Defaults.build .rawValue)'\n\n fig util:build [prod | staging | dev]"
            NativeCLI.printInTerminal(msg, using: connection)

        }
    }
}

extension NativeCLI {
//    static let scriptsFolder =
    static func runScript(_ command: Command, scriptName: String? = nil, with message: ShellMessage, from connection: WebSocketConnection) {
        
        let script = "\(scriptName ?? command.rawValue)"
        if let scriptPath = Bundle.main.path(forResource: script, ofType: "sh") {
            connection.send(message: "bash \(scriptPath)")
        } else {
            printError("Command does not exist.",
                       details: "Could not find the associated script for '\(script)'.")
        }
        
        
    }
    
    static func printError(_ error: String, details: String) {
        
    }
    
    static func printNotice(_ heading: String, details: String) {
        
    }
    
    static func printUsage(_ heading: String, details: String) {
        
    }
    
    static func printInTerminal(_ message: String, using connection: WebSocketConnection) {
        runInTerminal(command: "echo \"\(message)\"", over: connection)
    }
    
    static func runInTerminal(command: String, over connection: WebSocketConnection) {
        connection.send(message: command)
    }
    static func runShellScriptInTerminal(_ script: String, with scope: Scope) {
        let (message, connection) = scope
        
        let args = message.arguments.map { $0.contains(" ") ? "\"\($0)\"" : $0 } .joined(separator: " ")
    
        runInTerminal(command: "bash \(script) \(args)", over: connection)
       
    }
    
    static func trackCommandEvent(_ scope: Scope) {
        let (message, _) = scope
        
        let obuscatedArgs = message.arguments.map { TelemetryProvider.obscure($0) }.joined(separator: " ")
        
        TelemetryProvider.track(event: .ranCommand,
                                with:   [
                                        "command" : message.subcommand ?? "",
                                        "arguments" : obuscatedArgs,
                                        "shell" : message.shell ?? "<unknown>",
                                        "terminal" : message.terminal ?? "<unknown>"
                                        ])
    }
    
}
