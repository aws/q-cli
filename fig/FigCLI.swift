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
            FigCLI.stdin(with: scope)
        }

    }
    
    static func bundle(with scope: Scope) {
        scope.webView.loadBundleApp(scope.options[0])
        FigCLI.stdin(with: scope)
    }
    
    static func web(with scope: Scope) {
        scope.webView.loadRemoteApp(at: URL(string: scope.options[0]) ?? FigCLI.baseURL)
        FigCLI.stdin(with: scope)
    }
    
    static func position(with scope: Scope) {
        let positionValue = Int(scope.options[0]) ?? -1
        scope.companionWindow.positioning = CompanionWindow.OverlayPositioning(rawValue: positionValue) ?? .insideRightPartial
        scope.companionWindow.repositionWindow(forceUpdate: true)
    }

    static func hide(with scope: Scope) {
        scope.companionWindow.positioning = .icon
    }
    
    static func stdin(with scope: Scope) {
        Timer.delayWithSeconds(0.25) {
            let encoded = scope.stdin.data(using: .utf8)!
            scope.webView.evaluateJavaScript("fig.env = JSON.parse(`\(scope.env)`);", completionHandler: nil)
            scope.webView.evaluateJavaScript("fig.stdinb64(`\(encoded.base64EncodedString())`)", completionHandler: nil)
        }
    }
    
    static func url(with scope: Scope) {
        var all = scope.options
        all.insert(scope.cmd, at: 0)
        let url = ShellBridge.commandLineOptionsToRawURL(all)
        scope.webView.loadRemoteApp(at: url)
        FigCLI.stdin(with: scope)
    }
    
    static func route(_ message: ShellMessage, webView: WebView, companionWindow: CompanionWindow) {
        let stdin = message.data.replacingOccurrences(of: "`", with: "\\`")
        let env = message.env ?? ""
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
                FigCLI.hide(with: scope)
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
