//
//  WebViewController.swift
//  fig
//
//  Created by Matt Schrage on 4/17/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
import WebKit
import AppKit

class WebViewController: NSViewController, NSWindowDelegate {
    var mouseLocation: NSPoint? { self.view.window?.mouseLocationOutsideOfEventStream }

    var webView: WebView? // = WKWebView(frame:.zero)
    
    var icon: NSTextField = {
        let label = NSTextField()
        label.frame = CGRect(origin: .zero, size: CGSize(width: 50, height: 50))
        label.stringValue = "🍐"
        label.alignment = .center
        label.font = NSFont(name: "AppleColorEmoji", size: 30)
        label.backgroundColor = .white
        label.isBezeled = false
        label.isEditable = false
        label.sizeToFit()
//        label.isHidden = true

        let gesture = NSClickGestureRecognizer()
        gesture.buttonMask = 0x1 // left mouse
        gesture.numberOfClicksRequired = 1
        gesture.target = NSApp.delegate
        gesture.action = #selector(AppDelegate.toggleVisibility)

        label.addGestureRecognizer(gesture)
        
//        label.layer?.shadowColor = NSColor.black.cgColor
//        label.layer?.shadowRadius = 3.0
//        label.layer?.shadowOpacity = 1.0
//        label.layer?.shadowOffset = CGSize(width: 4, height: 4)
//        label.layer?.masksToBounds = false
        
        return label
    }()
    
    init(){
        super.init(nibName: nil, bundle: nil)
        webView = WebView(frame: .zero, configuration: WebBridge())
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    //    override func loadView() {
//        self.view = webView
//    }
    override func loadView() {
//        self.view = NSView(frame: .zero)
////        let blurView = NSView(frame: view.bounds)
//         view.wantsLayer = true
//         view.layer?.backgroundColor = NSColor.clear.cgColor
//         view.layer?.masksToBounds = true
//         view.layerUsesCoreImageFilters = true
//         view.layer?.needsDisplayOnBoundsChange = true
//
//
//
//
//         if let blurFilter = CIFilter(name: "CIGaussianBlur"),
//         let satFilter = CIFilter(name: "CIColorControls"){
//
//             satFilter.setDefaults()
//             satFilter.setValue(2, forKey: "inputSaturation")
//
//             blurFilter.setDefaults()
//             blurFilter.setValue(2, forKey: "inputRadius")
//
//             view.layer?.backgroundFilters = [satFilter, blurFilter]
//         }
//
//         view.layer?.needsDisplay()
        
        let effect = NSVisualEffectView(frame: .zero)
        effect.blendingMode = .behindWindow
        effect.state = .active
        effect.material = .mediumLight
        effect.maskImage = _maskImage(cornerRadius: 15)
        self.view = effect;
        self.view.postsFrameChangedNotifications = true
        self.view.postsBoundsChangedNotifications = true
        

        

    }
    override func viewDidAppear() {

//        blur(view:self.view)

        

//        webView?.autoresizingMask = self.view.autoresizingMask
//        webView?.autoresizingMask = NSView.AutoresizingMask(rawValue: NSView.AutoresizingMask.width.rawValue | NSView.AutoresizingMask.height.rawValue);

//        webView?.setValue(false, forKey: "drawsBackground")
        
        // add alpha when using NSVisualEffectView
        //ADD ALPHA TO WINDOW
        //self.view.window?.alphaValue = 0.9
        
//        self.view.wantsLayer = true
//        self.view.layer?.cornerRadius = 15
//        self.view.layer?.masksToBounds = true
//        self.webView.alphaValue = 0.75
//        self.view.alphaValue = 0.5;
        
        print("ViewDidAppear -- \( webView?.url?.absoluteString ?? "no url")")
        if !((webView?.url) != nil) {
            webView?.loadSideBar()
        }
    }


    override func viewDidLayout() {
        super.viewDidLayout()
//        self.webView!.frame = self.view.frame
//        self.webView!.setNeedsDisplay(self.webView!.frame)
        print("viewDidLayout")
//        self.webView?.needsLayout
//        self.webView?.frame = self.view.frame

    }


    func windowDidResize(_ notification: Notification) {
        // This will print the window's size each time it is resized.
//        self.view.frame = self.view.window?.frame ?? .zero
//        print(view.window?.frame.size ?? "<none>", self.webView!.frame.size)
////        self.webView!.frame = self.view.frame
//        print(view.window?.frame.size ?? "<none>", self.webView!.frame.size)
//        print(view.frame.size, self.webView!.frame.size)
//        if let window = self.view.window {
//            self.view.frame = window.frame
//        }
        
        print(view.window?.frame ?? .zero, view.frame, self.webView?.frame ?? .zero)

//        self.webView?.reload()
        print("resize")
    }
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        print("viewDidLoad")
        
        if (self.webView == nil) {
            webView = WebView(frame: .zero, configuration: WebBridge())
        }

        
        NotificationCenter.default.addObserver(self, selector: #selector(recievedDataFromPipe(_:)), name: .recievedDataFromPipe, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(recievedStdoutFromTerminal(_:)), name: .recievedStdoutFromTerminal, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(recievedUserInputFromTerminal(_:)), name: .recievedUserInputFromTerminal, object: nil)
        
        NotificationCenter.default.addObserver(self, selector: #selector(insertCommandInTerminal(_:)), name: .insertCommandInTerminal, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(executeCommandInTerminal(_:)), name: .executeCommandInTerminal, object: nil)
        
        NotificationCenter.default.addObserver(self, selector: #selector(windowDidResize(_:)), name: NSWindow.didResizeNotification, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(viewFrameResized), name:NSView.frameDidChangeNotification, object: self.view)
        NotificationCenter.default.addObserver(self, selector: #selector(viewFrameResized), name:NSView.boundsDidChangeNotification, object: self.view)
        NotificationCenter.default.addObserver(self, selector: #selector(overlayDidBecomeIcon), name:.overlayDidBecomeIcon, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(overlayDidBecomeMain), name:.overlayDidBecomeMain, object: nil)
        // TIMER to UPDATE INNER VIEW EVERY x INTERVAL
        
        
        view.window?.delegate = self
//        webView = WKWebView(frame: self.view.window?.frame ?? .zero)
//        webView.ba
        webView?.translatesAutoresizingMaskIntoConstraints = false

        webView?.navigationDelegate = self
        self.view.addSubview(webView!)
        NSLayoutConstraint.activate([
            webView!.topAnchor.constraint(equalTo: view.topAnchor),
            webView!.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            webView!.leftAnchor.constraint(equalTo: view.leftAnchor),
            webView!.rightAnchor.constraint(equalTo: view.rightAnchor)
        ])
//
        self.view.addSubview(self.icon)
        self.icon.isHidden = true;

//        webView?.bindFrameToSuperviewBounds()
        
    }
    
    @objc func overlayDidBecomeIcon() {
        print("didBecomeIcon")
//        self.icon.isHidden = false;
//
//        self.icon.frame = NSRect(x: 0, y: -6, width: 50, height: 50)
//        self.webView?.loadBundleApp("sidebar")
        self.webView?.loadSideBar()
        (self.view as! NSVisualEffectView).maskImage = _maskImage(cornerRadius: 5)
    }
    
    @objc func overlayDidBecomeMain() {
        print("didBecomeMain")
//        self.icon.isHidden = true
//        self.webView?.loadHomeScreen()
        (self.view as! NSVisualEffectView).maskImage = _maskImage(cornerRadius: 15)

    }
    
    @objc func viewFrameResized() {
//        print("viewResized")
        self.webView?.frame = self.view.bounds
    }
    
    func loadHTMLString(_ html: String) {
        webView?.loadHTMLString(html, baseURL: nil)
    }
    

    
    //https://stackoverflow.com/a/29801359
    private func blur(view: NSView!) {
        let blurView = NSView(frame: view.bounds)
        blurView.wantsLayer = true
        blurView.layer?.backgroundColor = NSColor.clear.cgColor
        blurView.layer?.masksToBounds = true
        blurView.layerUsesCoreImageFilters = true
        blurView.layer?.needsDisplayOnBoundsChange = true

       
       

        if let blurFilter = CIFilter(name: "CIGaussianBlur"),
        let satFilter = CIFilter(name: "CIColorControls"){
            
            satFilter.setDefaults()
            satFilter.setValue(2, forKey: "inputSaturation")
            
            blurFilter.setDefaults()
            blurFilter.setValue(2, forKey: "inputRadius")

            blurView.layer?.backgroundFilters = [satFilter, blurFilter]
        }
        view.addSubview(blurView)

        blurView.layer?.needsDisplay()
    }
    
    private func _maskImage(cornerRadius: CGFloat) -> NSImage {
        let edgeLength = 2.0 * cornerRadius + 1.0
        let maskImage = NSImage(size: NSSize(width: edgeLength, height: edgeLength), flipped: false) { rect in
            let bezierPath = NSBezierPath(roundedRect: rect, xRadius: cornerRadius, yRadius: cornerRadius)
            NSColor.black.set()
            bezierPath.fill()
            return true
        }
        maskImage.capInsets = NSEdgeInsets(top: cornerRadius, left: cornerRadius, bottom: cornerRadius, right: cornerRadius)
        maskImage.resizingMode = .stretch
        return maskImage
    }
}
//https://ribachenko.com/posts/nsvisualeffectview-with-adjustable-blur-level/
class SemiTransparentView: NSView {

    var alphaLevel: Double = 0.12

    override var allowsVibrancy: Bool { return true }

    override func draw(_ dirtyRect: NSRect) {
        NSColor(deviceWhite: 255, alpha: CGFloat(alphaLevel)).set()
        dirtyRect.fill()
//        NSRectFill(dirtyRect)
//        NSRect.fil
    }

}

extension WebViewController: WebBridgeEventListener {
    
    
    @objc func insertCommandInTerminal(_ notification: Notification) {
        ShellBridge.injectStringIntoTerminal(notification.object as! String, runImmediately: false, completion: {
            if let currentMouseLocation = self.mouseLocation {
               print("mouseLocation:", currentMouseLocation)
               print("mouseInWindow", self.view.bounds.contains(currentMouseLocation))
               if (self.view.bounds.contains(currentMouseLocation)) {
                   NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
               }
           }
        })
    }
    
    @objc func executeCommandInTerminal(_ notification: Notification) {
        ShellBridge.injectStringIntoTerminal(notification.object as! String, runImmediately: true, completion: {
            if let currentMouseLocation = self.mouseLocation {
               print("mouseLocation:", currentMouseLocation)
               print("mouseInWindow", self.view.bounds.contains(currentMouseLocation))
//               if (self.view.bounds.contains(currentMouseLocation)) {
//                   NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
//               }
           }
        })
    }
    
    
}

extension WebViewController: ShellBridgeEventListener {
    @objc func recievedUserInputFromTerminal(_ notification: Notification) {
        // match against regex?
        WebBridge.ttyin(webView: self.webView!, msg: notification.object as! ShellMessage)
    }
    
    @objc func recievedStdoutFromTerminal(_ notification: Notification) {
        // match against regex?
        if let msg = notification.object as? ShellMessage {
            if (ErrorMatcher.shouldMatchOn(data: msg.data)) {
                ErrorMatcher.matchOn(error: msg.data) { (data) in
                    DispatchQueue.main.async {
                        let companion = self.view.window as! CompanionWindow
                        companion.positioning = .notification
                        self.webView?.loadBundleApp("error")
                        self.webView?.onLoad.append {
                            self.webView?.evaluateJavaScript("fig.init(`\(data.base64EncodedString())`)", completionHandler: nil)
                        }
                    }
                }
            }
        }
        WebBridge.ttyout(webView: self.webView!, msg: notification.object as! ShellMessage)

    }
    
    
    @objc func recievedDataFromPipe(_ notification: Notification) {
        //Bring FIG to front when triggered explictly from commandline
        NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
        
        let msg = (notification.object as! ShellMessage)
        DispatchQueue.main.async {
            if let companion = self.view.window as? CompanionWindow {
            FigCLI.route(msg,
                         webView: self.webView!,
                         companionWindow: companion)
            }
        }
    }
}


extension WebViewController : WKNavigationDelegate {
    func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
         let webView = webView as! WebView

           for onNavigateCallback in webView.onNavigate {
               onNavigateCallback()
           }
           webView.onNavigate = []
    }
    
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        
        decisionHandler(.allow)
        
        let webView = webView as! WebView

        for onNavigateCallback in webView.onNavigate {
            onNavigateCallback()
        }
        webView.onNavigate = []
    }
    
    func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
        print("ERROR Loading URL: \(error.localizedDescription)")
    }
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        print("Loaded URL \(webView.url?.absoluteString ?? "<none>")")
        var scriptContent = "var meta = document.createElement('meta');"
        scriptContent += "meta.name='viewport';"
        scriptContent += "meta.content='width=device-width';"
        scriptContent += "document.getElementsByTagName('head')[0].appendChild(meta);"

        webView.evaluateJavaScript(scriptContent, completionHandler: nil)
        
        
        let webView = webView as! WebView
        
        if let configureEnv = webView.configureEnvOnLoad {
            configureEnv()
        }
        
        for onLoadCallback in webView.onLoad {
            onLoadCallback()
        }
        webView.onLoad = []
        WebBridge.declareAppVersion(webview: webView)
        WebBridge.initJS(webview: webView)
//        webView.evaluateJavaScript("fig.callinit()", completionHandler: nil)


        
//    webView.evaluateJavaScript("window.scrollTo(0,0)", completionHandler: nil)

        
//        self.webView?.evaluateJavaScript("document.body.style = document.body.style.cssText + \";background: transparent !important;\";", completionHandler: nil)
//        
        
//        self.webView?.evaluateJavaScript("document.readyState", completionHandler: { (complete, error) in
//            if complete != nil {
//                self.webView?.evaluateJavaScript("document.body.scrollHeight", completionHandler: { (height, error) in
//                    let h = height as! CGFloat
//                    print(h)
//                })
//                
//            }
//
//            })
    }
}

class WebView : WKWebView {
    var trackingArea : NSTrackingArea?
    var trackMouse = true
    var onLoad: [(() -> Void)] = []
    var onNavigate: [(() -> Void)] = []
    var configureEnvOnLoad: (() -> Void)?
    private var dragShouldRepositionWindow = false
    
//    override var intrinsicContentSize: NSSize {
//        get {
//            return self.superview?.bounds.size ?? NSSize.zero
//        }
//    }

    override func shouldDelayWindowOrdering(for event: NSEvent) -> Bool {
        return true
    }

    override init(frame: CGRect, configuration: WKWebViewConfiguration) {
        super.init(frame: frame, configuration: configuration)
        //self.customUserAgent = "Mozilla/5.0 (iPhone; CPU iPhone OS 13_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/83.0.4103.88 Mobile/15E148 Safari/604.1 FigBrowser/\(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0")"
        NotificationCenter.default.addObserver(self, selector: #selector(requestStopMonitoringMouseEvents(_:)), name: .requestStopMonitoringMouseEvents, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(requestStartMonitoringMouseEvents(_:)), name: .requestStartMonitoringMouseEvents, object: nil)
//        self.unregisterDraggedTypes()
//        self.autoresizingMask = NSView.AutoresizingMask.init(arrayLiteral: [.height, .width])
//        NSEvent.addLocalMonitorForEvents(matching: NSEvent.EventTypeMask.mouseEntered) { event -> NSEvent? in
//            print(event)
//            return event
//        }
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
   
    
    override func updateTrackingAreas() {
        if trackingArea != nil {
            self.removeTrackingArea(trackingArea!)
        }
        self.trackingArea = NSTrackingArea(rect: self.bounds,
                                       options: [.activeAlways, .mouseEnteredAndExited],
                                       owner: self,
                                       userInfo: nil)
        self.addTrackingArea(trackingArea!)

    }
    override func mouseDown(with event: NSEvent) {
       NSApp.preventWindowOrdering()
       super.mouseDown(with: event)
        
        let loc = event.locationInWindow;
        let height = self.window!.frame.height;
        if (loc.y > height - 28) {
            self.dragShouldRepositionWindow = true;
        }
    }
    
    override func mouseUp(with event: NSEvent) {
        super.mouseUp(with: event)
        dragShouldRepositionWindow = false
    }
    
    override func mouseDragged(with event: NSEvent) {
        super.mouseDragged(with: event)
        
        if (self.dragShouldRepositionWindow) {
            self.window?.performDrag(with: event)
        }
    }
    
    override func mouseEntered(with event: NSEvent) {
        print("mouse entered")
        guard let w = self.window, let window = w as? CompanionWindow else {
            return
        }
        if (trackMouse && window.positioning == CompanionWindow.defaultPassivePosition) {
            print("Attempting to activate fig")
            NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
        }
    }
    
    override func mouseExited(with event: NSEvent) {
        print("mouse exited")
        guard let w = self.window, let window = w as? CompanionWindow else {
                  return
            }
        if (trackMouse && NSWorkspace.shared.frontmostApplication?.isFig ?? false && window.positioning == CompanionWindow.defaultPassivePosition) {
            print("current frontmost application \(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "")")
            print("Attempting to activate previous app \( ShellBridge.shared.previousFrontmostApplication?.bundleIdentifier ?? "<none>")")
            ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)

        }
    }
    
    func loadBundleApp(_ app: String) {

            if let url = Bundle.main.url(forResource: app, withExtension: "html") {
                self.loadFileURL(url, allowingReadAccessTo: URL(string: "file://")!) // needed in order to load local files from anywhere
            } else {
                print("Bundle app '\(app)' does not exist")
            }
    }
    
    func loadLocalApp(_ url: URL) {
//        let localURL = URL(fileURLWithPath: appPath)
        self.evaluateJavaScript("document.documentElement.remove()") { (_, _) in

            self.loadFileURL(url, allowingReadAccessTo: URL(string: "file://")!) // needed in order to load local files from anywhere
        }
    }
    
    func loadRemoteApp(at url: URL) {
        print(url.absoluteString)
//        self.load(URLRequest(url: URL(string:"about:blank")!))
        self.evaluateJavaScript("document.documentElement.remove()") { (_, _) in
            self.load(URLRequest(url: url, cachePolicy: .useProtocolCachePolicy))
        }
    }
    
    func loadHomeScreen() {
        self.evaluateJavaScript("document.documentElement.remove()") { (_, _) in

            self.load(URLRequest(url: URL(string: "https://app.withfig.com")!, cachePolicy: .useProtocolCachePolicy))
        }

    }
    
    func loadSideBar() {
        self.evaluateJavaScript("document.documentElement.remove()") { (_, _) in
           self.load(URLRequest(url: URL(string: "https://app.withfig.com/sidebar")!, cachePolicy: .useProtocolCachePolicy))
       }
    }

    
//    override func scrollWheel(with event: NSEvent) {
//        self.nextResponder?.scrollWheel(with: event)
//    }
    
    
    

    
//    override func hitTest(_ point: NSPoint) -> NSView? {
//        if super.hitTest(point) {
//            return self
//        }
//
//        return nil
//
//    }
    
    
}

extension WebView : MouseMonitoring {
    @objc func requestStopMonitoringMouseEvents(_ notification: Notification) {
        self.trackMouse = false;
    }
    
    @objc func requestStartMonitoringMouseEvents(_ notification: Notification) {
        self.trackMouse = true;

    }
    
    
}


extension NSView {
    /// Adds constraints to this `UIView` instances `superview` object to make sure this always has the same size as the superview.
    /// Please note that this has no effect if its `superview` is `nil` – add this `UIView` instance as a subview before calling this.
    func bindFrameToSuperviewBounds() {
        guard let superview = self.superview else {
            print("Error! `superview` was nil – call `addSubview(view: UIView)` before calling `bindFrameToSuperviewBounds()` to fix this.")
            return
        }

        self.translatesAutoresizingMaskIntoConstraints = false
        self.topAnchor.constraint(equalTo: superview.topAnchor, constant: 0).isActive = true
        self.bottomAnchor.constraint(equalTo: superview.bottomAnchor, constant: 0).isActive = true
        self.leadingAnchor.constraint(equalTo: superview.leadingAnchor, constant: 0).isActive = true
        self.trailingAnchor.constraint(equalTo: superview.trailingAnchor, constant: 0).isActive = true

    }
}
