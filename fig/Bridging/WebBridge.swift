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

class WebBridge : NSObject {
    static let shared = WebBridge()
    let processPool = WKProcessPool()

    var eventDelegate: WebBridgeEventDelegate?
    
    
    func configure(_ configuration: WKWebViewConfiguration) -> WKWebViewConfiguration {
                configuration.preferences.setValue(true, forKey: "developerExtrasEnabled")
                configuration.preferences.setValue(true, forKey: "allowFileAccessFromFileURLs")
                configuration.preferences.javaScriptEnabled = true
                configuration.preferences.javaScriptCanOpenWindowsAutomatically = true

                configuration.processPool = self.processPool
                if configuration.urlSchemeHandler(forURLScheme: "fig") == nil {
                    configuration.setURLSchemeHandler(self, forURLScheme: "fig")
                }

                let contentController = WKUserContentController()
                contentController.addUserScript(WKUserScript(source: API.declareConstants(),
                                                            injectionTime: .atDocumentStart,
                                                            forMainFrameOnly: false))
                

                contentController.add(self, name: WebBridgeScript.protobufHandler.rawValue)
                contentController.add(self, name: WebBridgeScript.protobufJSONHandler.rawValue)

                configuration.userContentController = contentController
        return configuration;
    }
}

extension WebBridge: WKURLSchemeHandler {
  func webView(_ webView: WKWebView, start urlSchemeTask: WKURLSchemeTask) {
      guard let url = urlSchemeTask.request.url else {
          return
      }
    
        guard let fileicon = Icon.fileIcon(for: url) else { return }
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
  
  func webView(_ webView: WKWebView, stop urlSchemeTask: WKURLSchemeTask) { }
}

enum WebBridgeScript: String, CaseIterable {
    case protobufHandler = "proto"
    case protobufJSONHandler = "protoJSON"
}

extension WebBridge : WKScriptMessageHandler {
    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        
        let scriptType = WebBridgeScript.init(rawValue: message.name)

        if !WebBridge.authorized(webView: message.webView) {
            message.webView?.evaluateJavaScript("console.log(`Attempted to call fig runtime from unauthorized domain`)", completionHandler: nil)
            print("Attempted to call fig runtime from unauthorized domain")
            return
        }
        switch scriptType {
        case .protobufHandler:
            API.handle(scriptMessage: message, encoding: .binary)
        case .protobufJSONHandler:
            API.handle(scriptMessage: message, encoding: .json)
        default:
            print("Unhandled WKScriptMessage type '\(message.name)'")
        }
      
    }
}

extension WebBridge {
    static func authorized(webView: WKWebView?) -> Bool {
        if let webView = webView, let url = webView.url, let scheme = url.scheme {
            print("authorized for scheme?:", scheme)
            return scheme == "file" || url.host == Remote.baseURL.host || url.host == "fig.run" || url.host ?? "" == "localhost"
        }
        return false
    }
}
