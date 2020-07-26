//
//  FigJS.swift
//  fig
//
//  Created by Matt Schrage on 5/26/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

class Scope {
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

    
    init(cmd: String,
         stdin: String,
         options: [String],
         env: String,
         webView: WebView,
         companionWindow: CompanionWindow) {
        self.cmd = cmd
        self.stdin = stdin
        self.options = options
        self.env = env
        self.webView = webView
        self.companionWindow = companionWindow
        
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
        scope.webView.loadRemoteApp(at: FigCLI.baseURL)
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
        scope.webView.onLoad.append {
              WebBridge.appInitialPosition(webview: scope.webView) { (position) in
                scope.companionWindow.positioning = CompanionWindow.OverlayPositioning(rawValue:                     Int(position ?? "-1")! ) ?? scope.companionWindow.positioning
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
        scope.webView.loadRemoteApp(at: URL(string: scope.options[safe: 0] ?? "") ?? FigCLI.baseURL)
        FigCLI.env(with: scope)
        FigCLI.options(with: scope, removeFirstOption: true)
        FigCLI.stdin(with: scope)
//        FigCLI.initialPosition(with: scope)
        
        if (Defaults.automaticallyLaunchWebAppsInDetachedWindow) {
            scope.companionWindow.untether()
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
            let env = FigCLI.extract(keys: ["PWD","USER","HOME","SHELL", "OLDPWD", "TERM_PROGRAM", "TERM_SESSION_ID", "HISTFILE","FIG","FIGPATH"], from: scope.env)
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

    static func run(scope: Scope) {
        let modified = Scope(cmd: "run",
                             stdin: scope.stdin,
                             options: scope.options,
                             env: scope.env,
                             webView: scope.webView,
                             companionWindow: scope.companionWindow)
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
            Timer.delayWithSeconds(0.15) {
                ShellBridge.injectStringIntoTerminal(script, runImmediately: true, completion: nil)
            }
//            "\(cmd.cmdToRun!) \(flags.joined(separator: " "))".runAsCommand(cwd: scope.pwd, with: scope.env.jsonStringToDict() as? Dictionary<String, String>)
            
        } catch {
            
        }
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
    
    static func route(_ message: ShellMessage, webView: WebView, companionWindow: CompanionWindow) {
        
        let stdin = message.data.replacingOccurrences(of: "`", with: "\\`")
        let env = message.env ?? ""
        webView.clearHistory()
        webView.window?.representedURL = nil        
      
        guard let options = message.options, options.count > 0 else {
            let scope = Scope(cmd: "", stdin: stdin, options: [], env: env, webView: webView, companionWindow: companionWindow)
            companionWindow.positioning =  .spotlight
            FigCLI.index(with: scope)
            TelemetryProvider.post(event: .ranCommand, with:
                                    ["cmd" : scope.cmd,
                                    "args" : scope.options.map { TelemetryProvider.obscure($0)}.joined(separator: " ") ])
            return
        }
        let command = options.first!
        let flags = Array(options.suffix(from: 1))
        let scope = Scope(cmd: command,
                          stdin: stdin,
                          options: flags,
                          env: env,
                          webView: webView,
                          companionWindow: companionWindow)
        print("ROUTING \(command)")
        
        TelemetryProvider.post(event: .ranCommand, with: ["cmd" : scope.cmd, "args" : scope.options.map { TelemetryProvider.obscure($0)}.joined(separator: " ") ])
        
        // get aliases from sidebar
        let aliases = UserDefaults.standard.string(forKey: "aliases_dict")?.jsonStringToDict() ?? [:]
        
        // get
        let figPath = FigCLI.extract(keys: ["FIGPATH"], from: scope.env).first?.value.split(separator: ":") ?? []
        
        var isCLI = false
        var appPath: String? = nil
        for prefix in figPath {
            isCLI = FileManager.default.fileExists(atPath: "\(prefix)/\(command).fig")
            if (isCLI) {
                appPath = "\(prefix)/\(command).fig"
                break
            }
            
            let isRaw = FileManager.default.fileExists(atPath: "\(prefix)/\(command)")
            if (isRaw) {
                appPath = "\(prefix)/\(command)"
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
        }
        
         companionWindow.positioning = storedInitialPosition(for: command) ?? CompanionWindow.defaultActivePosition
        
        if let nativeCommand = NativeCLICommand(rawValue: command) {
            switch nativeCommand {
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
            case .apps, .store, .appstore, .blocks, .home:
                companionWindow.positioning = .fullwindow
                FigCLI.url(with: scope)
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
        } else if (isCLI) {
            FigCLI.dotfig(filePath: appPath!, scope: scope)
        } else if let path = appPath { //fig path
            let modified = Scope(cmd: "local",
                              stdin: stdin,
                              options: [path],
                              env: env,
                              webView: webView,
                              companionWindow: companionWindow)
            FigCLI.local(with: modified)
        } else {
            FigCLI.url(with: scope)
        }
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
