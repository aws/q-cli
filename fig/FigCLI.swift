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
    let options: [String]
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

}

class FigCLI {
    static let baseURL = URL(string: "https://app.withfig.com/")!

    static func index(with scope: Scope ) {
        scope.webView.loadRemoteApp(at: FigCLI.baseURL)
//        FigCLI.stdin(with: scope)
    }
    
    static func callback(with scope: Scope) {
        scope.webView.evaluateJavaScript("fig.\(scope.options[0])(`\(scope.stdin)`)", completionHandler: nil)
    }
    
    static func local(with scope: Scope) {
        if let path = scope.options[safe: 0], let pwd = scope.pwd  {
            let url = URL(fileURLWithPath: path, relativeTo: URL(fileURLWithPath: pwd))
            scope.webView.loadLocalApp(url)
            FigCLI.env(with: scope)
            FigCLI.options(with: scope)
            FigCLI.stdin(with: scope)
        }

    }
    
    static func bundle(with scope: Scope) {
        scope.webView.loadBundleApp(scope.options[0])
        FigCLI.env(with: scope)
        FigCLI.options(with: scope)
        FigCLI.stdin(with: scope)
    }
    
    static func web(with scope: Scope) {
        scope.webView.loadRemoteApp(at: URL(string: scope.options[safe: 0] ?? "") ?? FigCLI.baseURL)
        FigCLI.env(with: scope)
        FigCLI.options(with: scope)
        FigCLI.stdin(with: scope)
    }
    
    static func position(with scope: Scope) {
        let positionValue = Int(scope.options[0]) ?? -1
        scope.companionWindow.positioning = CompanionWindow.OverlayPositioning(rawValue: positionValue) ?? .insideRightPartial
        scope.companionWindow.repositionWindow(forceUpdate: true)
    }

    static func hide(with scope: Scope) {
        scope.companionWindow.positioning = CompanionWindow.defaultPassivePosition
    }
    
    static func options(with scope: Scope) {
        scope.webView.onLoad.append {
            let opts = scope.options
            if let json = try? JSONSerialization.data(withJSONObject: opts, options: .fragmentsAllowed) {
                scope.webView.evaluateJavaScript("fig.options = JSON.parse(b64DecodeUnicode(`\(json.base64EncodedString())`))", completionHandler: nil)
                  
            }
        }
    }
    
    static func env(with scope: Scope) {
        scope.webView.configureEnvOnLoad = {
            let env = FigCLI.extract(keys: ["PWD","USER","HOME","SHELL", "OLDPWD", "TERM_PROGRAM", "TERM_SESSION_ID"], from: scope.env)
            
            if let json = try? JSONSerialization.data(withJSONObject: env, options: .fragmentsAllowed) {
                scope.webView.evaluateJavaScript("fig.env = JSON.parse(b64DecodeUnicode(`\(json.base64EncodedString())`))", completionHandler: nil)
                  
            }
        }
    }
    
    private static func extract(keys:[String], from env: String) -> [String: String] {
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
    
    static func url(with scope: Scope) {
        var all = scope.options
        all.insert(scope.cmd, at: 0)
        let url = ShellBridge.commandLineOptionsToRawURL(all)
        scope.webView.loadRemoteApp(at: url)
        FigCLI.env(with: scope)
        FigCLI.options(with: scope)
        FigCLI.stdin(with: scope)
    }
    
    static func route(_ message: ShellMessage, webView: WebView, companionWindow: CompanionWindow) {
        let stdin = message.data.replacingOccurrences(of: "`", with: "\\`")
        let env = message.env ?? ""
        companionWindow.positioning =  CompanionWindow.defaultActivePosition

        guard let options = message.options, options.count > 0 else {
            let scope = Scope(cmd: "", stdin: stdin, options: [], env: env, webView: webView, companionWindow: companionWindow)
            FigCLI.index(with: scope)
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
        let data = env.data(using: .utf8)!
        do {
            if let jsonArray = try JSONSerialization.jsonObject(with: data, options : .allowFragments) as? [Dictionary<String,Any>]
            {
               print(jsonArray) // use the json here
            } else {
                print("bad json")
            }
        } catch let error as NSError {
            print(error)
        }
        
        if let nativeCommand = NativeCLICommand(rawValue: command) {
            switch nativeCommand {
            case .callback:
                FigCLI.callback(with: scope)
            case .bundle:
                FigCLI.bundle(with: scope)
            case .local:
                FigCLI.local(with: scope)
            case .web:
                FigCLI.web(with: scope)
            case .position:
                FigCLI.position(with: scope)
            case .hide:
                companionWindow.positioning = .icon
                FigCLI.url(with: scope)
            case .apps, .store, .appstore, .blocks, .home:
                companionWindow.positioning = .fullscreenInset
                FigCLI.url(with: scope)
            }
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
