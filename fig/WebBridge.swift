//
//  WebBridge.swift
//  fig
//
//  Created by Matt Schrage on 5/13/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import WebKit

protocol WebBridgeEventDelegate {
    func requestExecuteCLICommand(script: String)
    func requestInsertCLICommand(script: String)
}

class WebBridge : WKWebViewConfiguration {
    var eventDelegate: WebBridgeEventDelegate?
    
    convenience init(eventDelegate: WebBridgeEventDelegate?) {
        self.init()
        self.eventDelegate = eventDelegate;
    }
    
    override init() {
        super.init()
        self.preferences.setValue(true, forKey: "developerExtrasEnabled")
        self.preferences.setValue(true, forKey: "allowFileAccessFromFileURLs")
//        self.preferences.setValue(true, forKey: "mediaPreloadingEnabled")
//        self.preferences.setValue(true, forKey: "linkPreloadEnabled")

//        self.webView.configuration.preferences
        
        self.setURLSchemeHandler(self, forURLScheme: "fig")
        self.setURLSchemeHandler(self, forURLScheme: "figbundle")

        let contentController = WebBridgeContentController()
        
        let eventHandlers: [WebBridgeEventHandler] = [.logHandler,
                                                      .exceptionHandler,
                                                      .insertHandler,
                                                      .executeHandler,
                                                      .executeInBackgroundHandler,
                                                      .callbackHandler]
        
        contentController.add(self, name: WebBridgeScript.executeCLIHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.insertCLIHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.callbackHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.fwriteHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.freadHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.focusHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.blurHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.appwriteHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.appreadHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.positionHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.openHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.streamHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.defaultsHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.normalizeFilePath.rawValue)

        contentController.add(self, name: WebBridgeScript.onboardingHandler.rawValue)

        contentController.add(self, name: WebBridgeScript.logging.rawValue)
        contentController.add(self, name: WebBridgeScript.exceptions.rawValue)

        self.userContentController = contentController
//        self.setURLSchemeHandler(self, forURLScheme: "fig")
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    
}



enum WebBridgeEventHandler: String, CaseIterable {
    case logHandler = "logHandler"
    case exceptionHandler = "exceptionHandler"
    case insertHandler = "insertHandler"
    case executeHandler = "executeHandler"
    case executeInBackgroundHandler = "executeInBackgroundHandler"
    case callbackHandler = "callbackHandler"
}

enum WebBridgeScript: String, CaseIterable {
    case figJS = "js"
    case logging = "logHandler"
    case exceptions = "exceptionHandler"
    case insertFigTutorialCSS = "css"
    case insertFigTutorialJS = "tutorial"
    case insertCLIHandler = "insertHandler"
    case executeCLIHandler = "executeHandler"
    case callbackHandler = "callbackHandler"
    case executeInBackgroundHandler = "executeInBackgroundHandler"
    case stdoutHandler = "stdoutHandler"
    case fwriteHandler = "fwriteHandler"
    case freadHandler = "freadHandler"
    case focusHandler = "focusHandler"
    case blurHandler = "blurHandler"
    case appwriteHandler = "appwriteHandler"
    case appreadHandler = "appreadHandler"
    case positionHandler = "positionHandler"
    case openHandler = "openHandler"
    case streamHandler = "streamHandler"
    case defaultsHandler = "defaultsHandler"
    case injectTerminalCSS = "terminalCSS"
    case normalizeFilePath = "filepathHandler"

    case onboardingHandler = "onboardingHandler"

    case enforceViewportSizing = "enforceViewportSizing"

}

extension WebBridge: WKURLSchemeHandler {
    func webView(_ webView: WKWebView, start urlSchemeTask: WKURLSchemeTask) {
        guard let url = urlSchemeTask.request.url else {
            return
        }
        
        var width = 32.0
        var height = 32.0

        if let qs = url.queryDictionary, let w = qs["w"], let wd = Double(w), let h = qs["h"], let hd = Double(h) {
            width = wd
            height = hd
        }
        
        guard let specifier = (url as NSURL).resourceSpecifier else { return }
        let resource = specifier.replacingOccurrences(of: "?\(url.query ?? "<none>")", with: "")
        print(url.path)
        guard let fileicon = NSWorkspace.shared.icon(forFile: resource).resized(to: NSSize(width: width, height: height)) else {
            return
        }
        //Create a NSURLResponse with the correct mimetype.
        
        let response = URLResponse(url: url, mimeType: "image/png", expectedContentLength: -1, textEncodingName: nil)
        
        guard let tiffData = fileicon.tiffRepresentation else {
              print("failed to get tiffRepresentation. url: \(url)")
            return
        }
        let imageRep = NSBitmapImageRep(data: tiffData)
        guard let imageData = imageRep?.representation(using: .png, properties: [:]) else {
              print("failed to get PNG representation. url: \(url)")
            return
          }
        urlSchemeTask.didReceive(response)
        urlSchemeTask.didReceive(imageData)
        urlSchemeTask.didFinish()
    }
    
    func webView(_ webView: WKWebView, stop urlSchemeTask: WKURLSchemeTask) {
        
    }
}

extension NSImage {
    func resized(to newSize: NSSize) -> NSImage? {
        if let bitmapRep = NSBitmapImageRep(
            bitmapDataPlanes: nil, pixelsWide: Int(newSize.width), pixelsHigh: Int(newSize.height),
            bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
            colorSpaceName: .calibratedRGB, bytesPerRow: 0, bitsPerPixel: 0
        ) {
            bitmapRep.size = newSize
            NSGraphicsContext.saveGraphicsState()
            NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmapRep)
            draw(in: NSRect(x: 0, y: 0, width: newSize.width, height: newSize.height), from: .zero, operation: .copy, fraction: 1.0)
            NSGraphicsContext.restoreGraphicsState()

            let resizedImage = NSImage(size: newSize)
            resizedImage.addRepresentation(bitmapRep)
            return resizedImage
        }

        return nil
    }
}

class WebBridgeContentController : WKUserContentController {
    override init() {
        super.init()
    
        
//        let legacy: [WebBridgeScript]  = [ .insertFigTutorialCSS, .figJS ]
//        let scripts: [WebBridgeScript] = [.logging, .exceptions, .figJS]
       
//        self.addWebBridgeScript(.exceptions, location: .atDocumentStart)
//        self.addWebBridgeScript(.logging, location: .atDocumentStart);
        self.addWebBridgeScript(.insertFigTutorialCSS);
        self.addWebBridgeScript(.insertFigTutorialJS);
        
        self.addWebBridgeScript(.injectTerminalCSS);

        self.addWebBridgeScript(.figJS, location: .atDocumentStart);
    }
    
    func addWebBridgeScript(_ scriptType:WebBridgeScript,  location: WKUserScriptInjectionTime = .atDocumentEnd) {
        let source = scriptType.codeForScript()
        let script = WKUserScript(source: source, injectionTime: location, forMainFrameOnly: false)
        self.addUserScript(script)
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

extension WebBridge : WKScriptMessageHandler {
    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        let scriptType = WebBridgeScript.init(rawValue: message.name)
        switch scriptType {
        case .logging, .exceptions:
            WebBridge.log(scope: message)
        case .insertCLIHandler:
            WebBridge.insert(scope: message)
        case .executeCLIHandler:
            WebBridge.execute(scope: message)
        case .callbackHandler:
            WebBridge.executeInBackground(scope: message)
        case .fwriteHandler:
            WebBridge.fwrite(scope: message)
        case .freadHandler:
            WebBridge.fread(scope: message)
        case .focusHandler:
            WebBridge.focus(scope: message)
        case .blurHandler:
            WebBridge.blur(scope: message)
        case .appreadHandler:
            WebBridge.appread(scope: message)
        case .appwriteHandler:
            WebBridge.appwrite(scope: message)
        case .positionHandler:
            WebBridge.position(scope: message)
        case .openHandler:
            WebBridge.open(scope:message)
        case .streamHandler:
            WebBridge.stream(scope: message)
        case .onboardingHandler:
            WebBridge.onboarding(scope: message)
        case .defaultsHandler:
            WebBridge.defaults(scope: message)
        case .normalizeFilePath:
            WebBridge.normalizeFilePath(scope: message)
        default:
            print("Unhandled WKScriptMessage type '\(message.name)'")
        }
      
    }
}

protocol WebBridgeEventListener {
    func insertCommandInTerminal(_ notification: Notification)
    func executeCommandInTerminal(_ notification: Notification)
}

extension Notification.Name {
    static let insertCommandInTerminal = Notification.Name("insertCommandInTerminal")
    static let executeCommandInTerminal = Notification.Name("executeCommandInTerminal")
}


extension WebBridge {
    static func log(scope: WKScriptMessage) {
        let body = scope.body as? String
        if let body = body {
            print("JS Console: \(body)")
            Logger.log(message: "\(scope.webView?.url?.absoluteString ?? "<none>"): \(body)\n")
        } else {
            print("JS Console: Tried to write something that wasn't a string")
            Logger.log(message: "\(scope.webView?.url?.absoluteString ?? "<none>"): Attempted to write something that wasn't a string to the fig log.\n\nUse `fig.log()` in the future to avoid this error or `JSON.stringify()` any input passed into `console.log`. \n")
        }

    }
    
    static func insert(scope: WKScriptMessage) {
        NotificationCenter.default.post(name: .insertCommandInTerminal, object: scope.body as! String)
    }
    
    static func execute(scope: WKScriptMessage) {
        NotificationCenter.default.post(name: .executeCommandInTerminal, object: scope.body as! String)
    }
    
    static func executeInBackground(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let cmd = params["cmd"],
           let handlerId = params["handlerId"],
           let env = params["env"]?.jsonStringToDict(),
           let pwd = env["PWD"] as? String {
            print("'\(cmd)' running in background...")
            let output = cmd.runAsCommand(cwd: pwd, with: env as? Dictionary<String, String>)
            print("\(cmd) -> \(output)")
            let encoded = output.data(using: .utf8)!
            scope.webView?.evaluateJavaScript("fig.callback(`\(handlerId)`,`\(encoded.base64EncodedString())`)", completionHandler: nil)

        } else {
            Logger.log(message: "Couldn't execute \(scope.body)")
        }

    }
    
    static func executeWithCallback(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let cmd = params["cmd"],
           let handlerId = params["handlerId"] {
            NotificationCenter.default.post(name: .executeCommandInTerminal, object:
                "\(cmd) | fig callback \(handlerId)")
        }
    }
    
    static func fwrite(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let path = params["path"],
           let data = params["data"],
           let handlerId = params["handlerId"],
           let env = params["env"]?.jsonStringToDict(),
           let pwd = env["PWD"] as? String {
            let url = URL(fileURLWithPath: path, relativeTo: URL(fileURLWithPath: pwd))
            do {
                try data.write(to: url, atomically: true, encoding: String.Encoding.utf8)
//                scope.webView?.evaluateJavaScript("fig.callback(`\(handlerId)`, null)", completionHandler: nil)
            } catch {
//  scope.webView?.evaluateJavaScript("fig.callback(`\(handlerId)`,{message:'Could not write file to disk.'})", completionHandler: nil)

            }
        }
    }
    
    static func fread(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let path = params["path"],
           let handlerId = params["handlerId"],
           let env = params["env"]?.jsonStringToDict(),
           let pwd = env["PWD"] as? String {
            
            let url = URL(fileURLWithPath: path, relativeTo: URL(fileURLWithPath: pwd))
            do {
                let out = try String(contentsOf: url, encoding: String.Encoding.utf8)
                let encoded = out.data(using: .utf8)!
                
                scope.webView?.evaluateJavaScript("fig.callback(`\(handlerId)`, `\(encoded.base64EncodedString())`)", completionHandler: nil)

            } catch {
                scope.webView?.evaluateJavaScript("fig.callback(`\(handlerId)`, null,{message:'Could not read file from disk.'})", completionHandler: nil)
            }
        }
    }
    
    static func focus(scope: WKScriptMessage) {
        NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
    }
    
    static func blur(scope: WKScriptMessage) {
        ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
    }
    
    static func appwrite(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let path = params["path"],
           let data = params["data"],
           let handlerId = params["handlerId"],
           let webview = scope.webView as? WebView {
                WebBridge.appname(webview: webview) { (app) in
                  let url = URL(fileURLWithPath: "\(app ?? "tmp")/\(path)", relativeTo: WebBridge.appDirectory)
                    print(FileManager.default.fileExists(atPath: url.absoluteString))
                    do {
                        var directory = url
                        directory.deleteLastPathComponent()
                        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true, attributes: nil)
                        try data.write(to: url, atomically: true, encoding: String.Encoding.utf8)

                       
                        webview.evaluateJavaScript("fig.callback(`\(handlerId)`, null)", completionHandler: nil)
                    } catch {
                        Logger.log(message: "Could not write file '\(path)' to disk.")
                        scope.webView?.evaluateJavaScript("fig.callbackASCII(`\(handlerId)`,`Could not write file to disk.`)", completionHandler: nil)

                    }
                }
            }
        }
        
        static func appread(scope: WKScriptMessage) {
            if let params = scope.body as? Dictionary<String, String>,
               let path = params["path"],
               let handlerId = params["handlerId"],
               let app = scope.webView?.url?.deletingPathExtension().pathComponents.last {

                let url = URL(fileURLWithPath: "\(app)/\(path)", relativeTo: WebBridge.appDirectory)
                do {
                    let out = try String(contentsOf: url, encoding: String.Encoding.utf8)
                    let encoded = out.data(using: .utf8)!
                    
                    scope.webView?.evaluateJavaScript("fig.callback(`\(handlerId)`, `\(encoded.base64EncodedString())`)", completionHandler: nil)

                } catch {
                    scope.webView?.evaluateJavaScript("fig.callback(`\(handlerId)`, null,{message:'Could not read file from disk.'})", completionHandler: nil)
                }
            }
        }
    static func ttyin(webView: WKWebView, msg: ShellMessage) {
        DispatchQueue.main.async {
            let encoded = msg.data.data(using: .utf8)!
            webView.evaluateJavaScript("fig.ttyinb64(`\(encoded.base64EncodedString())`, '\(msg.session)')", completionHandler: nil)
        }
    }
    
    static func ttyout( webView: WKWebView, msg: ShellMessage) {
        DispatchQueue.main.async {
            let encoded = msg.data.data(using: .utf8)!
            webView.evaluateJavaScript("fig.ttyoutb64(`\(encoded.base64EncodedString())`,'\(msg.session)')", completionHandler: nil)
        }
    }
    
    static func position(scope: WKScriptMessage) {
        if let companion = scope.webView?.window as? CompanionWindow,
           let params = scope.body as? Dictionary<String, String>,
           let position = params["position"]{
            companion.positioning = CompanionWindow.OverlayPositioning(rawValue: Int(position) ?? 2) ?? .icon
            companion.repositionWindow(forceUpdate: true)
        }
    }
    
    static func open(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let urlString = params["url"],
           let url = URL(string: urlString) {
            NSWorkspace.shared.open(url)
        }
    }
    
    static func stream(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let cmd = params["cmd"],
           let handlerId = params["handlerId"],
           let env = params["env"]?.jsonStringToDict(),
           let pwd = env["PWD"] as? String {
            print("'\(cmd)' streaming in background...")
            let task = cmd.runInBackground(cwd:pwd, with: env as? Dictionary<String, String>, updateHandler: { (line, process) in
                DispatchQueue.main.async {
                    print("\(cmd) -> \(line)")
                    let encoded = line.data(using: .utf8)!
                scope.webView?.evaluateJavaScript("fig.callback(`\(handlerId)`,`\(encoded.base64EncodedString())`)", completionHandler: nil)
                }

            }, completion: {
                DispatchQueue.main.async {
                      print("\(cmd) is complete!")
                    scope.webView?.evaluateJavaScript("fig.callbackASCII(`\(handlerId)`, null)", completionHandler: nil)
                  }
            })
            
            (scope.webView as! WebView).onNavigate.append({
                print("Terminating process that was streaming '\(cmd)'")
                task.terminate()
            })

        } else {
            Logger.log(message: "Couldn't stream \(scope.body)")
        }

    }
    
    static func onboarding(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let action = params["action"],
           let handlerId = params["handlerId"] {

            switch (action) {
                case "cli":
                    ShellBridge.symlinkCLI {
                        DispatchQueue.main.async {
                            scope.webView?.evaluateJavaScript("fig.callback('\(handlerId)', '')", completionHandler: nil)
                        }
                    }
                case "permissions":
                    ShellBridge.promptForAccesibilityAccess()
                case "ws":
                    ShellBridge.shared.startWebSocketServer()
                case "close":
                    if let delegate = NSApplication.shared.delegate as? AppDelegate {
                        delegate.setupCompanionWindow()
                    }
                    NSWorkspace.shared.launchApplication("Terminal")
                    scope.webView?.window?.close()
                case "forceUpdate":
                    if let appDelegate = NSApp.delegate as? AppDelegate {
                        appDelegate.updater?.installUpdatesIfAvailable()
                    }
                case "promptUpdate":
                    if let appDelegate = NSApp.delegate as? AppDelegate {
                        appDelegate.updater?.checkForUpdates(self)
                    }
                case "hello":
                    Timer.delayWithSeconds(2) {
                        NSApp.deactivate()
                        ShellBridge.injectStringIntoTerminal("bash ~/.fig/hello.sh", runImmediately: true)
                }
            default:
                break;
            }
           
        }
    }
    
    static func callback(handler: String, value: String, scope: WKScriptMessage) {
        let encoded = value.data(using: .utf8)!
        scope.webView?.evaluateJavaScript("fig.callback(`\(handler)`,`\(encoded.base64EncodedString())`)", completionHandler: nil)
    }
    
    static func defaults(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
            let key = params["key"] {
            
            if let handlerId = params["handlerId"] {
                let value = UserDefaults.standard.string(forKey: key)
                WebBridge.callback(handler: handlerId, value: value ?? "", scope: scope)
                
            } else if let value = params["value"] {
                UserDefaults.standard.set(value, forKey: key)
                UserDefaults.standard.synchronize()
            }
        }
    }
    
    static func normalizeFilePath(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let path = params["path"],
           let handlerId = params["handlerId"] {
            
            WebBridge.callback(handler: handlerId, value: NSString(string: path).standardizingPath, scope: scope)
            
        }
    }
    
    static func tabInSidebar(webView: WebView, shift: Bool = false) {
        webView.evaluateJavaScript(shift ? "tabBackward()" : "tabForward()", completionHandler: nil)

        let sibling = shift ? "previousElementSibling" : "nextElementSibling";
        webView.evaluateJavaScript(
          """
          var next = document.activeElement.\(sibling)

          if (next) {
              while (next.tabIndex && next.tabIndex == -1) {
                  next = next.\(sibling)
                  if (!next) {
                      next = document.querySelector('.app')
                      break;
                  }
              }
              console.log(next)
              next.focus()
          } else {
            //document.querySelector('.app').focus()
            
            var nodes = document.querySelectorAll('.app');
            var first = nodes[0];
            var last = nodes[nodes.length-2];
            
            \(shift ? "last" : "first").focus()
          }
          """, completionHandler: nil)
    }
    
    static func activateSelectedAppFromSidebar(webView: WebView) {
        webView.evaluateJavaScript("activateSelectedApp()", completionHandler: nil)

        webView.evaluateJavaScript(
            """
            var target = document.activeElement
            var link = target.getElementsByTagName('a')[0]
            console.log(link)
            link.onmouseup()
            """, completionHandler: nil)
    }
    
    static func declareAppVersion(webview: WebView) {
        if let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String {
            webview.evaluateJavaScript("fig.appversion = '\(appVersion)'", completionHandler: nil)
        }
    }
    
    static func initJS(webview: WebView) {
        webview.evaluateJavaScript("fig.callinit()", completionHandler: nil)
    }
    
    static func appname(webview: WebView, response: @escaping (String?) -> Void) {
        webview.evaluateJavaScript("document.head.querySelector('meta[figapp]').getAttribute('figapp')") { (name, error) in
            response(name as? String)
            return
        }
    }

    
    static var appDirectory: URL = URL(fileURLWithPath: "\(NSHomeDirectory())/.fig/apps/")

}

struct WebBridgeJSCallback : Codable {
    var type: String
    var cmd: String
    var handlerId: String
}


extension URL {
    var queryDictionary: [String: String]? {
        guard let query = self.query else { return nil}

        var queryStrings = [String: String]()
        for pair in query.components(separatedBy: "&") {

            let key = pair.components(separatedBy: "=")[0]

            let value = pair
                .components(separatedBy:"=")[1]
                .replacingOccurrences(of: "+", with: " ")
                .removingPercentEncoding ?? ""

            queryStrings[key] = value
        }
        return queryStrings
    }
}

extension WebBridgeScript {
    func codeForScript() -> String {
        switch self {
            case .exceptions:
                return "function captureException(msg, src, lineno, colno) { window.webkit.messageHandlers.exceptionHandler.postMessage(`${msg} at ${lineno}:${colno} in ${src}`) } window.onerror = captureException;"
            case .logging:
                return "function captureLog(msg) { window.webkit.messageHandlers.logHandler.postMessage(msg); } window.console.log = captureLog;"
            case .figJS:
                return File.contentsOfFile("fig", type: "js")!
            case .insertFigTutorialJS:
                return File.contentsOfFile("tutorial", type: "js")!
            case .insertFigTutorialCSS:
                let cssString = File.contentsOfFile("tutorial", type: "css")!
                return """
                      var style = document.createElement('style');
                      style.innerHTML = `\(cssString)`;
                      document.head.appendChild(style);
                   """
            case .enforceViewportSizing:
                return """
                        var meta = document.createElement('meta');
                        meta.setAttribute('name', 'viewport');
                        meta.setAttribute('content', 'width=device-width');
                        meta.setAttribute('initial-scale', '1.0');
                        meta.setAttribute('maximum-scale', '1.0');
                        meta.setAttribute('minimum-scale', '1.0');
                        meta.setAttribute('user-scalable', 'no');
                        document.getElementsByTagName('head')[0].appendChild(meta);
                        """
            case .injectTerminalCSS:
                let bg = UserDefaults.standard.string(forKey: "terminal-bg-color")
                let text = UserDefaults.standard.string(forKey: "terminal-text-color")
                let cssString  = """
                .terminal-bg-color {
                    background-color: \(bg ?? "white") !important;
                }

                .terminal-text-color {
                    color: \(text ?? "black") !important;
                }
                """
                

                  return """
                        var style = document.createElement('style');
                        style.innerHTML = `\(cssString)`;
                        document.head.appendChild(style);
                        """
            default:
                return ""
        }
    }
}

//extension WebBridge : WKURLSchemeHandler {
//    func webView(_ webView: WKWebView, start urlSchemeTask: WKURLSchemeTask) {
//        guard self.eventDelegate != nil else {
//            print("Event Delegate is nil")
//            return
//        }
//        // fig://start/mattschrage/tutorialname
//        // fig://insert?cmd=
//        if let url = urlSchemeTask.request.url {
//            let cmd = url.host ?? ""
////            let path_components = url.path.split(separator: "/")
////            guard let cmd = path_components.first else {
////                print("No command in URL: \(url.absoluteString)")
////                return
////            }
//
//            switch cmd {
//            case "insert":
//                guard let script = url.queryDictionary?["cmd"] else {
//                    print("No `cmd` parameter for command 'insert'\n\(url)")
//                    return
//                }
//                self.eventDelegate!.requestInsertCLICommand(script: script)
//                break
//            case "execute":
//                guard let script = url.queryDictionary?["cmd"] else {
//                    print("No `cmd` parameter for command 'insert'\n\(url)")
//                    return
//                }
//                self.eventDelegate!.requestInsertCLICommand(script: script)
//                break
//
//            default:
//                print("Unknown command '\(cmd)' triggered")
//            }
//
//        }
//    }
//
//    func webView(_ webView: WKWebView, stop urlSchemeTask: WKURLSchemeTask) { }
//
//}
