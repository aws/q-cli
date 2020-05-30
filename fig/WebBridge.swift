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

    case enforceViewportSizing = "enforceViewportSizing"

}

class WebBridgeContentController : WKUserContentController {
    override init() {
        super.init()
        
//        let legacy: [WebBridgeScript]  = [ .insertFigTutorialCSS, .figJS ]
//        let scripts: [WebBridgeScript] = [.logging, .exceptions, .figJS]
       
        self.addWebBridgeScript(.exceptions)
        self.addWebBridgeScript(.logging);
//        self.addWebBridgeScript(.insertFigTutorialCSS);
//        self.addWebBridgeScript(.insertFigTutorialJS);

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
        let body = scope.body as! String
        print("JS Console: \(body)")
        Logger.log(message: "\(scope.webView?.url?.absoluteString ?? "<none>"): \(body)\n")
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
            
            let output = cmd.runAsCommand(cwd: pwd)
            print("\(cmd) -> \(output)")
            let encoded = output.data(using: .utf8)!
            scope.webView?.evaluateJavaScript("fig.callback(`\(handlerId)`,`\(encoded.base64EncodedString())`)", completionHandler: nil)

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
                scope.webView?.evaluateJavaScript("fig.callback(`\(handlerId)`, null,{message:'Could not write file to disk.'})", completionHandler: nil)
            }
        }
    }
    
    static func focus(scope: WKScriptMessage) {
        NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
    }
    
    static func blur(scope: WKScriptMessage) {
        ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
    }

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
                return "function captureException(msg) { window.webkit.messageHandlers.exceptionHandler.postMessage(msg) } window.onerror = captureException;"
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
