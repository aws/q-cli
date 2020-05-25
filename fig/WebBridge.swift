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
    func requestNextSection()
    func requestPreviousSection()
    func startTutorial(identifier: String)
}

class WebBridge : WKWebViewConfiguration {
    var eventDelegate: WebBridgeEventDelegate?
    
    convenience init(eventDelegate: WebBridgeEventDelegate) {
        self.init()
        self.eventDelegate = eventDelegate;
    }
    
    override init() {
        super.init()
        let contentController = WebBridgeContentController()
        contentController.add(self, name: WebBridgeScript.executeCLIHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.insertCLIHandler.rawValue)
        contentController.add(self, name: WebBridgeScript.callbackHandler.rawValue)

        contentController.add(self, name: WebBridgeScript.logging.rawValue)
        self.userContentController = contentController
        self.setURLSchemeHandler(self, forURLScheme: "fig")
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    
}

extension WebBridge : WKURLSchemeHandler {
    func webView(_ webView: WKWebView, start urlSchemeTask: WKURLSchemeTask) {
        guard self.eventDelegate != nil else {
            print("Event Delegate is nil")
            return
        }
        // fig://start/mattschrage/tutorialname
        // fig://insert?cmd=
        if let url = urlSchemeTask.request.url {
            let cmd = url.host ?? ""
//            let path_components = url.path.split(separator: "/")
//            guard let cmd = path_components.first else {
//                print("No command in URL: \(url.absoluteString)")
//                return
//            }
  
            switch cmd {
            case "next":
                self.eventDelegate!.requestNextSection()
                break
            case "back":
                self.eventDelegate!.requestPreviousSection()
                break
            case "start":
                self.eventDelegate!.startTutorial(identifier: url.pathComponents.dropFirst().joined(separator: "/"))
                break
            case "insert":
                guard let script = url.queryDictionary?["cmd"] else {
                    print("No `cmd` parameter for command 'insert'\n\(url)")
                    return
                }
                self.eventDelegate!.requestInsertCLICommand(script: script)
                break
            case "execute":
                guard let script = url.queryDictionary?["cmd"] else {
                    print("No `cmd` parameter for command 'insert'\n\(url)")
                    return
                }
                self.eventDelegate!.requestInsertCLICommand(script: script)
                break

            default:
                print("Unknown command '\(cmd)' triggered")
            }
            
        }
    }
    
    func webView(_ webView: WKWebView, stop urlSchemeTask: WKURLSchemeTask) { }

}

enum WebBridgeScript: String {
    case logging = "logHandler"
    case exceptions = "exceptionHandler"
    case insertFigTutorialCSS = "css"
    case insertFigJS = "js"
    case insertFigTutorialJS = "tutorial"
    case insertCLIHandler = "insertHandler"
    case executeCLIHandler = "executeHandler"
    case callbackHandler = "callbackHandler"
    case stdoutHandler = "stdoutHandler"

}

class WebBridgeContentController : WKUserContentController {
    override init() {
        super.init()
       
        self.addWebBridgeScript(.logging);
        self.addWebBridgeScript(.insertFigTutorialCSS);
        self.addWebBridgeScript(.insertFigTutorialJS);
//        self.addWebBridgeScript(.executeCLIHandler)
        self.addWebBridgeScript(.insertFigJS, location: .atDocumentStart);





//        let source = "function captureLog(msg) { window.webkit.messageHandlers.logHandler.postMessage(msg); } window.console.log = captureLog;"
//        let script = WKUserScript(source: source, injectionTime: .atDocumentEnd, forMainFrameOnly: false)
//        self.addUserScript(script)
//        self.add(self, name: WebBridgeScript.logging.rawValue)
    }
    
    func addWebBridgeScript(_ scriptType:WebBridgeScript,  location: WKUserScriptInjectionTime = .atDocumentEnd) {
        var source: String = ""
        switch scriptType {//window.onerror = function (errorMsg, url, lineNumber) {
           // alert('Error: ' + errorMsg + ' Script: ' + url + ' Line: ' + lineNumber);
        case .exceptions:
            source = " function captureException(msg) { window.webkit.messageHandlers.exceptionHandler.postMessage(msg) } window.onerror = captureException;"
            break
        case .logging:
            source = "function captureLog(msg) { window.webkit.messageHandlers.logHandler.postMessage(msg); } window.console.log = captureLog;"
            break
        case .insertFigJS:
            source = File.contentsOfFile("fig", type: "js")!
        case .insertFigTutorialJS:
            source = File.contentsOfFile("tutorial", type: "js")!
            source = """
            //
            function preFormatting(preNode) {

                              var wrapperDiv = document.createElement('div');
                              wrapperDiv.classList.add("wrapper");


                              preNode.parentNode.insertBefore(wrapperDiv, preNode);
                              wrapperDiv.appendChild(preNode);

                              // Set up pear button
                              var pearButton = document.createElement('button');
                              pearButton.classList.add("pearButton");
                              pearButton.classList.add("prePearButton");
                              pearButton.innerHTML = '🍐';

                              preNode.parentNode.insertBefore(pearButton, preNode);

                              // #### Events ####

                              // Show pear on mouse enter
                              wrapperDiv.addEventListener('mouseenter', function (e) {
                                pearButton.classList.add("buttonShow");
                              });

                              // Hide pear on mouse leave
                              wrapperDiv.addEventListener('mouseleave', function (e) {
                                pearButton.classList.remove("buttonShow");
                              });

                              // Add click event listener to pear
                              pearButton.addEventListener('click', function (e) {
                                //e.preventDefault();
                                //e.stopPropagation();

                                var deepLink = "fig://insert?cmd=" + preNode.innerText;
                                console.log("Pear: " + deepLink);
                                console.log(JSON.stringify(webkit))

                                window.webkit.messageHandlers.executeHandler.postMessage(preNode.innerText)
                                // This should insert and run the code
                                //window.location.href = deeplink
                              });

                              // Add event listener to copy code on click (but not highlight)
                              preNode.addEventListener('click', function (e) {
                                //e.preventDefault();
                                //e.stopPropagation();

                                if (window.getSelection().toString() === "") {
                                  var deepLink = "fig://insert?cmd=" + preNode.innerText;
                                  console.log("Insert: " + deepLink);

                                  // This should just insert the code, NOT run it
                                window.webkit.messageHandlers.insertHandler.postMessage(preNode.innerText)

                                  //window.location.href = deeplink
                                }

                                else {
                                  console.log("Highlight: " + window.getSelection().toString())
                                }


                              });
                            }




                            // #### Apply formatting to <code> node

                            function codeFormatting(codeNode) {

                              var wrapperSpan = document.createElement('span');
                              wrapperSpan.classList.add("wrapper");


                              codeNode.parentNode.insertBefore(wrapperSpan, codeNode);
                              wrapperSpan.appendChild(codeNode);

                              // Set up pear button
                              var pearButton = document.createElement('button');
                              pearButton.classList.add("pearButton");
                              pearButton.classList.add("inlinePearButton");
                              pearButton.innerHTML = '🍐';

                              codeNode.parentNode.insertBefore(pearButton, codeNode);

                              // #### Events ####

                              // Show pear on mouse enter
                              wrapperSpan.addEventListener('mouseenter', function (e) {
                                pearButton.classList.add("buttonShow");
                              });

                              // Hide pear on mouse leave
                              wrapperSpan.addEventListener('mouseleave', function (e) {
                                pearButton.classList.remove("buttonShow");
                              });

                              // Add click event listener to pear
                              pearButton.addEventListener('click', function (e) {
                                //e.preventDefault();
                                //e.stopPropagation();

                                var deepLink = "fig://insert?cmd=" + encodeURI(codeNode.innerText);
                                console.log("Pear: " + deepLink);
                                window.webkit.messageHandlers.executeHandler.postMessage(codeNode.innerText)


                                // This should insert and run the code
                                //window.location.href = deeplink
                              });

                              // Add event listener to copy code on click (but not highlight)
                              codeNode.addEventListener('click', function (e) {
                                //e.preventDefault();
                                //e.stopPropagation();

                                if (window.getSelection().toString() === "") {
                                  var deepLink = "fig://insert?cmd=" + codeNode.innerText;
                                  console.log("Insert: " + deepLink);

                                  // This should just insert the code, NOT run it
                                  //window.location.href = deeplink
                                    window.webkit.messageHandlers.insertHandler.postMessage(codeNode.innerText)

                                }

                                else {
                                  console.log("Highlight: " + window.getSelection().toString())
                                }

                              });
                            }



                            // #### Adds a 🍐on hover for <code> and <pre> tags
                            function addATouchOfFig() {

                              // Loop through <code> element
                              // (and exclude code elements that have <pre> as a parent)
                              var codes = document.querySelectorAll('code');
                              codes.forEach(function (codeNode) {

                                // Check if code is wrapped in <pre> or just <code>
                                if (codeNode.parentNode.nodeName !== "PRE") {
                                  codeFormatting(codeNode);
                                }
                              });

                              // Loop through <pre> element
                              var pres = document.querySelectorAll('pre');
                              pres.forEach(function (preNode) {
                                preFormatting(preNode);
                              });

                              console.log("finished adding fig touchups");
                            }
            addATouchOfFig()
            console.log(JSON.stringify(window.webkit))

            """
        case .insertFigTutorialCSS:
            var cssString = """
            body {
              margin: 10px;
            }

            code {
              cursor: pointer;
            }

            pre {
              cursor: pointer;
            }

            .buttonShow {
              opacity: 1 !important;
            }

            .wrapper {
              position: relative !important;
              overflow: visible !important;
            }

            .pearButton {
              cursor: pointer;
              position: absolute;
              font-size: 20px;
              background: white;
              border: none;
              border-radius: 5px;
              transition: all 0.1s ease-in-out;
              opacity: 0;
              z-index: 10;
            }

            .prePearButton {
              right: 5px;
              top: 5px;
            }

            .inlinePearButton {
              right: -36px;
              bottom: -7px;
            }

            button:focus {
              outline:0;
            }

            """
//            let cssString = " * { color: cyan; }"
            cssString = File.contentsOfFile("tutorial", type: "css")!
            source = """
                  var style = document.createElement('style');
                  style.innerHTML = `\(cssString)`;
                  document.head.appendChild(style);
               """
            
        default:
            return
        }
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
        case .logging:
            print("JS Console: \(message.body)")
            break
        case .insertCLIHandler:
//            print("Insert: \(message.body)")
            self.eventDelegate?.requestInsertCLICommand(script: message.body as! String)

            break
        case .executeCLIHandler:
//            print("Execute: \(message.body)")
            self.eventDelegate?.requestExecuteCLICommand(script: message.body as! String)
            break
        case .callbackHandler:
            print(message.body)
            let dict = message.body as! Dictionary<String, String>
            let callback = WebBridgeJSCallback(type: dict["type"]!, cmd: dict["cmd"]!, handlerId: dict["handlerId"]!)
            let figCLIPath = "fig"//"python3 /Users/mschrage/fig/research/pyfigcli/fig.py"
            self.eventDelegate?.requestExecuteCLICommand(script: "\(callback.cmd) | \(figCLIPath) callback \(callback.handlerId)")
        default:
            print("Unhandled WKScriptMessage type '\(message.name)'")
        }
      
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
