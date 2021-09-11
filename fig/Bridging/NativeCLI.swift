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
        case updateApp = "update:app"
        case source = "source"
        case resetCache = "util:reset-cache"
        case tools = "tools"
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
        case debugWindows = "debug:windows"
        case electronAccessibility = "util:axelectron"
        case openSettingsDocs = "settings:docs"
        case openSettings = "settings"
        case restartSettingsListener = "settings:init"
        case openSettingsFile = "settings:open"
        case runInstallScript = "util:install-script"
        case lockscreen = "util:lockscreen"
        case setPATH = "set:path"
        case community = "community"
        case chat = "chat"
        case discord = "discord"
        case viewLogs = "debug:log"
        case symlinkCLI = "util:symlink-cli"
        case loginItems = "util:login-items"
        case theme = "theme"

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
                                                           .tools,
                                                           .pty,
                                                           .debugApp,
                                                           .debugProcesses,
                                                           .debugDotfiles,
                                                           .debugSSHSession,
                                                           .debugWindows,
                                                           .electronAccessibility,
                                                           .issue,
                                                           .restartSettingsListener,
                                                           .runInstallScript,
                                                           .lockscreen,
                                                           .openSettings,
                                                           .quit,
                                                           .viewLogs,
                                                           .updateApp,
                                                           .symlinkCLI,
                                                           .loginItems,
                                                           .theme,
                                                           .docs]
               return implementatedNatively.contains(self)
            }
        }
        

        func run(_ scope: Scope) {
            guard self.implementatedNatively else {
              Logger.log(message: "CLI function '\(self.rawValue)' not implemented natively", subsystem: .cli)
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
            case .tools:
                NativeCLI.toolsCommand(scope)
            case .debugWindows:
                NativeCLI.debugWindowsCommand(scope)
            case .viewLogs:
                NativeCLI.debugLogsCommand(scope)
            case .openSettings:
                NativeCLI.openSettingsCommand(scope)
            case .updateApp:
              NativeCLI.updateAppCommand(scope)
            case .symlinkCLI:
              NativeCLI.symlinkCLICommand(scope)
            case .loginItems:
              NativeCLI.updateLoginItemCommand(scope)
            case .theme:
              NativeCLI.themeCommand(scope)
            default:
                break;
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
        let (message, connection) = scope
        let env = message.env?.jsonStringToDict() ?? [:]
        NativeCLI.printInTerminal(Diagnostic.summaryWithEnvironment(env), using: connection)
    }
  
    static func symlinkCLICommand(_ scope: Scope) {
        let (_, connection) = scope
        NativeCLI.printInTerminal("Symlinking CLI to ~/.fig/bin/fig...", using: connection)
        Onboarding.copyFigCLIExecutable(to:"~/.fig/bin/fig")
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
  
    static func toolsCommand(_ scope: Scope) {
        let folder = "\(NSHomeDirectory())/.fig/tools"
        let (message, connection) = scope
      switch message.arguments.count {
        case 0:
          let files = (try? FileManager.default.contentsOfDirectory(atPath: folder)) ?? []
          let out = files.map { (str) -> String in
            guard let name = str.split(separator: ".").first else {
              return str
            }
            return String(name)
          }.joined(separator: "\n")

          NativeCLI.printInTerminal(out, using: connection)

        case 1:
          let path = message.arguments.first!
          let fullPathIncludingExtension = folder + "/" + path + ".sh"
          let fullPathWithoutExtension =  folder + "/" + path
          if FileManager.default.fileExists(atPath: fullPathIncludingExtension) {
            NativeCLI.runInTerminal(command: "bash \(fullPathIncludingExtension)", over: connection)
          } else if FileManager.default.fileExists(atPath: fullPathWithoutExtension) {
            NativeCLI.runInTerminal(command: "bash \(fullPathWithoutExtension)", over: connection)
          } else {
            NativeCLI.printInTerminal("\nNo matching script found...\n", using: connection)
          }
          
          break;
        default:
          NativeCLI.printInTerminal("\nToo many arguments. Expects 0 or 1.\n", using: connection)
          return
      }
      
      
    }
  
    static func debugWindowsCommand(_ scope: Scope) {
        let (_, connection) = scope
      let window2tty = ShellHookManager.shared.ttys()
      
      let matchedWindowHashes = window2tty.keys.map{ "\($0) - \(window2tty[$0]?.descriptor ?? "???") (\(window2tty[$0]?.pid ?? 0))" }.joined(separator: "\n")
      let current = AXWindowServer.shared.whitelistedWindow
      
      let out =
      """
      \(current?.hash ?? "???") - \(current?.tty?.descriptor ?? "???") (\(current?.tty?.pid ?? 0))
      ---
      \(matchedWindowHashes)
      """
      NativeCLI.printInTerminal(out, using: connection)
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

        if VSCodeIntegration.default.isInstalled {
            NativeCLI.printInTerminal("\n› VSCode Integration is already installed.\n  You may need to restart VSCode for the changes to take effect.\n  If you are having issues, please use fig report.\n", using: connection)
            connection.send(message: "disconnect")
        } else {
            NativeCLI.printInTerminal("→ Prompting VSCode Integration...", using: connection)
            connection.send(message: "disconnect")

            VSCodeIntegration.default.promptToInstall()
        }

    }
  
    static func iTermCommand(_ scope: Scope) {
        let (_, connection) = scope

        if iTermIntegration.default.isInstalled {
            NativeCLI.printInTerminal("\n› iTerm Integration is already installed.\n  If you are having issues, please use fig report.\n", using: connection)
            connection.send(message: "disconnect")

        } else {
            NativeCLI.printInTerminal("→ Prompting iTerm Integration...", using: connection)
            connection.send(message: "disconnect")
            iTermIntegration.default.promptToInstall()
        }

    }
  
    static func HyperCommand(_ scope: Scope) {
        let (_, connection) = scope

        if HyperIntegration.default.isInstalled {
            NativeCLI.printInTerminal("\n› Hyper Integration is already installed.\n  You may need to restart Hyper for the changes to take effect.\n  If you are having issues, please use fig report.\n", using: connection)
            connection.send(message: "disconnect")
        } else {
            NativeCLI.printInTerminal("→ Prompting Hyper Integration...", using: connection)
            connection.send(message: "disconnect")
            HyperIntegration.default.promptToInstall()
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
    
    static func themeCommand(_ scope: Scope) {
        let (message, connection) = scope
        
        guard message.arguments.count == 1 else {
            let theme = Settings.shared.getValue(forKey: Settings.theme) as? String ?? "dark"
            NativeCLI.printInTerminal("\(theme)", using: connection)
            return
        }
        
        let builtins: Set<String> = ["dark", "light"]
        if let themeName = message.arguments.first {
            let pathToTheme = NSHomeDirectory() + "/.fig/themes/\(themeName).json"
            
            if FileManager.default.fileExists(atPath: pathToTheme),
               let data = try? Data(contentsOf: URL(fileURLWithPath: pathToTheme)),
               let theme = try? JSONSerialization.jsonObject(with: data, options: []) as? [String: Any] {
                
                var byLine: String? = nil
                var twitterLine: String? = nil
                var githubLine: String? = nil

                if let author = theme["author"] as? [String: String],
                   let name = author["name"] {
                    byLine = " by " + name
                    
                    if let handle = author["twitter"] {
                        twitterLine = "  🐦 \u{001b}[0;96m\(handle)\u{001b}[0m\n"
                    }
                    
                    if let handle = author["github"] {
                        githubLine = "  💻 \u{001b}[4mgithub.com/\(handle)\u{001b}[0m\n"
                    }
                }
                
                Settings.shared.set(value: themeName, forKey: Settings.theme)
                
                let text = "\n› Switching to theme '\u{001b}[1m\(themeName)\u{001b}[0m'\(byLine ?? "")\n\n\(twitterLine ?? "")\(githubLine ?? "")"
                NativeCLI.printInTerminal(text, using: connection)

                
            } else if builtins.contains(themeName) {
                let text = "\n› Switching to theme '\u{001b}[1m\(themeName)\u{001b}[0m'\n"
                Settings.shared.set(value: themeName, forKey: Settings.theme)
                NativeCLI.printInTerminal(text, using: connection)

            } else {
                NativeCLI.printInTerminal("'\(themeName)' does not exist. \n", using: connection)

            }
            
        }
      
    }
  
    static func openSettingsCommand(_ scope: Scope) {
      let (_, connection) = scope

      NativeCLI.printInTerminal("\n› Opening settings...\n", using: connection)
      Settings.openUI()
      
    }
  
    static func runInstallScriptCommand(_ scope: Scope) {
      let (_, connection) = scope

      NativeCLI.printInTerminal("\n› Running installation script...\n", using: connection)
      Onboarding.setUpEnviroment()
    }
  
    static func updateLoginItemCommand(_ scope: Scope) {
        let (message, connection) = scope
        let command = message.arguments.first ?? ""
        
        switch command {
            case "--remove":
                LoginItems.shared.currentApplicationShouldLaunchOnStartup = false
                NativeCLI.printInTerminal("\n› Removing Fig from LoginItems\n", using: connection)
            case "--add":
                LoginItems.shared.currentApplicationShouldLaunchOnStartup = true
                NativeCLI.printInTerminal("\n› Adding Fig to LoginItems\n", using: connection)
            case "--remove-all":
                LoginItems.shared.removeAllItemsMatchingBundleURL()
                NativeCLI.printInTerminal("\n› Removing all Fig entries from LoginItems\n", using: connection)
            default:
                NativeCLI.printInTerminal("\(LoginItems.shared.currentApplicationShouldLaunchOnStartup ? "true" : "false")", using: connection)

        }
    }
    
    
    static func lockscreenCommand(_ scope: Scope) {
      let (_, connection) = scope

      NativeCLI.printInTerminal("\n→ Locking screen...\n  This may resolve issues with Secure Keyboard Entry\n", using: connection)
      SecureKeyboardInput.lockscreen()

    }
  
    static func updateAppCommand(_ scope: Scope) {
      let (message, connection) = scope

      if message.arguments.contains("--force") {
        if UpdateService.provider.updateIsAvailable {
          NativeCLI.printInTerminal("\n→ Installing update for macOS app...\n", using: connection)
          DispatchQueue.main.asyncAfter(deadline:.now() + 0.25) {
            UpdateService.provider.installUpdateIfAvailible()
          }
        }

      } else {
        NativeCLI.printInTerminal("\n→ Checking for updates to macOS app...\n", using: connection)
        UpdateService.provider.checkForUpdates(nil)
      }

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
  
    static func debugLogsCommand(_ scope: Scope) {
        let (message, connection) = scope
        let loggingEnabled = Settings.shared.getValue(forKey: Settings.logging) as? Bool ?? false
        guard loggingEnabled else {
          NativeCLI.printInTerminal("'\(Settings.logging)' is not enabled.", using: connection)
          return
        }
        let all = Set((try? FileManager.default.contentsOfDirectory(atPath: Logger.defaultLocation.path)) ?? [])
        let removed = Set(message.arguments.filter { $0.starts(with: "-") }.map { $0.stringByReplacingFirstOccurrenceOfString("-", withString: "") + ".log" })

        var files: String!
        if message.arguments.count == 0 {
          files = all.map { "\(Logger.defaultLocation.path)/\($0)" }.joined(separator: " ")
        } else if removed.count > 0 {
          files = all.subtracting(removed).map { "\(Logger.defaultLocation.path)/\($0)" }.joined(separator: " ")
        } else {
          files = message.arguments.map { "\(NSHomeDirectory())/.fig/logs/\($0).log" }.joined(separator: " ")
        }

        connection.send(message: "execvp:tail -n0 -qf \(files!)")
        
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
          Logger.log(message: "Script does not exist for command '\(command.rawValue)'", subsystem: .cli)
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

extension String {

  func stringByReplacingFirstOccurrenceOfString(_ target: String, withString replaceString: String) -> String {
      if let range = self.range(of: target) {
          return self.replacingCharacters(in: range, with: replaceString)
      }
      return self
  }

}
