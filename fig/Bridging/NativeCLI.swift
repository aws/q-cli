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
        case helpCommand =  "help"
        case version = "--version"
        case accessibility = "util:axprompt"
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
        case contribute = "contribute"
        case issue = "issue"
        case openMenuBar = " _fig" // leading space means this can never be run directly
        case uninstall = "uninstall"
        case disable = "disable"
        case remove = "remove"
        case report = "report"
        case ssh = "integrations:ssh"
        case teamUpload = "team:upload"
        case teamDownload = "team:download"
        case diagnostic = "diagnostic"

        var isUtility: Bool {
            get {
                let utilities: Set<Command> = [.resetCache, .build, .logout, .restart, .accessibility]
               return utilities.contains(self)
            }
        }

        var implementatedNatively: Bool {
            get {
                let implementatedNatively: Set<Command> = [.resetCache,
                                                           .build,
                                                           .logout,
                                                           .restart,
                                                           .accessibility,
                                                           .openMenuBar,
                                                           .onboarding,
                                                           .version,
                                                           .report,
                                                           .diagnostic,
                                                           .docs]
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
            case .accessibility:
                NativeCLI.accessibilityCommand(scope)
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
            case .docs:
                NativeCLI.docsCommand(scope)
            case .report:
                NativeCLI.reportCommand(scope)
            case .diagnostic:
                NativeCLI.diagnosticCommand(scope)
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
            case .uninstall, .disable, .remove:
                scriptName = "uninstall_spec"
            case .star:
                scriptName = "contribute"
            case .share:
                scriptName = "tweet"
            case .ssh:
                scriptName = "ssh"
            default:
                break;
            }
            
            let script = scriptName ?? self.rawValue.split(separator: ":").joined(separator: "-")
            if let scriptPath = Bundle.main.path(forResource: script,
                                                 ofType: "sh") {
                NativeCLI.runShellScriptInTerminal(scriptPath, with: scope)
                Logger.log(message: "CLI: \(scriptPath)")
            } else {
                Logger.log(message: "CLI: Failed to find script")
               
            }
        }
    }
    
    static func route(_ command: Command, with message: ShellMessage, from connection: WebSocketConnection) {
        let scope = (message, connection)
        
        DispatchQueue.main.async {
            if !Accessibility.enabled {
                printAccessibilityWarning(scope)
            }
          
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
    static func printAccessibilityWarning(_ scope: Scope) {
      let (_, connection) = scope

      NativeCLI.printInTerminal("›  Fig does not have Accessibility Permissions enabled.", using: connection)
    }
  
    static func versionCommand(_ scope: Scope)  {
        let (_, connection) = scope
        let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "-1"
        NativeCLI.printInTerminal(version, using: connection)
    }
    
    static func logoutCommand(_ scope: Scope) {
        let (_, connection) = scope
        let domain = Bundle.main.bundleIdentifier!
        let uuid = Defaults.uuid
        UserDefaults.standard.removePersistentDomain(forName: domain)
        UserDefaults.standard.removePersistentDomain(forName: "\(domain).shared")

        UserDefaults.standard.synchronize()
                
        UserDefaults.standard.set(uuid, forKey: "uuid")
        UserDefaults.standard.synchronize()
        
        WebView.deleteCache()


        let _ = """
        grep -q 'FIG_LOGGED_IN' ~/.fig/user/config || echo "\nFIG_LOGGED_IN=0" >> ~/.fig/user/config;

        sed -i '' "s/FIG_LOGGED_IN=.*/FIG_LOGGED_IN=0/g" ~/.fig/user/config 2> /dev/null
        """.runAsCommand()
        
        printInTerminal("→ Logging out of Fig...", using: connection)
        connection.send(message: "disconnect")

        if let delegate = NSApp.delegate as? AppDelegate {
            delegate.restart()
        }
        
    }
    
    static func restartCommand(_ scope: Scope) {
        let (_, connection) = scope

        printInTerminal("→ Restarting Fig...", using: connection)
        connection.send(message: "disconnect")

        if let delegate = NSApp.delegate as? AppDelegate {
            delegate.restart()
        }
        
    }
    
    static func accessibilityCommand(_ scope: Scope) {
        Accessibility.promptForPermission()
    }
    
    static func resetCacheCommand(_ scope: Scope) {
        let (_, connection) = scope

        WebView.deleteCache()
        NativeCLI.printInTerminal("→ Resetting WebKit Cache...", using: connection)
    }
    
    static func openMenuBarCommand(_ scope: Scope) {
        let (_, connection) = scope
        connection.send(message: "disconnect")

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
  
    static func diagnosticCommand(_ scope: Scope) {
        let (_
      , connection) = scope
        NativeCLI.printInTerminal(Diagnostic.summary, using: connection)
    }
    
    static func reportCommand(_ scope: Scope) {
        let (message, connection) = scope

        NativeCLI.printInTerminal("→ Send any bugs or feedback directly to the Fig team!", using: connection)
        connection.send(message: "disconnect")
        let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "-1"
                
        let tracked = KeypressProvider.shared.buffers.keys.map { (hash) -> (TTY?, String?) in
            let proc = ShellHookManager.shared.tty(for: hash)
            let buffer = KeypressProvider.shared.keyBuffer(for: hash).representation
            return (proc, buffer)
        }
        let env = message.env?.jsonStringToDict()
        let path = env?["PATH"] as? String
        let figIntegratedWithShell = env?["FIG_ENV_VAR"] as? String
  

        let placeholder =
        """
        \(message.arguments.joined(separator: " "))
        
        \(message.data)
        

        
        
        
        
        
        
        
        
        ---------------------------------------
        DEFAULTS:
        Version:\(version)
        SSH Integration:\(Defaults.SSHIntegrationEnabled)
        iTerm Tab Integration:\(iTermTabIntegration.isInstalled())
        Only insert on tab:\(Defaults.onlyInsertOnTab)
        Autocomplete:\(Defaults.useAutocomplete)
        Usershell:\(Defaults.userShell)
        ---------------------------------------
        ENVIRONMENT:
        CLI installed:\(Diagnostic.installedCLI)
        Number of specs: \(Diagnostic.numberOfCompletionSpecs)
        Accessibility: \(Accessibility.enabled)
        SecureKeyboardInput: \(Diagnostic.secureKeyboardInput)
        SecureKeyboardProcess: \(Diagnostic.blockingProcess ?? "<none>")
        PATH: \(path ?? "Not found")
        FIG_ENV_VAR: \(figIntegratedWithShell ?? "Not found")
        --------------------------------------
        CONFIG
        \(Diagnostic.userConfig ?? "?")
        """
        
        /*
         --------------------------------------
         TERMINAL KEYBUFFERS:
         \(tracked.map({ (pair) -> String in
             let (proc, buffer) = pair
         return "\(proc?.cmd ?? "(???)")  \(proc?.cwd ?? "?"): \(buffer ?? "<no context>")"
         }).joined(separator: "\n"))
         */
        
        Feedback.getFeedback(source: "fig_report_cli", placeholder: placeholder)

    }
    
    static func docsCommand(_ scope: Scope) {
        let (_, connection) = scope

        if let delegate = NSApp.delegate as? AppDelegate {
            delegate.viewDocs()
        }

        NativeCLI.printInTerminal("→ Opening docs in browser...", using: connection)
    }
}

extension NativeCLI {
//    static let scriptsFolder =
    static func runScript(_ command: Command, scriptName: String? = nil, with message: ShellMessage, from connection: WebSocketConnection) {
        
        let script = "\(scriptName ?? command.rawValue)"
        if let scriptPath = Bundle.main.path(forResource: script, ofType: "sh") {
            connection.send(message: "bash \(scriptPath)")
        } else {
            Logger.log(message: "Script does not exisr for command '\(command.rawValue)'")
        }
        
        
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
        
        let obfuscatedArgs = message.arguments.map { TelemetryProvider.obscure($0) }.joined(separator: " ")
        
        TelemetryProvider.track(event: .ranCommand,
                                with:   [
                                        "command" : message.subcommand ?? "",
                                        "arguments" : obfuscatedArgs,
                                        "shell" : message.shell ?? "<unknown>",
                                        "terminal" : message.terminal ?? "<unknown>"
                                        ])
    }
    
}
