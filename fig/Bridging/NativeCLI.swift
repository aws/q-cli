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
        case logout = "logout"
        case logoutLegacy = "util:logout"
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
        case quit = "quit"
        case ssh = "integrations:ssh"
        case vscode = "integrations:vscode"
        case iterm = "integrations:iterm"
        case hyper = "integrations:hyper"
        case teamUpload = "team:upload"
        case teamDownload = "team:download"
        case diagnostic = "diagnostic"
        case pty = "debug:pty"
        case debugApp = "debug:app"
        case debugSSH = "debug:ssh"
        case debugSSHSession = "debug:ssh-session"
        case debugProcesses = "debug:ps"
        case debugDotfiles = "debug:dotfiles"
        case electronAccessibility = "util:axelectron"
        case restartSettingsListener = "settings:init"
        case openSettingsFile = "settings:open"
        case runInstallScript = "util:install-script"
        case lockscreen = "util:lockscreen"

        var isUtility: Bool {
            get {
                let utilities: Set<Command> = [.resetCache, .build, .logoutLegacy, .restart, .accessibility]
               return utilities.contains(self)
            }
        }
      
        var handlesDisconnect: Bool {
            get {
              let handlesDisconnection: Set<Command> = [.pty, .hyper, .iterm, .vscode, .quit ]
                return handlesDisconnection.contains(self)
            }
        }

        var implementatedNatively: Bool {
            get {
                let implementatedNatively: Set<Command> = [.resetCache,
                                                           .build,
                                                           .logout,
                                                           .logoutLegacy,
                                                           .restart,
                                                           .accessibility,
                                                           .openMenuBar,
                                                           .onboarding,
                                                           .version,
                                                           .report,
                                                           .diagnostic,
                                                           .vscode,
                                                           .iterm,
                                                           .hyper,
                                                           .pty,
                                                           .debugApp,
                                                           .debugProcesses,
                                                           .debugDotfiles,
                                                           .debugSSHSession,
                                                           .electronAccessibility,
                                                           .issue,
                                                           .restartSettingsListener,
                                                           .runInstallScript,
                                                           .lockscreen,
                                                           .quit,
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
            case .logout, .logoutLegacy:
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
            case .pty:
                NativeCLI.ptyCommand(scope)
            case .vscode:
                NativeCLI.VSCodeCommand(scope)
            case .iterm:
                NativeCLI.iTermCommand(scope)
            case .hyper:
                NativeCLI.HyperCommand(scope)
            case .debugApp:
                NativeCLI.debugAppCommand(scope)
            case .electronAccessibility:
                NativeCLI.electronAccessibilityCommand(scope)
            case .debugProcesses:
                NativeCLI.debugProcessCommand(scope)
            case .debugDotfiles:
                NativeCLI.debugDotfilesCommand(scope)
            case .debugSSHSession:
                NativeCLI.debugSSHSessionCommand(scope)
            case .quit:
                NativeCLI.quitCommand(scope)
            case .issue:
                NativeCLI.issueCommand(scope)
            case .restartSettingsListener:
                NativeCLI.initSettingsCommand(scope)
            case .runInstallScript:
                NativeCLI.runInstallScriptCommand(scope)
            case .lockscreen:
                NativeCLI.lockscreenCommand(scope)
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
            
            if (!command.handlesDisconnect) {
                connection.send(message: "disconnect")
            }
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
        if NSWorkspace.shared.urlForApplication(withBundleIdentifier: "com.surteesstudios.Bartender") != nil {
            NativeCLI.printInTerminal("\nLooks like you might be using Bartender?\n\n→ Fig can't automatically open the menu, but you can click it manually.\n", using: connection)
        }

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
        let (_, connection) = scope
        NativeCLI.printInTerminal(Diagnostic.summary, using: connection)
    }
  
    static func quitCommand(_ scope: Scope) {
        let (_, connection) = scope
        NativeCLI.printInTerminal("\nQuitting Fig...\n", using: connection)
        connection.send(message: "disconnect")
        NSApp.terminate(nil)
    }
    
    static func ptyCommand(_ scope: Scope) {
        let (message, connection) = scope
//        message.
        let command = message.arguments.joined(separator: " ")
        let pty = PseudoTerminalHelper()
        pty.start(with: [:])
        pty.execute(command) { (out) in
          NativeCLI.printInTerminal(out, using: connection)
          pty.close()
          connection.send(message: "disconnect")
        }

    }
    
    static func reportCommand(_ scope: Scope) {
        let (message, connection) = scope

        NativeCLI.printInTerminal("→ Send any bugs or feedback directly to the Fig team!", using: connection)
        connection.send(message: "disconnect")
        // let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "-1"
                
        // let tracked = KeypressProvider.shared.buffers.keys.map { (hash) -> (TTY?, String?) in
        //    let proc = ShellHookManager.shared.tty(for: hash)
        //    let buffer = KeypressProvider.shared.keyBuffer(for: hash).representation
        //    return (proc, buffer)
        // }
        let env = message.env?.jsonStringToDict()
        let path = env?["PATH"] as? String
        let figIntegratedWithShell = env?["FIG_ENV_VAR"] as? String

        let placeholder =
        """
        \(message.arguments.joined(separator: " "))
        
        \(message.data)
        

        
        
        
        
        
        
        
        
        ---------------------------------------
        DIAGNOSTIC
        \(Diagnostic.summary)
        ---------------------------------------
        ENVIRONMENT
        Terminal: \(message.terminal ?? "<unknown>")
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
  
    static func VSCodeCommand(_ scope: Scope) {
        let (_, connection) = scope

        if VSCodeIntegration.isInstalled {
            NativeCLI.printInTerminal("\n› VSCode Integration is already installed.\n  You may need to restart VSCode for the changes to take effect.\n  If you are having issues, please use fig report.\n", using: connection)
            connection.send(message: "disconnect")
        } else {
            NativeCLI.printInTerminal("→ Prompting VSCode Integration...", using: connection)
            connection.send(message: "disconnect")

            VSCodeIntegration.promptToInstall()
        }

    }
  
    static func iTermCommand(_ scope: Scope) {
        let (_, connection) = scope

        if iTermTabIntegration.isInstalled {
            NativeCLI.printInTerminal("\n› iTerm Tab Integration is already installed.\n  If you are having issues, please use fig report.\n", using: connection)
            connection.send(message: "disconnect")

        } else {
            NativeCLI.printInTerminal("→ Prompting iTerm Tab Integration...", using: connection)
            connection.send(message: "disconnect")
            iTermTabIntegration.promptToInstall()
        }

    }
  
    static func HyperCommand(_ scope: Scope) {
        let (_, connection) = scope

        if HyperIntegration.isInstalled {
            NativeCLI.printInTerminal("\n› Hyper Integration is already installed.\n  You may need to restart Hyper for the changes to take effect.\n  If you are having issues, please use fig report.\n", using: connection)
            connection.send(message: "disconnect")
        } else {
            NativeCLI.printInTerminal("→ Prompting Hyper Integration...", using: connection)
            connection.send(message: "disconnect")
            HyperIntegration.promptToInstall()
        }

    }
  
    static func initSettingsCommand(_ scope: Scope) {
        let (_, connection) = scope
        NativeCLI.printInTerminal("\n› Restarting ~/.fig/settings.json file watcher.. \n", using: connection)
        Settings.shared.restartListener()
    }
  
    static func debugAppCommand(_ scope: Scope) {
        let (_, connection) = scope

        NativeCLI.printInTerminal("\n› Run Fig from executable to view logs...\n\n  \(Bundle.main.executablePath ?? "")\n\n  Make sure to Quit the existing instance\n  of Fig before running this command.\n", using: connection)
    }
  
    static func issueCommand(_ scope: Scope) {
        let (message, connection) = scope
        NativeCLI.printInTerminal("\n→ Opening Github...\n", using: connection)

        Github.openIssue(with: message.arguments.joined(separator: " "))
      
    }
  
    static func runInstallScriptCommand(_ scope: Scope) {
      let (_, connection) = scope

      NativeCLI.printInTerminal("\n› Running installation script...\n", using: connection)
      Onboarding.setUpEnviroment()
    }
  
    static func lockscreenCommand(_ scope: Scope) {
      let (_, connection) = scope

      NativeCLI.printInTerminal("\n→ Locking screen...\n  This may resolve issues with Secure Keyboard Entry\n", using: connection)
      SecureKeyboardInput.lockscreen()

    }
  
    static func electronAccessibilityCommand(_ scope: Scope) {
        let (_, connection) = scope
        if let app = AXWindowServer.shared.topApplication, let name = app.localizedName {
          NativeCLI.printInTerminal("\n› Enabling DOM Accessibility in '\(name)'...\n", using: connection)
          Accessibility.triggerScreenReaderModeInChromiumApplication(app)
        } else {
          NativeCLI.printInTerminal("\n› Could not find Electron app!\n", using: connection)
        }
    }
  
    static func debugProcessCommand(_ scope: Scope) {
        let (message, connection) = scope
      let ps = ProcessStatus.getProcesses(for: String(message.arguments[safe: 0]?.split(separator:"/").last ?? "")).map { return "\($0.pid) \($0.tty ?? "?")   \($0.cmd) \($0._cwd ?? "?")" }.joined(separator: "\n")
      
        
        NativeCLI.printInTerminal(ps, using: connection)
    }
  
    static func debugDotfilesCommand(_ scope: Scope) {
        let (_, connection) = scope
      
        let dotfiles = [".profile", ".bashrc", ".bash_profile", ".zshrc", ".zprofile", ".config/fish/config.fish", ".tmux.conf", ".ssh/config"]

        let print = dotfiles.map({ (path) -> String in
          let fullPath = "\(NSHomeDirectory())/\(path)"
          let exists = FileManager.default.fileExists(atPath: fullPath)
          let symlink = (try? FileManager.default.destinationOfSymbolicLink(atPath: fullPath))
          let contents = try? String(contentsOf: URL(fileURLWithPath:symlink ?? fullPath))
          
          return "\(exists ? (contents?.contains("~/.fig") ?? false ? "✅" : "❌") : "❔") ~/\(path)\(symlink != nil ? " -> \(symlink!)" : "")"
        }).joined(separator: "\n")


        NativeCLI.printInTerminal(print, using: connection)
    }
  
    static func debugSSHSessionCommand(_ scope: Scope) {
        let (_, connection) = scope
      let prefixes: [String] = ShellHookManager.shared.ttys().values.map { (tty) -> String? in
          guard let ssh = tty.integrations[SSHIntegration.command] as? SSHIntegration else {
            return nil
          }
          
          guard let controlPath = ssh.runUsingPrefix() else {
            return nil
          }
          
          return controlPath + "\n"
      }.filter { $0 != nil } as! [String]
      
      let remote_cwd_script = SSHIntegration.pathToRemoteWorkingDirectoryScript()
      
      let out =
      """
        
      Run commands in SSH Session using PREFIX:
      ---
      \(prefixes.count == 0 ? "no sessions found..." : prefixes.joined(separator: "\n"))
      ---
      
      Remove -q and add -v flags for additional logging.
      
      To simulate getting remote working directory run:
      
      PREFIX bash -s < \(remote_cwd_script)
      
      """
      
      NativeCLI.printInTerminal(out, using: connection)
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
