//
//  FigJS.swift
//  fig
//
//  Created by Matt Schrage on 5/26/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class Scope {
    let session: String
    let cmd: String
    let env: String
    let stdin: String
    var options: [String]
    let webView: WebView
    let companionWindow: CompanionWindow
    var pwd: String? {
        if let dict = self.env.jsonStringToDict() {
            return dict["PWD"] as? String
        }
        return nil
    }
    
    var shell: String? {
        if let dict = self.env.jsonStringToDict() {
            return dict["SHELL"] as? String
        }
        return nil
    }
    
    var term: String? {
        if let dict = self.env.jsonStringToDict() {
            if let _ = dict["KITTY_WINDOW_ID"] {
                return "kitty"
            }
            
            if let _ = dict["ALACRITTY_LOG"] {
                return "Alacritty"
            }
            
            return dict["TERM_PROGRAM"] as? String
        }
        return nil
    }

    
    init(cmd: String,
         stdin: String,
         options: [String],
         env: String,
         webView: WebView,
         companionWindow: CompanionWindow,
         session: String) {
        self.cmd = cmd
        self.stdin = stdin
        self.options = options
        self.env = env
        self.webView = webView
        self.companionWindow = companionWindow
        self.session = session
        
    }
    
}

enum NativeCLICommand : String {
    case web = "web"
    case local = "local"
    case bundle = "bundle"
    case callback = "callback"
    case hide = "hide"
    case position = "position"
    case apps = "apps"
    case store = "store"
    case appstore = "appstore"
    case blocks = "blocks"
    case home = "home"
    case help = "--help"
    case h = "-h"
    case version = "--version"
    case accesibility = "util:axprompt"
    case logout = "util:logout"
    case restart = "util:restart"
    case build = "util:build"
    case sidebar = "sidebar"
    case close = "close"
    case feedback = "feedback"
    case invite = "invite"
    case docs = "docs"
    case update = "update"
    case source = "source"
    case resetCache = "util:reset-cache"
    case list = "list"
    
    var openInNewWindow: Bool {
        get {
            let popups: Set<NativeCLICommand> = [ .web, .local, .bundle, .apps, .appstore, .home, .appstore, .blocks]
            return popups.contains(self)
        }
    }
}

class FigCLI {
    static let baseURL = Remote.baseURL

    static func index(with scope: Scope ) {
        scope.webView.loadRemoteApp(at: Remote.baseURL)
        FigCLI.env(with: scope)
        FigCLI.options(with: scope)
        FigCLI.stdin(with: scope)
        FigCLI.initialPosition(with: scope)
//        FigCLI.stdin(with: scope)
    }
    
    
    static func form(with scope: Scope, format: String) {
        let url = ShellBridge.aliasToRawURL(format, options: scope.options)
        scope.webView.loadRemoteApp(at: url)
        FigCLI.prepareWebView(with: scope)
    }
    
    static func prepareWebView(with scope: Scope) {
        FigCLI.env(with: scope)
        FigCLI.options(with: scope)
        FigCLI.stdin(with: scope)
        FigCLI.initialPosition(with: scope)
    }
    
    static func callback(with scope: Scope) {
        scope.webView.evaluateJavaScript("fig.\(scope.options[0])(`\(scope.stdin)`)", completionHandler: nil)
    }
    
    static func initialPosition(with scope: Scope) {
        // this causes crashes!
        return;
            
        scope.webView.onLoad.append {
              WebBridge.appInitialPosition(webview: scope.webView) { (position) in
                // there is a crash that occurs here and I don't know why...
                if (scope.companionWindow != nil) {
                    scope.companionWindow.positioning = CompanionWindow.OverlayPositioning(rawValue:                     Int(position ?? "-1")! ) ?? scope.companionWindow.positioning
                }
              }
          }
        
    }
    
    static func local(with scope: Scope) {
        if let path = scope.options[safe: 0], let pwd = scope.pwd  {
            let url = URL(fileURLWithPath: path, relativeTo: URL(fileURLWithPath: pwd))
            scope.webView.loadLocalApp(url)
            FigCLI.env(with: scope)
            FigCLI.options(with: scope, removeFirstOption: true)
            FigCLI.stdin(with: scope)
            FigCLI.initialPosition(with: scope)

        } else {
            let modified = Scope(cmd: "local", stdin: scope.stdin, options: scope.options, env: scope.env, webView: scope.webView, companionWindow: scope.companionWindow, session: scope.session)
            FigCLI.url(with: modified)
        }
        FigCLI.env(with: scope)

    }
    
    static func bundle(with scope: Scope) {
        scope.webView.loadBundleApp(scope.options[0])
        FigCLI.env(with: scope)
        FigCLI.options(with: scope)
        FigCLI.stdin(with: scope)
        FigCLI.initialPosition(with: scope)

    }
    
    static func web(with scope: Scope) {
        guard let url =  URL(string: scope.options[safe: 0] ?? "") else {
            let modified = Scope(cmd: "web", stdin: scope.stdin, options: scope.options, env: scope.env, webView: scope.webView, companionWindow: scope.companionWindow, session: scope.session)
            FigCLI.url(with: modified)
            return
        }
        scope.webView.loadRemoteApp(at: url)
        FigCLI.env(with: scope)
        FigCLI.options(with: scope, removeFirstOption: true)
        FigCLI.stdin(with: scope)
//        FigCLI.initialPosition(with: scope)
        
        if (Defaults.automaticallyLaunchWebAppsInDetachedWindow) {
            scope.companionWindow.untether()
            scope.companionWindow.makeKey()
        }
    }
    
    static func position(with scope: Scope) {
        let positionValue = Int(scope.options[0]) ?? -1
        scope.companionWindow.positioning = CompanionWindow.OverlayPositioning(rawValue: positionValue) ?? .insideRightPartial
        scope.companionWindow.repositionWindow(forceUpdate: true)
        FigCLI.env(with: scope)

    }

    static func hide(with scope: Scope) {
        scope.companionWindow.positioning = CompanionWindow.defaultPassivePosition
    }
    
    static func options(with scope: Scope, removeFirstOption: Bool = false) {
        if (removeFirstOption && scope.options.count > 0) {
            scope.options.removeFirst()
        }
        scope.webView.onLoad.append {
            let opts = scope.options
            if let json = try? JSONSerialization.data(withJSONObject: opts, options: .fragmentsAllowed) {
                scope.webView.evaluateJavaScript("fig.options = JSON.parse(b64DecodeUnicode(`\(json.base64EncodedString())`))", completionHandler: nil)
                  
            }
        }
    }
    
    static func env(with scope: Scope) {
        scope.webView.configureEnvOnLoad = {
            
            let env = scope.env.jsonStringToDict() ?? FigCLI.extract(keys: ["PWD","USER","HOME","SHELL", "OLDPWD", "TERM_PROGRAM", "TERM_SESSION_ID", "HISTFILE","FIG","FIGPATH"], from: scope.env)
            if let json = try? JSONSerialization.data(withJSONObject: env, options: .fragmentsAllowed) {
                scope.webView.evaluateJavaScript("fig.env = JSON.parse(b64DecodeUnicode(`\(json.base64EncodedString())`))", completionHandler: nil)
                  
            }
        }
    }
    
    static func extract(keys:[String], from env: String) -> [String: String] {
        var out: [String: String]  = [:]
        for key in keys {
            if let group = env.groups(for: "\"\(key)\": \"(.*?)\",").first, let value = group.last {
                  out[key] = value
              }
        }
        
        return out
    }
    
    static func entry(with scope: Scope) {
        scope.webView.onLoad.append {
            scope.webView.evaluateJavaScript("fig.callinit()", completionHandler: nil)
        }
    }
    static func stdin(with scope: Scope) {
        scope.webView.onLoad.append {
            print(scope.stdin.applyingTransform(StringTransform.toXMLHex, reverse: false)!)
            print(scope.stdin.applyingTransform(StringTransform.toUnicodeName, reverse: false)!)
            print(scope.stdin.applyingTransform(StringTransform.toLatin, reverse: false)!)
            print(scope.stdin.data(using: .utf8)!)
            let encoded = scope.stdin.data(using: .utf8)!
            scope.webView.evaluateJavaScript("fig.stdinb64(`\(encoded.base64EncodedString())`)", completionHandler: nil)
        }
    }
    
//    static func run() {}

    static func run(scope: Scope, path: String? = nil) {
        let modified = Scope(cmd: "run",
                             stdin: scope.stdin,
                             options: path != nil ? [path!] + scope.options : scope.options,
                             env: scope.env,
                             webView: scope.webView,
                             companionWindow: scope.companionWindow,
                             session: scope.session)
        FigCLI.url(with: modified)
    }
    static func openHelp(schema: CLICommandSchema, scope: Scope) {
        let markup = schema.text
        //markup.
        do {
            let path = WebBridge.appDirectory.appendingPathComponent("tmp/help.run")
            try markup.write(to: path, atomically: true, encoding: String.Encoding.utf8)
            scope.options = [path.deletingPathExtension().path]
            FigCLI.run(scope: scope)
            // this can be done without running the command again
//            print("fig run \(path)")
//            Timer.delayWithSeconds(1) {
//                ShellBridge.injectStringIntoTerminal("fig run \(path.deletingPathExtension().path)", runImmediately: true, completion: nil)
//            }
        } catch {
            print("error writing file")
        }
      
    }
    
    static func dotfig(filePath: String, scope: Scope) {
        let url = URL(fileURLWithPath: filePath)
        do {
//            let out = try String(contentsOf: url, encoding: String.Encoding.utf8)
            let data = try Data(contentsOf:url)
            // parse as JSON
            let decoder = JSONDecoder()

            guard let schema = try? decoder.decode(CLICommandSchema.self, from: data) else {
                return
            }
            
            let (flags, seq, mismatch) = try! FigCLI.traverseCLI(options: scope.options, subcommands: schema.children)
            
            guard let cmd = seq.last else {
                // open schema --help
                return openHelp(schema: schema, scope: scope)
            }
            
            guard !mismatch else {
                //open cmd --help
                return openHelp(schema: cmd, scope: scope)

            }
            
            if (flags.count == 0 && !(cmd.runWithNoInput ?? false)) {
                // open cmd --help
                return openHelp(schema: cmd, scope: scope)
            }
            
            
            if (flags.contains("--help") || flags.contains("-h") || flags.contains("--runbook")) {
                // open cmd --help
                return openHelp(schema: cmd, scope: scope)
            }
            
            // i have no idea why but if this isn't called, there is a memory leak that causes a crash when the window is closed
            FigCLI.env(with: scope)

            scope.companionWindow.windowManager.close(window:  scope.companionWindow)
            //run command on behalf of user
            let script =  "\(cmd.script!) \(flags.joined(separator: " "))"
            ShellBridge.shared.socketServer.send(sessionId: scope.session ?? "", command: script)
//            Timer.delayWithSeconds(0.15) {
//                ShellBridge.injectStringIntoTerminal(script, runImmediately: true, completion: nil)
//            }
//            "\(cmd.cmdToRun!) \(flags.joined(separator: " "))".runAsCommand(cwd: scope.pwd, with: scope.env.jsonStringToDict() as? Dictionary<String, String>)
            
        } catch {
            
        }
    }
    
    static func runInTerminal(script: String, scope: Scope) {
        // i have no idea why but if this isn't called, there is a memory leak that causes a crash when the window is closed
        FigCLI.env(with: scope)
        
        scope.companionWindow.windowManager.close(window:  scope.companionWindow)

        ShellBridge.shared.socketServer.send(sessionId: scope.session, command: "\(script) \(scope.options.map { $0.contains(" ") ? "\"\($0)\"" : $0 } .joined(separator: " "))")
    }
    
    static func printInTerminal(text: String, scope: Scope) {
        var modified = Scope(cmd: scope.cmd,
                             stdin: scope.stdin,
                             options: [],
                             env: scope.env,
                             webView: scope.webView,
                             companionWindow: scope.companionWindow,
                             session: scope.session)
        FigCLI.runInTerminal(script: "echo \"\(text)\"", scope: modified)
    }
    
    struct CLICommandSchema: Codable {
        var title: String
        var command: String
        var text: String
        var children: [CLICommandSchema]
        var script: String?
        var runWithNoInput: Bool?
    }


    enum CLICommandSchemaError: Error {
        case duplicatedSubcommand
    }
    
    static func traverseCLI(options: [String], subcommands: [CLICommandSchema]) throws -> ([String], [CLICommandSchema], Bool){
        guard let cmd = options.first else {
            return ([], [], false)
        }
        
        guard subcommands.count > 0 else {
            return (options, [], false)
        }
        
        let match = subcommands.filter { $0.command == cmd }
        
        guard match.count <= 1 else {
            print("CLI parsing error: Subcommands must be unique")
            throw CLICommandSchemaError.duplicatedSubcommand
        }
        
        if match.count == 1, let next = match.first {
            let (opts, cmds, mismatch) = try! traverseCLI(options: Array(options.dropFirst()), subcommands: next.children)
            return (opts, [next] + cmds, mismatch)
        }
        
        return (options, [], true)
        
    }
    
    static func url(with scope: Scope) {
        var all = scope.options
        all.insert(scope.cmd, at: 0)
        let url = ShellBridge.commandLineOptionsToRawURL(all)
        scope.webView.loadRemoteApp(at: url)
        FigCLI.env(with: scope)
        FigCLI.options(with: scope)
        FigCLI.stdin(with: scope)
    }
    
    static func storedInitialPosition(for command: String) -> CompanionWindow.OverlayPositioning? {
        guard let initial = UserDefaults.standard.string(forKey: "\(command):position") else {
            return nil
        }
        
        guard let raw = Int(initial) else {
            return nil
        }
        
        return CompanionWindow.OverlayPositioning(rawValue: raw)
    }
    
    static func notifyAccessibilityError(_ message: ShellMessage) {
        let checkOptPrompt = kAXTrustedCheckOptionPrompt.takeUnretainedValue() as NSString
               //set the options: false means it wont ask
               //true means it will popup and ask
               let opt = [checkOptPrompt: false]
               //translate into boolean value
               let accessEnabled = AXIsProcessTrustedWithOptions(opt as CFDictionary?)
               guard accessEnabled else {
                   
                   let error =
                   """

                   › \u{001b}[31mCould not open fig.app.\u{001b}[0m

                     \u{001b}[1mQUICK FIX\u{001b}[0m
                     Fig does not have Accessibility Permissions enabled.

                   → Click the Fig icon in your menu bar \u{001b}[1mDebug > Request Accessibility Permission\u{001b}[0m

                     Please email \u{001b}[1mhello@withfig.com\u{001b}[0m if this problem persists.

                   """
                   //            → Run \u{001b}[1mopen -b com.mschrage.fig\u{001b}[0m to prompt for access.

                   ShellBridge.shared.socketServer.send(sessionId: message.session, command: "echo \"\(error)\"")
                   ShellBridge.shared.socketServer.send(sessionId: message.session, command: "disconnect")
                   
                   return
               }
    }
    
    static func route(_ message: ShellMessage, webView: WebView, companionWindow: CompanionWindow) {
        
        let stdin = message.data.replacingOccurrences(of: "`", with: "\\`")
        let env = message.env ?? ""
        //webView.clearHistory()
        webView.window?.representedURL = nil        
      
        guard let options = message.options, options.count > 0 else {
            let scope = Scope(cmd: "", stdin: stdin, options: [], env: env, webView: webView, companionWindow: companionWindow, session: message.session)
            FigCLI.env(with: scope)

            ShellBridge.shared.socketServer.send(sessionId: scope.session, command: "disconnect")
            scope.companionWindow.windowManager.close(window:  scope.companionWindow)

            if let delegate = NSApp.delegate as? AppDelegate {
                delegate.openMenu()
            }
            
//            companionWindow.positioning =  .spotlight
//            FigCLI.index(with: scope)
            TelemetryProvider.post(event: .ranCommand, with:
                                    ["cmd" : scope.cmd,
                                    "args" : scope.options.map { TelemetryProvider.obscure($0)}.joined(separator: " "),
                                    "shell" : scope.shell ?? "<unknown>",
                                    "terminal" : scope.term ?? "<unknown>"])
            return
        }
        let command = options.first!
        let flags = Array(options.suffix(from: 1))
        let scope = Scope(cmd: command,
                          stdin: stdin,
                          options: flags,
                          env: env,
                          webView: webView,
                          companionWindow: companionWindow,
                          session: message.session)
        print("ROUTING \(command)")
        
        TelemetryProvider.post(event: .ranCommand, with: ["cmd" : scope.cmd, "args" : scope.options.map { TelemetryProvider.obscure($0)}.joined(separator: " "), "shell" : scope.shell ?? "<unknown>", "terminal" : scope.term ?? "<unknown>"])
        
        // get aliases from sidebar
        let aliases = UserDefaults.standard.string(forKey: "aliases_dict")?.jsonStringToDict() ?? [:]
        
        // get
        var figPath = (scope.env.jsonStringToDict()?["FIGPATH"] as? String)?.split(separator: ":").map { String($0) } ?? []
        
        if (!figPath.contains("~/run")) {
            figPath.insert("~/run", at: 0)
        }
        
        if (!figPath.contains("~/.fig/bin")) {
            figPath.insert("~/.fig/bin", at: 0)
        }
        
        figPath = figPath.map { NSString(string: String($0)).standardizingPath }
        
        var isRundown = false
        var isCLI = false
        var isExecutable = false

        var appPath: String? = nil
        for prefix in figPath {
            isCLI = FileManager.default.fileExists(atPath: "\(prefix)/\(command).fig")
            if (isCLI) {
                appPath = "\(prefix)/\(command).fig"
                break
            }
    
            let withPathExtension = FileManager.default.fileExists(atPath: "\(prefix)/\(command).html")
            if (withPathExtension) {
                appPath = "\(prefix)/\(command).html"
                break
            }
            
            var isDir : ObjCBool = false
            let isDirectory = FileManager.default.fileExists(atPath: "\(prefix)/\(command)", isDirectory: &isDir)
            if (isDirectory) {
                appPath = "\(prefix)/\(command)/index.html"
                break
            }
            
            let isRaw = FileManager.default.fileExists(atPath: "\(prefix)/\(command)")
            if (isRaw) {
                appPath = "\(prefix)/\(command)"
                break
            }
            
            isRundown = FileManager.default.fileExists(atPath: "\(prefix)/\(command).run")
            if (isRundown) {
                appPath = "\(prefix)/\(command)"
                break
            }
            isExecutable = FileManager.default.fileExists(atPath: "\(prefix)/\(command).sh")
            if (isExecutable) {
                appPath = "\(prefix)/\(command).sh"
                break
            }
            
            isExecutable = FileManager.default.fileExists(atPath: "\(prefix)/\(command).py")
            if (isExecutable) {
                 appPath = "\(prefix)/\(command).py"
                 break
            }
        }
        
        //companionWindow.positioning = storedInitialPosition(for: command) ?? CompanionWindow.defaultActivePosition
        companionWindow.positioning = CompanionWindow.defaultActivePosition

        if let nativeCommand = NativeCLICommand(rawValue: command) {
            switch nativeCommand {
            case .docs:
                if let delegate = NSApp.delegate as? AppDelegate {
                    delegate.viewDocs()
                }
                FigCLI.printInTerminal(text: "→ Opening docs in browser...", scope: scope)
            case .callback:
                FigCLI.callback(with: scope)
            case .bundle:
                FigCLI.bundle(with: scope)
            case .local:
                FigCLI.local(with: scope)
            case .web:
                companionWindow.positioning = .fullscreenInset
                scope.webView.customUserAgent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/83.0.4103.116 Safari/537.36"
                FigCLI.web(with: scope)
            case .position:
                FigCLI.position(with: scope)
            case .hide:
                companionWindow.positioning = .icon
                FigCLI.url(with: scope)
            case .apps, .store, .appstore, .blocks, .home, .sidebar:
                companionWindow.positioning = .fullwindow
                FigCLI.url(with: scope)
            case .accesibility:
                ShellBridge.promptForAccesibilityAccess()
                scope.companionWindow.windowManager.close(window:  scope.companionWindow)

            case .logout:
                Defaults.email = nil
                Defaults.loggedIn = false
                scope.webView.deleteCache()
                FigCLI.printInTerminal(text: "→ Logging out of Fig...", scope: scope)
                ShellBridge.shared.socketServer.send(sessionId: scope.session, command: "disconnect")
                if let delegate = NSApp.delegate as? AppDelegate {
                    delegate.restart()
                }

                            
//                let _ = "open -b \"com.mschrage.fig\"".runAsCommand()
            case .restart:
                FigCLI.printInTerminal(text: "→ Restarting Fig...", scope: scope)
                ShellBridge.shared.socketServer.send(sessionId: scope.session, command: "disconnect")

                if let delegate = NSApp.delegate as? AppDelegate {
                    delegate.restart()
                }
//                let _ = "osascript -e 'quit app \"Fig\"'; open -b \"com.mschrage.fig\"".runAsCommand()
            case .build:
                if let buildMode = Build(rawValue: scope.options.first ?? "") {
                    let msg = "→ Setting build to \(buildMode.rawValue)"
                    FigCLI.printInTerminal(text: msg, scope: scope)
                    Defaults.build = buildMode
                } else {
                    let msg = "→ Current build is '\( Defaults.build .rawValue)'\n\n fig util:build [prod | staging | dev]"
                    FigCLI.printInTerminal(text: msg, scope: scope)

                    scope.companionWindow.windowManager.close(window:  scope.companionWindow)

                }

            case .version:
                FigCLI.runInTerminal(script: "echo \"\(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "-1")\"", scope: scope)
            case .close:
                scope.companionWindow.windowManager.close(window:  scope.companionWindow)
            case .resetCache:
                WebView.deleteCache()
                FigCLI.printInTerminal(text: "→ Resetting WebKit Cache", scope: scope)
            case .source:
                let path = Bundle.main.path(forResource: "source", ofType: "sh")
                FigCLI.runInTerminal(script: "bash \(path!)", scope: scope)
            case .list:
                let specs = try? FileManager.default.contentsOfDirectory(at: URL(fileURLWithPath:  "\(NSHomeDirectory())/.fig/autocomplete", isDirectory: true), includingPropertiesForKeys: nil, options: .skipsHiddenFiles).map { "\($0.lastPathComponent.replacingOccurrences(of: ".\($0.pathExtension)", with: ""))" }.joined(separator: "\n")
                FigCLI.printInTerminal(text: specs ?? "No completions found.\n  Try running fig update.", scope: scope)
            case .feedback:
                let path = Bundle.main.path(forResource: "feedback", ofType: "sh")
                FigCLI.runInTerminal(script: "bash \(path!)", scope: scope)
            case .update:
                let path = Bundle.main.path(forResource: "update-autocomplete", ofType: "sh")
                FigCLI.runInTerminal(script: "bash \(path!)", scope: scope)
            case .invite:
                let count = scope.options.count
                let isPlural = count != 1
                FigCLI.printInTerminal(text: "→ Sending invite\(isPlural ? "s" : "") to \(count) \(isPlural ? "people" :"person")!", scope: scope)
                ShellBridge.shared.socketServer.send(sessionId: scope.session, command: "disconnect")


                var request = URLRequest(url: URL(string:"https://fig-core-backend.herokuapp.com/waitlist/invite-friends?via=cli")!)
                guard let json = try? JSONSerialization.data(withJSONObject: ["emails" : scope.options, "referrer" : Defaults.email ?? ""] , options: .sortedKeys) else {
//                    ShellBridge.shared.socketServer.send(sessionId: scope.session, command: "disconnect")
                    return
                    
                }

                request.httpMethod = "POST"
                request.httpBody = json
                request.setValue("application/json; charset=utf-8", forHTTPHeaderField: "Content-Type")
                let task = URLSession.shared.dataTask(with: request) { (data, res, err) in
//                    guard err == nil else {
//                        ShellBridge.shared.socketServer.send(sessionId: scope.session, command: "disconnect")
//                        return
//                    }
//
//                    FigCLI.printInTerminal(text: "Sent invites!", scope: scope)
//                    ShellBridge.shared.socketServer.send(sessionId: scope.session, command: "disconnect")
                }

                task.resume()
                return
            case .help, .h:
                scope.companionWindow.windowManager.close(window:  scope.companionWindow)

//                let localRunbooks = try? FileManager.default.contentsOfDirectory(at: URL(fileURLWithPath:  "\(NSHomeDirectory())/run/", isDirectory: true), includingPropertiesForKeys: nil, options: .skipsHiddenFiles).map { "  \($0.lastPathComponent.replacingOccurrences(of: ".\($0.pathExtension)", with: ""))" }.joined(separator: "\n")
//                // load file from disk
//                 let url = URL(fileURLWithPath: "~/.fig/cli.txt")
//                 let out = try? String(contentsOf: url, encoding: String.Encoding.utf8)
//                let helpMessage =
//"""
//CLI to interact with Fig
//
//\\u001b[1mUSAGE\\u001b[0m
//  $ fig [SUBCOMMAND]
//
//\\u001b[1mCOMMANDS\\u001b[0m
//  home            update your sidebar
//  apps            browse all availible apps
//  runbooks        view and edit your runbooks
//  settings        view settings for Fig
//  docs            open Fig documentation
//  web <URL>       access websites on the internet
//  local <PATH>    load local html files
//  run <PATH>      load local rundown file
//
//\\u001b[1mAPPS\\u001b[0m
//  dir             browse your file system
//  curl            build http requests
//  git             a lightweight UI for git
//  google <QUERY>  search using Google
//  psql            view and query Postgres databases
//  monitor         visualize CPU usage by process
//  sftp            browse files on remote servers
//  alias           create aliases for common commands
//  readme          preview Readme markdown documents
//  + more          (run \\u001b[1mfig apps\\u001b[0m to view App Store)
//
//\\u001b[1mCOMMUNITY\\u001b[0m
//  @user           view a user's public runbooks
//  +team.com       view your team's shared runbooks
//
//\\u001b[1mLOCAL RUNBOOKS\\u001b[0m
//\(localRunbooks ?? "  (none)          no runbooks in ~/run")
//"""
//  #chat           chat with others about a #topic

                let helpMessage =
"""
CLI to interact with Fig

\\u001b[1mUSAGE\\u001b[0m
  $ fig [COMMAND]

\\u001b[1mCOMMANDS\\u001b[0m
  fig invite        invite up to 5 friends & teammates to Fig
  fig feedback      send feedback directly to the Fig founders
  fig update        update repo of completion scripts
  fig docs          documentation for building completion specs
  fig source        (re)connect fig to the current shell session
  fig list          print all commands with completion specs
  fig --help        a summary of the Fig CLI commands
"""
                FigCLI.runInTerminal(script: "echo \"\(helpMessage)\"", scope: scope)
            }
        } else if (aliases.keys.contains(command)) { // user defined shortcuts
            
            if let meta = aliases[command] as? [String: Any],
                let rawCommand = meta["raw"] as? String,
                let popup = meta["popup"] as? Bool {
                
                if popup {
                    FigCLI.form(with: scope, format: rawCommand)
                    companionWindow.positioning = .popover
                } else {
                    Timer.delayWithSeconds(0.15) {
                        ShellBridge.injectStringIntoTerminal(rawCommand, runImmediately: true)
                    }
                }
            }
            
            
            
//            companionWindow.windowManager.close(window: companionWindow)
            // get script for command
        } else if (scope.cmd.starts(with: "@") || scope.cmd.starts(with: "+")) {
            // fig.run/@mschrage/document
            
            scope.webView.loadRemoteApp(at: URL(string: "https://fig.run/\(scope.cmd)/\(scope.options.first ?? "")?token=\(Defaults.domainToken ?? "")") ?? FigCLI.baseURL)
            FigCLI.env(with: scope)
            FigCLI.options(with: scope, removeFirstOption: true)
            FigCLI.stdin(with: scope)
        } else if (isCLI) {
            FigCLI.dotfig(filePath: appPath!, scope: scope)
        } else if (isRundown) {
            FigCLI.run(scope: scope, path: appPath!)
        } else if (isExecutable) {
            FigCLI.runInTerminal(script: appPath!, scope: scope)
        } else if let path = appPath { //fig path
            let modified = Scope(cmd: "local",
                              stdin: stdin,
                              options: [path] + scope.options,
                              env: env,
                              webView: webView,
                              companionWindow: companionWindow,
                              session: message.session)
            FigCLI.local(with: modified)
        } else {
            FigCLI.url(with: scope)
        }
        
        ShellBridge.shared.socketServer.send(sessionId: scope.session, command: "disconnect")
    }
}

extension String {
    func jsonStringToDict() -> [String: Any]? {
        if let data = self.data(using: .utf8) {
            do {
                return try JSONSerialization.jsonObject(with: data, options: []) as? [String: Any]
            } catch {
                print(error.localizedDescription)
            }
        }
        return nil
    }
}

extension Collection {

    /// Returns the element at the specified index if it is within bounds, otherwise nil.
    subscript (safe index: Index) -> Element? {
        return indices.contains(index) ? self[index] : nil
    }
}

//https://stackoverflow.com/a/48360631
extension URL {
    func relativePath(from base: URL) -> String? {
        // Ensure that both URLs represent files:
        guard self.isFileURL && base.isFileURL else {
            return nil
        }

        // Remove/replace "." and "..", make paths absolute:
        let destComponents = self.standardized.pathComponents
        let baseComponents = base.standardized.pathComponents

        // Find number of common path components:
        var i = 0
        while i < destComponents.count && i < baseComponents.count
            && destComponents[i] == baseComponents[i] {
                i += 1
        }

        // Build relative path:
        var relComponents = Array(repeating: "..", count: baseComponents.count - i)
        relComponents.append(contentsOf: destComponents[i...])
        return relComponents.joined(separator: "/")
    }
}

extension String {
    func groups(for regexPattern: String) -> [[String]] {
    do {
        let text = self
        let regex = try NSRegularExpression(pattern: regexPattern)
        let matches = regex.matches(in: text,
                                    range: NSRange(text.startIndex..., in: text))
        return matches.map { match in
            return (0..<match.numberOfRanges).map {
                let rangeBounds = match.range(at: $0)
                guard let range = Range(rangeBounds, in: text) else {
                    return ""
                }
                return String(text[range])
            }
        }
    } catch let error {
        print("invalid regex: \(error.localizedDescription)")
        return []
    }
}
}

extension String {
    var unescapingUnicodeCharacters: String {
        let mutableString = NSMutableString(string: self)
        CFStringTransform(mutableString, nil, "Any-Hex/Java" as NSString, true)

        return mutableString as String
    }
}

extension String {
    var unescaped: String {
        let entities = ["\0", "\t", "\n", "\r", "\"", "\'", "\\"]
        var current = self.replacingOccurrences(of: "\\/", with: "/")
        for entity in entities {
            let descriptionCharacters = entity.debugDescription.dropFirst().dropFirst().dropLast().dropLast()
            let description = String(descriptionCharacters)
            current = current.replacingOccurrences(of: description, with: entity)
        }
        return current
    }
}
