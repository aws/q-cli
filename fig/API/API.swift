//
//  API.swift
//  fig
//
//  Created by Matt Schrage on 8/24/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import WebKit

typealias Request = Fig_ClientOriginatedMessage
typealias Response = Fig_ServerOriginatedMessage

class API {
    let broker = API()

    static func handle(scriptMessage: WKScriptMessage) {
        
        guard let webView = scriptMessage.webView else {
            API.log("no webview associated with API request")
            return
        }
        
        do {
            let request = try scriptMessage.parseAsAPIRequest()
            API.handle(request, in: webView)

        } catch APIError.generic(message: let message) {
            API.reportGlobalError(message: message, in: webView)
        } catch {
            API.reportGlobalError(message: "could not deserialize request", in: webView)
        }
        
    }
    
    static func handle(_ request: Request, in webView: WKWebView) {
        
        print(request.id, String(reflecting: request.submessage))
        var response = Response()
        response.id = request.id

        do {
            switch request.submessage {
                case .getBufferRequest(let getBufferRequest):
                    print("API: getBufferRequest")
                case .positionWindowRequest(let positionWindowRequest):
                    print("API: positionWindowRequest")
                    response.positionWindowResponse = try WindowPositioning.positionWindow(positionWindowRequest)
                case .ptyRequest(let ptyRequest):
                    print("API: ptyRequest")
                default:
                    print("API: unknown request")
            }
        } catch APIError.generic(message: let message) {
            response.error = message
        } catch {
            response.error = "An unknown error occured."
        }
        
        API.send(response, to: webView)
    }
    
    static func send(_ response: Response, to webView: WKWebView) {
        guard let data = try? response.serializedData() else {
            return
        }
        
        let b64 = data.base64EncodedString()
        
        let payload = "document.dispatchEvent(new CustomEvent('FigProtoMessageRecieved', {'detail': `\(b64)`}));"
        
        webView.evaluateJavaScript(payload, completionHandler: nil)
    }
    
    static func reportGlobalError(message: String, in webView: WKWebView) {
        API.log("reporting global error: " + message)
        
        let payload = "document.dispatchEvent(new CustomEvent('FigGlobalErrorOccured', {'detail': {'error' : '\(message)' } }));"
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
