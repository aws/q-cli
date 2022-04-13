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
// swiftlint:disable type_name
class API {
  // binary is the primary transport method.
  // json is a fallback only used in environments where the API bindings are not accessible (eg. onboarding flow)
  // note: notifications will ignore json encodings
  enum Encoding {
    case binary
    case json

    var eventName: String {
      switch self {
      case .binary:
        return "FigProtoMessageRecieved"
      case .json:
        return "FigJSONMessageRecieved"
      }
    }

    var webkitMessageHandler: String {
      switch self {
      case .binary:
        return "proto"
      case .json:
        return "protoJSON"
      }
    }
  }
  static let notifications = APINotificationCenter()
  static func handle(scriptMessage: WKScriptMessage, encoding: Encoding) {

    guard let webView = scriptMessage.webView else {
      API.log("no webview associated with API request")
      return
    }

    do {
      let request = try scriptMessage.parseAsAPIRequest(using: encoding)
      API.handle(request, from: webView, using: encoding)

    } catch APIError.generic(message: let message) {
      API.reportGlobalError(message: message, in: webView)
    } catch {
      API.reportGlobalError(message: "could not deserialize request", in: webView)
    }

  }

  static func handle(_ request: Request, from webView: WKWebView, using encoding: API.Encoding) {

    let id = request.id
    var response = Response()
    response.id = id

    var isAsync = false
    do {
      switch request.submessage {
      case .positionWindowRequest(let positionWindowRequest):
        guard let window = webView.window as? CompanionWindow else {
          throw APIError.generic(message: "No window associated with webview")
        }
        response.positionWindowResponse = try WindowPositioning.positionWindow(positionWindowRequest,
                                                                               companionWindow: window)
      case .pseudoterminalWriteRequest(let request):
        response.success = try PseudoTerminal.shared.handleWriteRequest(request)
      case .pseudoterminalExecuteRequest(let request):
        isAsync = true
        PseudoTerminal.shared.handleExecuteRequest(request, with: id) { output in
          var response = Response()
          response.id = id
          response.pseudoterminalExecuteResponse = output
          API.send(response, to: webView, using: encoding)
        }
      case .pseudoterminalRestartRequest:
        isAsync = true
        PseudoTerminal.shared.restart(with: [:], completion: { status in
          var response = Response()
          response.id = id
          response.success = status
          API.send(response, to: webView, using: encoding)
        })
      case .appendToFileRequest(let request):
        response.success = try FileSystem.appendToFile(request)
      case .readFileRequest(let request):
        response.readFileResponse = try FileSystem.readFile(request)
      case .writeFileRequest(let request):
        response.success = try FileSystem.writeFile(request)
      case .contentsOfDirectoryRequest(let request):
        response.contentsOfDirectoryResponse = try FileSystem.contentsOfDirectory(request)
      case .notificationRequest(let request):
        guard encoding == .binary else {
          throw APIError.generic(message: "Notifications must use the binary encoding.")
        }
        response.success = try API.notifications.handleRequest(id: id, request: request, for: webView)
      case .insertTextRequest(let request):
        response.success = try FigTerm.handleInsertRequest(request)
      case .getSettingsPropertyRequest(let request):
        response.getSettingsPropertyResponse = try Settings.shared.handleGetRequest(request)
      case .updateSettingsPropertyRequest(let request):
        response.success = try Settings.shared.handleSetRequest(request)
      case .updateApplicationPropertiesRequest(let request):
        response.success = try FigApp.updateProperties(request,
                                                       for: FigApp(identifier: webView.appIdentifier))
      case .destinationOfSymbolicLinkRequest(let request):
        response.destinationOfSymbolicLinkResponse = try FileSystem.destinationOfSymbolicLink(request)
      case .getDefaultsPropertyRequest(let request):
        response.getDefaultsPropertyResponse = try Defaults.shared.handleGetRequest(request)
      case .updateDefaultsPropertyRequest(let request):
        response.success = try Defaults.shared.handleSetRequest(request)
      case .telemetryAliasRequest(let request):
        response.success = try TelemetryProvider.shared.handleAliasRequest(request)
      case .telemetryIdentifyRequest(let request):
        response.success = try TelemetryProvider.shared.handleIdentifyRequest(request)
      case .telemetryTrackRequest(let request):
        response.success = try TelemetryProvider.shared.handleTrackRequest(request)
      case .onboardingRequest(let request):
        isAsync = true
        Onboarding.handleRequest(request, in: webView) { output in
          var response = Response()
          response.id = id
          response.success = output
          API.send(response, to: webView, using: encoding)
        }
      case .windowFocusRequest(let request):
        response.success = try WindowServer.handleFocusRequest(request)
      case .openInExternalApplicationRequest(let request):
        response.success = try NSWorkspace.shared.handleOpenURLRequest(request)
      case .getConfigPropertyRequest(let request):
        response.getConfigPropertyResponse = try Config.shared.handleGetRequest(request)
      case .updateConfigPropertyRequest(let request):
        response.success = try Config.shared.handleSetRequest(request)
      case .terminalSessionInfoRequest(let request):
        response.terminalSessionInfoResponse = try TerminalSessionLinker.shared.handleRequest(request)
      case .debuggerUpdateRequest(let request):
        response.success = try Diagnostic.setDebuggerStatus(request)
      case .applicationUpdateStatusRequest(let request):
        response.applicationUpdateStatusResponse = try UpdateService.provider.applicationUpdateStatusRequest(request)
      case .getLocalStateRequest(let request):
        response.getLocalStateResponse = try LocalState.shared.handleGetRequest(request)
      case .updateLocalStateRequest(let request):
        response.success = try LocalState.shared.handleSetRequest(request)
      case .runProcessRequest(let request):
        isAsync = true
        Process.handleRunProcessRequest(request) { output in
          var response = Response()
          response.id = id
          response.runProcessResponse = output
          API.send(response, to: webView, using: encoding)
        }
      case .macosInputMethodRequest(let request):
        response.macosInputMethodResponse = try InputMethod.default.handleAPIRequest(request)
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
      API.send(response, to: webView, using: encoding)
    }
  }

  static func send(_ response: Response, to webView: WKWebView, using encoding: API.Encoding) {
    assert(Thread.isMainThread)

    var payload: String!
    switch encoding {
    case .binary :
      guard let data = try? response.serializedData() else {
        return
      }

      let b64 = data.base64EncodedString()

      payload = "document.dispatchEvent(new CustomEvent('\(encoding.eventName)', {'detail': `\(b64)`}));"
    case .json:
      guard let jsonString = try? response.jsonString() else {
        return
      }

      payload = "document.dispatchEvent(new CustomEvent('\(encoding.eventName)', {'detail': \(jsonString)}));"

    }

    webView.evaluateJavaScript(payload, completionHandler: nil)
  }

  static func reportGlobalError(message: String, in webView: WKWebView,
                                file: String = #file,
                                function: String = #function,
                                line: Int = #line) {
    API.log("reporting global error: " + message)
    let source = "\(function) in \(file):\(line)"
    // swiftlint:disable line_length
    let payload = "document.dispatchEvent(new CustomEvent('FigGlobalErrorOccurred', {'detail': {'error' : '\(message)', 'source': `\(source)` } }));"
    webView.evaluateJavaScript(payload, completionHandler: nil)

  }

}

extension API: Logging {
  static func log(_ message: String) {
    Logger.log(message: message, subsystem: .api)
  }
}

extension WKScriptMessage {
  func parseAsAPIRequest(using encoding: API.Encoding) throws -> Request {
    var message: Request!
    switch encoding {
    case .binary:
      guard let b64 = self.body as? String,
            let data = Data(base64Encoded: b64) else {
        throw APIError.generic(message: "Could not convert from WKScriptMessage to data")
      }

      message = try Request(serializedData: data)
    case .json:
      guard let jsonString = self.body as? String else {
        throw APIError.generic(message: "Could not convert from WKScriptMessage to json")
      }
      message = try Request(jsonString: jsonString)
    }

    return message
  }
}
