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
    
//    init(eventDelegate: WebBridgeEventDelegate?) {
//        self.init()
//        self.eventDelegate = eventDelegate;
//    }
    
    func configure(_ configuration: WKWebViewConfiguration) -> WKWebViewConfiguration {
                configuration.preferences.setValue(true, forKey: "developerExtrasEnabled")
                configuration.preferences.setValue(true, forKey: "allowFileAccessFromFileURLs")
                configuration.preferences.javaScriptEnabled = true
                configuration.preferences.javaScriptCanOpenWindowsAutomatically = true
        //        self.preferences.setValue(true, forKey: "mediaPreloadingEnabled")
        //        self.preferences.setValue(true, forKey: "linkPreloadEnabled")

        //        self.webView.configuration.preferences
                configuration.processPool = self.processPool
                if configuration.urlSchemeHandler(forURLScheme: "fig") == nil {
                    configuration.setURLSchemeHandler(self, forURLScheme: "fig")
                }
//                configuration.setURLSchemeHandler(self, forURLScheme: "figbundle")

                let contentController = WebBridgeContentController()
                
                let _: [WebBridgeEventHandler] = [.logHandler,
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
                contentController.add(self, name: WebBridgeScript.propertyUpdateHandler.rawValue)
                contentController.add(self, name: WebBridgeScript.ptyHandler.rawValue)
                contentController.add(self, name: WebBridgeScript.notificationHandler.rawValue)
                contentController.add(self, name: WebBridgeScript.detachHandler.rawValue)
                contentController.add(self, name: WebBridgeScript.globalExecuteHandler.rawValue)
                contentController.add(self, name: WebBridgeScript.stdoutHandler.rawValue)
                contentController.add(self, name: WebBridgeScript.privateHandler.rawValue)

                contentController.add(self, name: WebBridgeScript.onboardingHandler.rawValue)

                contentController.add(self, name: WebBridgeScript.logging.rawValue)
                contentController.add(self, name: WebBridgeScript.exceptions.rawValue)

                configuration.userContentController = contentController
        return configuration;
    }
    
//
//    override init() {
//        super.init()
//        self.preferences.setValue(true, forKey: "developerExtrasEnabled")
//        self.preferences.setValue(true, forKey: "allowFileAccessFromFileURLs")
//        self.preferences.javaScriptEnabled = true
//        self.preferences.javaScriptCanOpenWindowsAutomatically = true
////        self.preferences.setValue(true, forKey: "mediaPreloadingEnabled")
////        self.preferences.setValue(true, forKey: "linkPreloadEnabled")
//
////        self.webView.configuration.preferences
//        self.processPool = (NSApp.delegate as! AppDelegate).processPool
//        self.setURLSchemeHandler(self, forURLScheme: "fig")
//        self.setURLSchemeHandler(self, forURLScheme: "figbundle")
//
//        let contentController = WebBridgeContentController()
//
//        let eventHandlers: [WebBridgeEventHandler] = [.logHandler,
//                                                      .exceptionHandler,
//                                                      .insertHandler,
//                                                      .executeHandler,
//                                                      .executeInBackgroundHandler,
//                                                      .callbackHandler]
//
//        contentController.add(self, name: WebBridgeScript.executeCLIHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.insertCLIHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.callbackHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.fwriteHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.freadHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.focusHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.blurHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.appwriteHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.appreadHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.positionHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.openHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.streamHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.defaultsHandler.rawValue)
//        contentController.add(self, name: WebBridgeScript.normalizeFilePath.rawValue)
//        contentController.add(self, name: WebBridgeScript.propertyUpdateHandler.rawValue)
//
//        contentController.add(self, name: WebBridgeScript.onboardingHandler.rawValue)
//
//        contentController.add(self, name: WebBridgeScript.logging.rawValue)
//        contentController.add(self, name: WebBridgeScript.exceptions.rawValue)
//
//        self.userContentController = contentController
////        self.setURLSchemeHandler(self, forURLScheme: "fig")
//    }
    
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
    case insertableCodeTags = "insert-tutorial"
    case runnableCodeTags = "run-tutorial"
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
    case propertyUpdateHandler = "propertyUpdateHandler"
    case ptyHandler = "ptyHandler"
    case notificationHandler = "notificationHandler"
    case detachHandler = "detachHandler"
    case globalExecuteHandler = "globalExecuteHandler"
    case privateHandler = "privateHandler"

    case onboardingHandler = "onboardingHandler"

    case enforceViewportSizing = "enforceViewportSizing"

}

//fig:iconForType:zip
extension WebBridge: WKURLSchemeHandler {
    static func fileIcon(for url: URL) -> NSImage? {
        
        var width = 32.0
        var height = 32.0
        var color: NSColor?
        var badge: String?

        if let qs = url.queryDictionary, let w = qs["w"], let wd = Double(w), let h = qs["h"], let hd = Double(h) {
            width = wd
            height = hd
        }
        
        if let qs = url.queryDictionary {
            color = NSColor(hex: qs["color"] ?? "")
            badge = qs["badge"]
        }
        
        // fig://template?background-color=ccc&icon=box
        if let host = url.host, host == "template" {
          guard let icon = Bundle.main.image(forResource: "template") else { return nil }
          return icon.overlayColor(color).overlayText(badge).resized(to: NSSize(width: width, height: height))//?.overlayBadge(color: color,  text: badge)


        }
      
        // fig://icon?asset=git
        if let host = url.host, let qs = url.queryDictionary, let type = qs["asset"], host == "icon" {
            let icon = Bundle.main.image(forResource: type) ?? Bundle.main.image(forResource: "box")!
            return icon.resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)
        }

        // fig://icon?type=mp4
        if let host = url.host, let qs = url.queryDictionary, let type = qs["type"], host == "icon" {
            if let icon = Bundle.main.image(forResource: type) {
                return icon.resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)
            }
            
            var t = type
            if (type == "folder") {
                t = NSFileTypeForHFSTypeCode(OSType(kGenericFolderIcon))
            }
            return NSWorkspace.shared.icon(forFileType: t).resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)

        }
        
        if let host = url.host, let qs = url.queryDictionary, let pid = qs["pid"], host == "icon" {

            return NSRunningApplication(processIdentifier: pid_t(pid) ?? -1)?.icon?.resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)
        }

        guard var specifier = (url as NSURL).resourceSpecifier else { return nil }
        if (specifier.prefix(2) == "//") { specifier = String(specifier.dropFirst(2)) }
//        if (specifier.prefix(1) !=  "/") { specifier = "/" + specifier }
        let resource = specifier.replacingOccurrences(of: "?\(url.query ?? "<none>")", with: "") as NSString
        let fullPath = resource.expandingTildeInPath.removingPercentEncoding ?? ""
        
        var isDirectory : ObjCBool = false
        let isFile = FileManager.default.fileExists(atPath: fullPath, isDirectory:&isDirectory)
        guard isFile || isDirectory.boolValue else {
            var t = NSString(string: fullPath).pathExtension
            if (String(resource).last == "/") {
                t = NSFileTypeForHFSTypeCode(OSType(kGenericFolderIcon))
            }
            
            return NSWorkspace.shared.icon(forFileType: t).resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)
        }
        
        return NSWorkspace.shared.icon(forFile: fullPath).resized(to: NSSize(width: width, height: height))?.overlayBadge(color: color,  text: badge)
        
    }
    
    func webView(_ webView: WKWebView, start urlSchemeTask: WKURLSchemeTask) {
        guard let url = urlSchemeTask.request.url else {
            return
        }
//        DispatchQueue(label: "com.withfig.icon-fetcher", qos: .userInitiated, attributes: .concurrent).async {
          guard let fileicon = WebBridge.fileIcon(for: url) else { return }
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
//        }
    }
    
    func webView(_ webView: WKWebView, stop urlSchemeTask: WKURLSchemeTask) {
        
    }
}

extension NSImage {
    func resized(to newSize: NSSize) -> NSImage? {
        if let rep = self.bestRepresentation(for: NSRect(origin: .zero, size: newSize), context: NSGraphicsContext.current, hints: nil)/*NSBitmapImageRep(
            bitmapDataPlanes: nil, pixelsWide: Int(newSize.width), pixelsHigh: Int(newSize.height),
            bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
            colorSpaceName: .calibratedRGB, bytesPerRow: 0, bitsPerPixel: 0
        ) */{
//            bitmapRep.size = newSize
//
////            let screenScale = NSScreen.main?.backingScaleFactor ?? 1.0
////            NSBitmapImageRep
//
////            float targetScaledWidth = sourceImage.size.width*scale/screenScale;
////            float targetScaledHeight = sourceImage.size.height*scale/screenScale;
//            NSGraphicsContext.saveGraphicsState()
//            NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmapRep)
//            draw(in: NSRect(x: 0, y: 0, width: newSize.width, height: newSize.height), from: .zero, operation: .copy, fraction: 1.0)
//            NSGraphicsContext.restoreGraphicsState()

            let resizedImage = NSImage(size: newSize)
            resizedImage.addRepresentation(rep)
            return resizedImage
        }

        return nil
    }
    
    func overlayAppIcon() -> NSImage {
        let background = self
        // let side:CGFloat = 32

        let overlay = NSImage(imageLiteralResourceName: NSImage.applicationIconName)//.resized(to: NSSize(width:  background.size.width/2, height:  background.size.height/2))!
        
        let newImage = NSImage(size: background.size)
        newImage.lockFocus()

        var newImageRect: CGRect = .zero
        newImageRect.size = newImage.size
        
        background.draw(in: newImageRect)
        overlay.draw(in: NSRect(x: background.size.width/2, y: 0, width: background.size.width/2 - 4, height: background.size.height/2 - 4))

        newImage.unlockFocus()
        return newImage//.resized(to: NSSize(width: background.size.width * 1.5, height: background.size.height * 1.5))!
    }
  
  func overlayImage(_ image: NSImage) -> NSImage {
        let background = self
        // let side:CGFloat = 32

        let overlay = image//.resized(to: NSSize(width:  background.size.width/2, height:  background.size.height/2))!
        
        let newImage = NSImage(size: background.size)
        newImage.lockFocus()

        var newImageRect: CGRect = .zero
        newImageRect.size = newImage.size
        
        background.draw(in: newImageRect)
        overlay.draw(in: NSRect(x: background.size.width/2, y: 0, width: background.size.width/2 - 4, height: background.size.height/2 - 4))

        newImage.unlockFocus()
        return newImage//.resized(to: NSSize(width: background.size.width * 1.5, height: background.size.height * 1.5))!
    }
  
    func overlayColor(_ color: NSColor?) -> NSImage {
      guard let color = color, let bitmapRep = NSBitmapImageRep(bitmapDataPlanes: nil,
                                                           pixelsWide: Int(self.size.width),
                                                           pixelsHigh: Int(self.size.height),
                                                           bitsPerSample: 8,
                                                           samplesPerPixel: 4,
                                                           hasAlpha: true,
                                                           isPlanar: false,
                                                           colorSpaceName: .calibratedRGB,
                                                           bytesPerRow: 0,
                                                           bitsPerPixel: 0) else { return self }
      bitmapRep.size = self.size
      NSGraphicsContext.saveGraphicsState()
      NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmapRep)
      draw(in: NSRect(x: 0, y: 0, width: self.size.width, height: self.size.height), from: .zero, operation: .copy, fraction: 1.0)
      NSGraphicsContext.restoreGraphicsState()

      self.addRepresentation(bitmapRep)
      
      guard let cgImage = self.cgImage(forProposedRect: nil, context: nil, hints: nil) else { return self }

      self.lockFocus()
      color.set()
      
      guard let context = NSGraphicsContext.current?.cgContext else { return self }
      let imageRect = NSRect(origin: NSZeroPoint, size: self.size)

      context.clip(to: imageRect, mask: cgImage)
      imageRect.fill(using: .darken)
      self.unlockFocus()

      
      return self
    }
  
  func overlayText(_ text: String?) -> NSImage {
    guard let text = text, let bitmapRep = NSBitmapImageRep(bitmapDataPlanes: nil,
                                                         pixelsWide: Int(self.size.width),
                                                         pixelsHigh: Int(self.size.height),
                                                         bitsPerSample: 8,
                                                         samplesPerPixel: 4,
                                                         hasAlpha: true,
                                                         isPlanar: false,
                                                         colorSpaceName: .calibratedRGB,
                                                         bytesPerRow: 0,
                                                         bitsPerPixel: 0) else { return self }
    bitmapRep.size = self.size
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmapRep)
    draw(in: NSRect(x: 0, y: 0, width: self.size.width, height: self.size.height), from: .zero, operation: .copy, fraction: 1.0)
    NSGraphicsContext.restoreGraphicsState()

    self.addRepresentation(bitmapRep)
    
    self.lockFocus()
    
    let imageRect = NSRect(origin: NSZeroPoint, size: self.size)
    let paragraphStyle: NSMutableParagraphStyle = NSMutableParagraphStyle()
    paragraphStyle.alignment = NSTextAlignment.center
    
    let string = NSAttributedString(string: text,
                                    attributes: [ NSAttributedString.Key.font : NSFont.systemFont(ofSize: floor(imageRect.height * 0.65)),
                                                  NSAttributedString.Key.foregroundColor : NSColor.white,
                                                  NSAttributedString.Key.paragraphStyle : paragraphStyle])

    
    string.draw(in: imageRect.insetBy(dx: 0, dy: imageRect.height * 0.1))
    self.unlockFocus()
    return self
  }
  
    func overlayBadge(color: NSColor?, text: String?) -> NSImage {
        guard color != nil || text != nil else {
            return self
        }
        
        if let bitmapRep = NSBitmapImageRep(
            bitmapDataPlanes: nil, pixelsWide: Int(self.size.width), pixelsHigh: Int(self.size.height),
            bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
            colorSpaceName: .calibratedRGB, bytesPerRow: 0, bitsPerPixel: 0
        ) {
            bitmapRep.size = self.size
            NSGraphicsContext.saveGraphicsState()
            NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmapRep)
            draw(in: NSRect(x: 0, y: 0, width: self.size.width, height: self.size.height), from: .zero, operation: .copy, fraction: 1.0)
            NSGraphicsContext.restoreGraphicsState()

            self.addRepresentation(bitmapRep)
            self.lockFocus()

             let rect = NSMakeRect(size.width/2, 0, size.width/2, size.height/2)
             let ctx = NSGraphicsContext.current?.cgContext
//             ctx!.clear(rect)
            ctx!.setFillColor((color ?? NSColor.clear).cgColor)
             ctx!.fillEllipse(in: rect)
            
            if let text = text {
                let paragraphStyle: NSMutableParagraphStyle = NSMutableParagraphStyle()
                paragraphStyle.alignment = NSTextAlignment.center
                
                let string = NSAttributedString(string: text,
                                                attributes: [ NSAttributedString.Key.font : NSFont.systemFont(ofSize: rect.height * 0.9),
                                                              NSAttributedString.Key.foregroundColor : NSColor.white,
                                                              NSAttributedString.Key.paragraphStyle : paragraphStyle])

                
                string.draw(in: rect)
            }

            self.unlockFocus()
            return self
        }
        
        return self
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
        
        //this is now injected from the WebView
        //self.addWebBridgeScript(.insertFigTutorialJS);
        
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

        if !WebBridge.authorized(webView: message.webView) && scriptType != .insertCLIHandler {
            message.webView?.evaluateJavaScript("console.log(`Attempted to call fig runtime from unauthorized domain`)", completionHandler: nil)
            print("Attempted to call fig runtime from unauthorized domain")
            return
        }
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
        case .propertyUpdateHandler:
            WebBridge.propertyValueChanged(scope: message)
        case .ptyHandler:
            WebBridge.pty(scope: message)
        case .notificationHandler:
            WebBridge.notification(scope: message)
        case .detachHandler:
            WebBridge.detach(scope: message)
        case .globalExecuteHandler:
            WebBridge.executeInGlobalScope(scope: message)
        case .stdoutHandler:
            WebBridge.stdout(scope: message)
        case .privateHandler:
            WebBridge.privateAPI(scope: message)
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
    static func authorized(webView: WKWebView?) -> Bool {
        if let webView = webView, let url = webView.url, let scheme = url.scheme {
            print("authorized for scheme?:", scheme)
            return scheme == "file" || url.host == Remote.baseURL.host || url.host == "fig.run" || url.host ?? "" == "localhost"
        }
        return false
    }
    
    static func log(scope: WKScriptMessage) {
        let body = scope.body as? String
        if let body = body {
            print("JS Console: \(body)")
            Logger.log(message: "\(scope.webView?.url?.absoluteString ?? "<none>"): \(body)", subsystem: .javascript)
        } else {
            print("JS Console: Tried to write something that wasn't a string")
            Logger.log(message: "\(scope.webView?.url?.absoluteString ?? "<none>"): Attempted to write something that wasn't a string to the fig log.\n\nUse `fig.log()` in the future to avoid this error or `JSON.stringify()` any input passed into `console.log`.")
        }

    }
    
    static func insert(scope: WKScriptMessage) {
        if let webview = scope.webView, let window = webview.window, let controller = window.contentViewController as? WebViewController {
            let hack = Notification(name: .insertCommandInTerminal, object: scope.body as! String, userInfo: nil)
            controller.insertCommandInTerminal(hack)
            NotificationCenter.default.post(hack)
        }
    }
    
    static func execute(scope: WKScriptMessage) {
//        NotificationCenter.default.post(name: .executeCommandInTerminal, object: scope.body as! String)
        if let webview = scope.webView, let window = webview.window, let controller = window.contentViewController as? WebViewController {
            let hack = Notification(name: .executeCommandInTerminal, object: scope.body as! String, userInfo: nil)
            controller.executeCommandInTerminal(hack)
            
            // Check for sidebar shortcut
            if let companion = window as? CompanionWindow, companion.isSidebar {
                TelemetryProvider.track(event: .selectedShortcut, with: [:])
            }
        }
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
            Logger.log(message: "Couldn't execute \(scope.body)", subsystem: .javascript)
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
    
    static func executeInGlobalScope(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let cmd = params["cmd"],
           let handlerId = params["handlerId"] {
            cmd.runInBackground(completion:  { (result) in
                DispatchQueue.main.async {
                    WebBridge.callback(handler: handlerId, value: result, webView: scope.webView)
                }
            })
        }
    }
    
    static func fwrite(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let path = params["path"],
           let data = params["data"],
           //let handlerId = params["handlerId"],
           let env = params["env"]?.jsonStringToDict(),
           let pwd = env["PWD"] as? String {
            
            let relative = pwd
            let url: URL = {
                let filepath = NSString(string: path).standardizingPath
                if (filepath.starts(with: "/")) {
                    return URL(fileURLWithPath: filepath)
                } else {
                  return URL(fileURLWithPath: filepath, relativeTo: URL(fileURLWithPath: relative))
                }
            }()

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
           let handlerId = params["handlerId"]
//           let env = params["env"]?.jsonStringToDict(),
//           let pwd = env["PWD"] as? String
            {
            let relative: String? = params["env"]?.jsonStringToDict()?["PWD"]  as? String
            let url: URL = {
                let filepath = NSString(string: path).standardizingPath
                if (filepath.starts(with: "/")) {
                    return URL(fileURLWithPath: filepath)
                } else {
                    return URL(fileURLWithPath: filepath, relativeTo: URL(fileURLWithPath: relative ?? ""))
                }
            }()
                
//            let url = URL(fileURLWithPath: NSString(string: path).standardizingPath, relativeTo: URL(fileURLWithPath: relative ?? ""))
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
        WindowServer.shared.takeFocus()
    }
    
    static func blur(scope: WKScriptMessage) {
        WindowServer.shared.returnFocus()
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
                        Logger.log(message: "Could not write file '\(path)' to disk.", subsystem: .javascript)
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
            //companion.repositionWindow(forceUpdate: true, explicit: true)
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

            }, completion: { (out) in
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
            Logger.log(message: "Couldn't stream \(scope.body)", subsystem: .javascript)
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
                    scope.webView?.window?.level = .normal
                    Accessibility.promptForPermission { (granted) in
                      DispatchQueue.main.async {
                          scope.webView?.window?.level = .floating
                          scope.webView?.evaluateJavaScript("fig.callback('\(handlerId)', '')", completionHandler: nil)
                      }
                    }

                case "ssh":
                    SSHIntegration.install()
                case "ws":
                    ShellBridge.shared.startWebSocketServer()
                case "close":
//                    WindowManager.shared.bringTerminalWindowToFront()
                    //WindowManager.shared.newNativeTerminalSession()
                    scope.webView?.window?.close()
                    Defaults.loggedIn = true

                    Onboarding.setupTerminalsForShellOnboarding {
                      SecureKeyboardInput.notifyIfEnabled()
                    }
                
                    if let delegate = NSApplication.shared.delegate as? AppDelegate {
                        delegate.setupCompanionWindow()
                    }
                
//                    NSWorkspace.shared.launchApplication("Terminal")
                case "forceUpdate":
                    if let appDelegate = NSApp.delegate as? AppDelegate {
                        appDelegate.updater.installUpdatesIfAvailable()
                    }
                case "promptUpdate":
                    if let appDelegate = NSApp.delegate as? AppDelegate {
                        appDelegate.updater.checkForUpdates(self)
                    }
                case "hello":
                    Timer.delayWithSeconds(2) {
                        NSApp.deactivate()
                        ShellBridge.injectStringIntoTerminal("bash ~/.fig/hello.sh", runImmediately: true)
                }
                case "deleteCache":
                    (scope.webView as? WebView)?.deleteCache()
                case "newTerminalWindow":
                    let path = Bundle.main.path(forResource: "open_new_terminal_window", ofType: "scpt")
                    NSAppleScript.run(path: path!)
                case "terminaltitle:true":
                    AutocompleteContextNotifier.addIndicatorToTitlebar = true
                case "terminaltitle:false":
                    AutocompleteContextNotifier.addIndicatorToTitlebar = false

                case "openOnStartup:true":
                    LoginItems.shared.currentApplicationShouldLaunchOnStartup = true
                case "openOnStartup:false":
                    LoginItems.shared.currentApplicationShouldLaunchOnStartup = false
                case "themes":
                    let files = (try? FileManager.default.contentsOfDirectory(atPath: NSHomeDirectory() + "/.fig/themes")) ?? []
                    let themes = [ "dark", "light"] + files.map { String($0.split(separator: ".")[0]) }.sorted()
                    WebBridge.callback(handler: handlerId,
                                       value: themes.joined(separator: "\n"),
                                       webView: scope.webView)
            default:
                break;
            }
           
        }
    }
    
    static func callback(handler: String, value: String, webView: WKWebView?) {
        let encoded = value.data(using: .utf8)!
        webView?.evaluateJavaScript("fig.callback(`\(handler)`,`\(encoded.base64EncodedString())`)", completionHandler: nil)
    }
    
    static func defaults(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
            let key = params["key"] {
            
            if let handlerId = params["handlerId"] {
                let value = UserDefaults.standard.string(forKey: key)
                WebBridge.callback(handler: handlerId, value: value ?? "", webView: scope.webView)
                
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
            
            WebBridge.callback(handler: handlerId, value: NSString(string: path).standardizingPath, webView: scope.webView)
//            NSURL(string: path)?.pathComponents
            
        }
    }
    
      static func notification(scope: WKScriptMessage) {
            if let params = scope.body as? Dictionary<String, String>,
               let title = params["title"],
               let text = params["text"] {
                
               let notification = NSUserNotification()
               notification.title = title
               notification.subtitle = text
               notification.soundName = NSUserNotificationDefaultSoundName
               NSUserNotificationCenter.default.deliver(notification)
            }
        }
    
    static func detach(scope: WKScriptMessage) {
        if let webView = scope.webView,
           let window = webView.window as? CompanionWindow {
            if (window.isDocked) {
                window.untether()
            } else {
                print("Cannot untether an undocked window")
            }
        }
    }
    
    static func stdout(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
            let out = params["out"],
            let companion = scope.getCompanionWindow(),
            let sessionId = companion.sessionId {
            ShellBridge.shared.socketServer.send(sessionId: sessionId, command: out)
        }
    }

    
    static func privateAPI(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, Any>,
            let type = params["type"] as? String,
            let data = params["data"] as? Dictionary<String, String>,
            let handlerId = params["handlerId"] as? String? {
                
            switch(type) {
                case "track":
                    var dict = data
                    guard let event = dict["name"] else {
                        return
                    }
                    dict.removeValue(forKey: "name")

                    TelemetryProvider.track(event: event, with: dict, needsPrefix: nil)
                case "identify":
                    TelemetryProvider.identify(with: data, needsPrefix: nil)
                case "alias":
                    guard let newId = data["userId"] else {
                        return
                    }
                    
                    TelemetryProvider.alias(userId: newId)
                case "cwd":
                    if let window = scope.getCompanionWindow()?.tetheredWindow,
                        let tty = ShellHookManager.shared.tty(for: window.hash) {
                        let running = tty.running
                        print("tty: \(running?.cmd ?? "?") \(running?.cwd ?? "cwd")")
                    }
                case "keystroke":
                    guard let keyCodeString = data["key"], let keyCode = UInt16(keyCodeString), let consumerString = data["consumer"] else {
                        print("Missing params for keystroke")
                        return
                    }
                    
                    if (consumerString == "true") {
                        KeypressProvider.shared.addRedirect(for: keyCode, in: (scope.getCompanionWindow()?.tetheredWindow)!)
                    } else {
                        KeypressProvider.shared.removeRedirect(for: keyCode, in: (scope.getCompanionWindow()?.tetheredWindow)!)

                    }
                case "autocomplete-hide":
//                  break;
                    guard let companion = scope.getCompanionWindow(), companion.isAutocompletePopup else { return }//, let window = AXWindowServer.shared.whitelistedWindow  else { return }
                    //KeypressProvider.shared.keyBuffer(for: window.hash).writeOnly = true
                    Autocomplete.position {
                      // Cause up arrow to immediately start scrolling through history
                      ShellBridge.simulate(keypress: .upArrow)
                    }
                    
                case "setAutocompleteHeight":
                    print("positioning: attempting to setAutocompleteHeight")

                    guard let heightString = data["height"] else { return }
                    let companion = scope.getCompanionWindow()
                    let previousMax = companion?.maxHeight
                    if let number = NumberFormatter().number(from: heightString) {
                       companion?.maxHeight = CGFloat(truncating: number)
                    } else {
                       companion?.maxHeight = nil
                    }
                    print("flicker: set maxHeight = \(heightString)")
                    guard previousMax != companion?.maxHeight else {
                        print("flicker: heights matched")
                        
                        // still may need to make window visible
                        if companion?.maxHeight != 0 {
                            companion?.orderFrontRegardless()
                        }

                        if let handlerId = handlerId {
                            WebBridge.callback(handler: handlerId, value: "", webView: scope.webView)
                        }

                        return
                    }
                    // testing
                    if(!(companion?.isAutocompletePopup ?? false)) {
                        companion?.windowManager.requestWindowUpdate()
                    } else {
                        if companion?.maxHeight == 0 {
                            companion?.orderOut(self)
                        } else {
                            if (previousMax == 0 || previousMax == nil) {
                                NotificationCenter.default.post(name: NSNotification.Name("showAutocompletePopup"), object: nil)
                            }
                            companion?.orderFrontRegardless()
                            let rect = Accessibility.getTextRect()
                            WindowManager.shared.positionAutocompletePopover(textRect: rect)
                        }
                        
                        if let handlerId = handlerId {
                            WebBridge.callback(handler: handlerId, value: "", webView: scope.webView)
                        }
  
                    }
                case "prompt":
                    let source = data["source"]

                    Feedback.getFeedback(source: source ?? "javascript")
                case "alert":
                    let title = data["title"] ?? "title"
                    let message = data["message"] ?? "message"
                    let yesButtonText = data["yesButtonText"] ?? "OK"
                    let noButtonText = data["noButtonText"]


                    let response = Alert.show(title: title,
                                              message: message,
                                              okText: yesButtonText,
                                              icon: Alert.appIcon,
                                              hasSecondaryOption: noButtonText != nil && noButtonText != "",
                                              secondaryOptionTitle: noButtonText)
                    
                    if let handlerId = handlerId {
                        WebBridge.callback(handler: handlerId,
                                           value: response ? "true" : "false",
                                           webView: scope.webView)
                    }
                case "positioning.isValidFrame":
                    guard let width = Float(data["width"] ?? ""),
                      let height = Float(data["height"]  ?? ""),
                      let anchorX = Float(data["anchorX"]  ?? ""),
                      let anchorY = Float(data["offsetFromBaseline"]  ?? "0"),
                      let handler = handlerId else {
                      return
                    }
                    
                    do {
                        let response = try WindowPositioning.frameRelativeToCursor(width: CGFloat(width),
                                                                               height: CGFloat(height),
                                                                               anchorOffset: CGPoint(x: CGFloat(anchorX), y: CGFloat(anchorY)))
                        WebBridge.callback(handler: handler, value: "{ \"isAbove\":  \(response.isAbove ? "true" : "false"), \"isClipped\": \(response.isClipped ? "true" : "false") }", webView: scope.webView)
                        
                    } catch APIError.generic(message: let message) {
                        WebBridge.callback(handler: handler,
                                           value: "{ \"error\" : \"\(message)\" }",
                                           webView: scope.webView)
                    } catch {}
                    
  
                    
                case "positioning.setFrame":
                    guard let companion = scope.getCompanionWindow() else {
                        return
                    }
                    
                    if let width = Float(data["width"] ?? "") {
                        print("autocomplete.width := \(width)")
                        companion.width = CGFloat(width)
                    }
                    
                    if let height = Float(data["height"]  ?? "") {
                        print("autocomplete.height := \(height)")
                        companion.maxHeight = CGFloat(height)
                    }
                    
                    if let anchorX = Float(data["anchorX"]  ?? "") {
                        print("autocomplete.anchorX := \(anchorX)")
                        var anchor = companion.anchorOffsetPoint
                        anchor.x = CGFloat(anchorX)
                        companion.anchorOffsetPoint = anchor                    }
                    
                    if let anchorY = Float(data["offsetFromBaseline"]  ?? "0") {
                        print("autocomplete.anchorY := \(anchorY)")
                        var anchor = companion.anchorOffsetPoint
                        anchor.y = CGFloat(anchorY)
                        companion.anchorOffsetPoint = anchor
                    }
                    
                    WindowManager.shared.positionAutocompletePopover(textRect: Accessibility.getTextRect())

                    if let handlerId = handlerId {
                        WebBridge.callback(handler: handlerId, value: "", webView: scope.webView)
                    }
                    
                case "key":
                  guard let codeString = data["code"], let keycode = UInt16(codeString), let keypress = ShellBridge.Keypress(rawValue: keycode) else {
                      return
                  }
              
                  ShellBridge.simulate(keypress: keypress)
                case "settings":
                  guard let key = data["key"],
                    let valueString = data["value"],
                    let valueData = valueString.data(using: .utf8),
                    let value = try? JSONSerialization.jsonObject(with: valueData, options: .allowFragments) else {
                    return
                  }
                  
                  Settings.shared.set(value: value, forKey: key)
                case "status":

                  let companion = scope.getCompanionWindow()              
                  companion?.status = (data["message"] ?? "", data["color"], data["display"] == "true")
                case "uninstall":
                  NSApp.appDelegate.uninstall()
                default:
                    print("private command '\(type)' does not exist.")
            }
        }
    }
    
    static func propertyValueChanged(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let webView = scope.webView,
           let window = webView.window,
           let prop = params["prop"],
           let value = params["value"] {

            switch prop {
            case "title":
                window.title = value.truncate(length: 15)
            case "color":
                window.backgroundColor = NSColor(hex: value) ?? .white
            case "icon":
                if let url = URL(string: value, relativeTo: webView.url) {
                    window.representedURL = url;

                    if (url.scheme == "fig") {
                        guard let fileicon = WebBridge.fileIcon(for: url) else { return }
                        window.standardWindowButton(.documentIconButton)?.image = fileicon

                        return
                    }

                 DispatchQueue.global().async {
                     guard let data = try? Data(contentsOf: url)  else { return }//make sure your image in this url does exist, otherwise unwrap in a if let check / try-catch
                        DispatchQueue.main.async {
                        window.standardWindowButton(.documentIconButton)?.image = NSImage(data: data)
                        }
                    }
               }
            case "loaded":
              let companion = scope.getCompanionWindow()
              companion?.loaded = value == "true"
            case "maxheight":
                print("positioning: attempting to set maxheight")
                let companion = scope.getCompanionWindow()
                let previousMax = companion?.maxHeight
                if let number = NumberFormatter().number(from: value) {
                   companion?.maxHeight = CGFloat(truncating: number)
                } else {
                   companion?.maxHeight = nil
                }
                
                guard previousMax != companion?.maxHeight else {
                    print("flicker: heights matched")
                    return
                }
                
                // testing
                if(!(companion?.isAutocompletePopup ?? false)) {
                    companion?.windowManager.requestWindowUpdate()
                } else {
                    if companion?.maxHeight == 0 {
                        companion?.orderOut(self)
                    } else {
                        if (previousMax == 0 || previousMax == nil) {
                            NotificationCenter.default.post(name: NSNotification.Name("showAutocompletePopup"), object: nil)
                        }
                        companion?.orderFrontRegardless()
                        //let rect = KeypressProvider.shared.getTextRect()
                        //WindowManager.shared.positionAutocompletePopover(textRect: rect)
                        Autocomplete.position()
                    }
//                    let rect = KeypressProvider.shared.getTextRect()
//                    WindowManager.shared.positionAutocompletePopover(textRect: rect)
                }
            case "width":
                
                guard let companion = scope.getCompanionWindow(), companion.isAutocompletePopup else {
                    return
                }
                let previousWidth = companion.width
                if let number = NumberFormatter().number(from: value) {
                    companion.width = CGFloat(truncating: number)
                } else {
                    companion.width = nil
                }
                
                guard previousWidth != companion.width else {
                    print("flicker: widths matched")
                    return
                }
                
                //let rect = KeypressProvider.shared.getTextRect()
                //WindowManager.shared.positionAutocompletePopover(textRect: rect)
                Autocomplete.position()


            case "interceptKeystrokes":
                KeypressProvider.shared.setEnabled(value: value == "true")

            default:
                print("Unrecognized property value '\(prop)' updated with value: \(value)")
            }
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
            var last = nodes[nodes.length-3];
            
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
            target.blur()
            """, completionHandler: nil)
    }
    
    static func declareAppVersion(webview: WebView) {
        if let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String {
            webview.evaluateJavaScript("fig.appversion = '\(appVersion)'", completionHandler: nil)
        }
    }
    
    static func declareFigCLIPath(webview: WebView) {
        if let cliPath = Bundle.main.path(forAuxiliaryExecutable: "figcli") {
            webview.evaluateJavaScript("fig.clipath = '\(cliPath)'", completionHandler: nil)
        }
    }
    
    static func declareHomeDirectory(webview: WebView) {
        webview.evaluateJavaScript("fig.home = '\(NSHomeDirectory())'", completionHandler: nil)
    }
  
    static func declareSettings(webview: WebView) {
        guard let settings = Settings.shared.jsonRepresentation(), let b64 = settings.data(using: .utf8)?.base64EncodedString() else { return }
        webview.evaluateJavaScript("fig.updateSettings(b64DecodeUnicode(`\(b64)`))", completionHandler: nil)
      }
    
    
    static func declareRemoteURL(webview: WebView) {
        webview.evaluateJavaScript("fig.remoteURL = '\( Remote.baseURL.absoluteString)'", completionHandler: nil)
    }
    
    static func declareBuildNumber(webview: WebView) {
        webview.evaluateJavaScript("fig.buildNumber = '\( Diagnostic.build)'", completionHandler: nil)
    }
  
    static func declareUpdate(webview: WebView) {
      webview.evaluateJavaScript("fig.updateAvailable = \(UpdateService.provider.updateIsAvailable)", completionHandler: nil)

      if UpdateService.provider.updateIsAvailable {
        webview.evaluateJavaScript(
          """
          fig.updateMetadate =
          {
            version: "\( UpdateService.provider.updateVersion ?? "")",
            build: "\( UpdateService.provider.updateBuild ?? "")",
            published: "\(UpdateService.provider.updatePublishedOn ?? "")"
          }
          """, completionHandler: nil)
      } else {
        webview.evaluateJavaScript(
          """
          fig.updateMetadate = null
          """, completionHandler: nil)
      }
    }
  
  static func declareCurrentApplication(webview: WebView) {
      let bundleId = AXWindowServer.shared.whitelistedWindow?.bundleId
      webview.evaluateJavaScript("fig.currentApp = '\(bundleId ?? "")'", completionHandler: nil)
          
  }
    
    static func initJS(webview: WebView) {
        webview.evaluateJavaScript("fig.callinit()", completionHandler: nil)
    }
    
    static func appinfo(webview: WebView, response: @escaping (Dictionary<String, String?>) -> Void) {
        webview.evaluateJavaScript("fig.appinfo()") { (info, error) in
            
            if let info = info as? Dictionary<String, String?> {
                  
                   response(info)
                   return
            }
        }
    }
    
    static func updateTitlebar(webview: WebView) {
        WebBridge.appinfo(webview: webview) { (info) -> Void in
            if let window = webview.window, let c = window as? CompanionWindow {
                if c.positioning.hasTitleBar {
                    c.setupTitleBar()
                }
            }
            
            webview.window?.title = (info["name"] ?? "Fig") ?? webview.title?.truncate(length: 25) ?? ""
          
          if let hexValue = info["color"], let hex = hexValue {
              webview.window?.backgroundColor = NSColor(hex: hex)
          }

          if let icon = info["icon"], let url = URL(string: icon ?? "", relativeTo: webview.url) {
              webview.window?.representedURL = url;

              DispatchQueue.global().async {
                  guard let data = try? Data(contentsOf: url)  else { return }//make sure your image in this url does exist, otherwise unwrap in a if let check / try-catch
                     DispatchQueue.main.async {
                      webview.window?.standardWindowButton(.documentIconButton)?.image = NSImage(data: data)
                     }
                 }
            }
        }
    }
    
    static func enableInteractiveCodeTags(webview: WebView) {
        let script: WebBridgeScript = WebBridge.authorized(webView: webview) ? .runnableCodeTags : WebBridgeScript.insertableCodeTags
        
        webview.evaluateJavaScript(script.codeForScript(), completionHandler: nil)
    }
    
    static func appInitialPosition(webview: WebView, response: @escaping (String?) -> Void) {
        webview.evaluateJavaScript("try { document.head.querySelector('meta[initial-position]').getAttribute('initial-position') } catch(e) {}") { (name, error) in
            response(name as? String)
            return
        }
    }
    
    
    static func appname(webview: WebView, response: @escaping (String?) -> Void) {
        webview.evaluateJavaScript("try { document.head.querySelector('meta[fig\\:app]').getAttribute('fig:app') } catch(e) {}") { (name, error) in
            response(name as? String)
            return
        }
    }
    
    static func appicon(webview: WebView, response: @escaping (String?) -> Void) {
        webview.evaluateJavaScript("document.head.querySelector('meta[fig\\:icon]').getAttribute('fig:icon')") { (name, error) in
            response(name as? String)
            return
        }
    }

    static func pty(scope: WKScriptMessage) {
        if let params = scope.body as? Dictionary<String, String>,
           let type = params["type"],
           let webview = scope.webView as? WebView,
           let window = webview.window,
           let controller = window.contentViewController as? WebViewController {
            print("\(params) \(webview), \(controller.pty.pty.process.running), \(controller.pty.pty.process.delegate)")
            switch (type) {
                case "init":
                    
                    if let env = params["env"]{
                        var parsedEnv = env.jsonStringToDict() as? [String: String] ?? FigCLI.extract(keys: ["PWD","USER","HOME","SHELL", "OLDPWD", "TERM_PROGRAM", "TERM_SESSION_ID", "HISTFILE","FIG","FIGPATH"], from: env)
                        parsedEnv["HOME"] = NSHomeDirectory()
                        parsedEnv["TERM"] = "xterm"//-256color
//                        parsedEnv["SHELL"] = "/bin/bash"
//                        DispatchQueue.global(qos: .userInteractive).async {
                        controller.pty.start(with: parsedEnv)
//                        }
                    } else {
                        controller.pty.start(with: ["HOME":NSHomeDirectory(), "TERM" : "xterm-256color", "SHELL":"/bin/bash"])
                    }
         
                case "stream":
                    if let cmd = params["cmd"],
                        let handlerId = params["handlerId"] {
                        controller.pty.stream(command: cmd, handlerId: handlerId)
                    }
                case "execute":
                    if let cmd = params["cmd"],
                        let handlerId = params["handlerId"] {
                        
                        var asBackgroundJob: Bool = true
                        var asPipeline: Bool = false

                        if let options = params["options"],
                           let parsedOptions = options.jsonStringToDict() {
                            
                          asBackgroundJob = parsedOptions["backgroundJob"] as? Bool ?? asBackgroundJob
                          asPipeline = parsedOptions["pipelined"] as? Bool ?? asPipeline
                          
                        }
                      controller.pty.execute(command: cmd, handlerId: handlerId, asBackgroundJob: asBackgroundJob, asPipeline: asPipeline)
                    }
                case "shell":
                    if let cmd = params["cmd"],
                        let handlerId = params["handlerId"] {
                        controller.pty.shell(command: cmd, handlerId: handlerId)
                    }
                case "write":
                    if let cmd = params["cmd"] {
                        if let code = ControlCode(rawValue:cmd) {
                            controller.pty.write(command: "", control: code)
                        } else {
                            controller.pty.write(command: cmd, control: nil)
                        }
                    }
                case "exit":
                    controller.pty.close()
                default:
                    break;
            }
           
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
        var dict = [String:String]()

        if let components = URLComponents(url: self, resolvingAgainstBaseURL: false) {
          if let queryItems = components.queryItems {
            for item in queryItems where item.value != nil {
              dict[item.name] = item.value!
            }
          }
          return dict
        } else {
          return [:]
        }
    }
//    var queryDictionary: [String: String]? {
//
//        guard let query = self.query else { return nil}
//
//        var queryStrings = [String: String]()
//        for pair in query.components(separatedBy: "&") {
//
//            let key = pair.components(separatedBy: "=")[0]
//
//            let value = pair
//                .components(separatedBy:"=")[1]
//                .replacingOccurrences(of: "+", with: " ")
//                .removingPercentEncoding ?? ""
//
//            queryStrings[key] = value
//        }
//        return queryStrings
//    }
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
            case .insertableCodeTags:
                return File.contentsOfFile("insert-tutorial", type: "js")!
            case .runnableCodeTags:
                return File.contentsOfFile("run-tutorial", type: "js")!
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

extension WKScriptMessage {
    func getFigWebView() -> WebView? {
        return self.webView as? WebView
    }
    
    func getCompanionWindow() -> CompanionWindow? {
        return self.webView?.window as? CompanionWindow
    }
    
    func getContentController() -> WebViewController? {
        return self.webView?.window?.contentViewController as? WebViewController
    }
}

extension String {
  /*
   Truncates the string to the specified length number of characters and appends an optional trailing string if longer.
   - Parameter length: Desired maximum lengths of a string
   - Parameter trailing: A 'String' that will be appended after the truncation.
    
   - Returns: 'String' object.
  */
  func truncate(length: Int, trailing: String = "…") -> String {
    return (self.count > length) ? self.prefix(length) + trailing : self
  }
}
