//
//  API.swift
//  fig
//
//  Created by Matt Schrage on 8/24/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import WebKit
import FigAPIBindings

typealias Request = Fig_ClientOriginatedMessage
typealias Response = Fig_ServerOriginatedMessage
typealias NotificationRequest = Fig_NotificationRequest
class API {
    static let notifications = APINotificationCenter()
    static func handle(scriptMessage: WKScriptMessage) {
        
        guard let webView = scriptMessage.webView else {
            API.log("no webview associated with API request")
            return
        }
        
        do {
            let request = try scriptMessage.parseAsAPIRequest()
            API.handle(request, from: webView)

        } catch APIError.generic(message: let message) {
            API.reportGlobalError(message: message, in: webView)
        } catch {
            API.reportGlobalError(message: "could not deserialize request", in: webView)
        }
        
    }
    
    static func handle(_ request: Request, from webView: WKWebView) {
        
        var response = Response()
        response.id = request.id

        var isAsync = false
        do {
            switch request.submessage {
                case .positionWindowRequest(let positionWindowRequest):
                    response.positionWindowResponse = try WindowPositioning.positionWindow(positionWindowRequest)
                case .pseudoterminalWriteRequest(let request):
                    response.success = true
                case .pseudoterminalExecuteRequest(let request):
                    isAsync = true
                case .readFileRequest(let request):
                    response.readFileResponse = try FileSystem.readFile(request)
                case .writeFileRequest(let request):
                    response.success = try FileSystem.writeFile(request)
                case .contentsOfDirectoryRequest(let request):
                    response.contentsOfDirectoryResponse = try FileSystem.contentsOfDirectory(request)
                case .notificationRequest(let request):
                    response.success = try API.notifications.handleRequest(id: response.id, request: request, for: webView)
                case .insertTextRequest(let request):
                    ShellBridge.injectStringIntoTerminal(request.text)
                    response.success = true
                case .getSettingsPropertyRequest(let request):
                    response.getSettingsPropertyResponse = try Settings.shared.handleGetRequest(request)
                case .updateSettingsPropertyRequest(let request):
                    response.success = try Settings.shared.handleSetRequest(request)
                case .updateApplicationPropertiesReuest(let request):
                    if request.hasInterceptBoundKeystrokes {
                        KeypressProvider.shared.setEnabled(value: request.interceptBoundKeystrokes)
                    }
                    response.success = true
                case .none:
                    throw APIError.generic(message: "No submessage was included in request.")
                
            }
        } catch APIError.generic(message: let message) {
            response.error = message
        } catch {
            response.error = "An unknown error occured."
        }
        
        // Send response immediately if request is completed synchronously
        if !isAsync {
            API.send(response, to: webView)
        }
    }
    
    static func send(_ response: Response, to webView: WKWebView) {
        guard let data = try? response.serializedData() else {
            return
        }
        
        let b64 = data.base64EncodedString()
        
        let payload = "document.dispatchEvent(new CustomEvent('FigProtoMessageRecieved', {'detail': `\(b64)`}));"
        
        webView.evaluateJavaScript(payload, completionHandler: nil)
    }
    
    static func reportGlobalError(message: String, in webView: WKWebView,
                                  file: String = #file,
                                  function: String = #function,
                                  line: Int = #line) {
        API.log("reporting global error: " + message)
        let source = "\(function) in \(file):\(line)"
        let payload = "document.dispatchEvent(new CustomEvent('FigGlobalErrorOccurred', {'detail': {'error' : '\(message)', '', 'source': '\(source)' } }));"
        webView.evaluateJavaScript(payload, completionHandler: nil)

    }
    
}

extension API: Logging {
    static func log(_ message: String) {
        Logger.log(message: message, subsystem: .api)
    }
}

extension WKScriptMessage {
    func parseAsAPIRequest() throws -> Request  {
        
        guard let b64 = self.body as? String,
              let data = Data(base64Encoded: b64) else {
            throw APIError.generic(message: "Could not convert from WKScriptMessage to string")
        }
                
        let message = try Request(serializedData: data)
        
        return message
    }
}

// todo: move to Settings.swift once merged
extension Settings {
    func handleGetRequest(_ request: Fig_GetSettingsPropertyRequest) throws -> Fig_GetSettingsPropertyResponse {
        guard request.hasKey else {
            throw APIError.generic(message: "No key provided with request")
        }
        
        guard let value = Settings.shared.getValue(forKey: request.key) else {
            throw APIError.generic(message: "No value for key")
        }
        
        guard let data = try? JSONSerialization.data(withJSONObject: value, options: .prettyPrinted) else {
            throw APIError.generic(message: "Could not convert value for key to JSON")
        }
                
        return Fig_GetSettingsPropertyResponse.with {
            $0.jsonBlob = String(decoding: data, as: UTF8.self)
        }
    }
    
    func handleSetRequest(_ request: Fig_UpdateSettingsPropertyRequest) throws -> Bool {
        guard request.hasKey else {
            throw APIError.generic(message: "No key provided with request")
        }
        
        
        if request.hasValue {
            Settings.shared.set(value: request.value, forKey: request.key)
        } else {
            // todo: uncomment once merged with more recent settings API
            // Settings.shared.set(value: nil, forKey: request.key)
        }
        
        return true
        
    }
}
