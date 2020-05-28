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
        

        

    }
    override func viewDidAppear() {

//        blur(view:self.view)

        

//        webView?.autoresizingMask = self.view.autoresizingMask
//        webView?.autoresizingMask = NSView.AutoresizingMask(rawValue: NSView.AutoresizingMask.width.rawValue | NSView.AutoresizingMask.height.rawValue);

//        webView?.setValue(false, forKey: "drawsBackground")
        
        // add alpha when using NSVisualEffectView
        self.view.window?.alphaValue = 0.9
        
//        self.view.wantsLayer = true
//        self.view.layer?.cornerRadius = 15
//        self.view.layer?.masksToBounds = true
//        self.webView.alphaValue = 0.75
//        self.view.alphaValue = 0.5;
        
        print("ViewDidAppear -- \( webView?.url?.absoluteString ?? "no url")")
        if !((webView?.url) != nil) {
            webView?.loadHomeScreen()
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
        
        print(view.window?.frame ?? .zero, view.frame, self.webView?.frame ?? .zero)

//        self.webView?.reload()
        print("resize")
    }
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        print("viewDidLoad")
        
        
        webView = WebView(frame: .zero, configuration: WebBridge())
        
        NotificationCenter.default.addObserver(self, selector: #selector(recievedDataFromPipe(_:)), name: .recievedDataFromPipe, object: nil)
        
        NotificationCenter.default.addObserver(self, selector: #selector(insertCommandInTerminal(_:)), name: .insertCommandInTerminal, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(executeCommandInTerminal(_:)), name: .executeCommandInTerminal, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(windowDidResize(_:)), name: NSWindow.didResizeNotification, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(viewFrameResized), name:NSView.frameDidChangeNotification, object: self.view)
        
        
        view.window?.delegate = self
//        webView = WKWebView(frame: self.view.window?.frame ?? .zero)
//        webView.ba
        webView?.navigationDelegate = self
        self.view.addSubview(webView!)
//        webView?.bindFrameToSuperviewBounds()
        
    }
    
    @objc func viewFrameResized() {
        print("viewResized")
        self.webView?.frame = self.view.bounds
    }
    
    func loadHTMLString(_ html: String) {
        webView?.loadHTMLString(html, baseURL: nil)
    }
    
//    func loadBundleApp(_ app: String) {
//        let url = Bundle.main.url(forResource: app, withExtension: "html")!
//        webView?.loadFileURL(url, allowingReadAccessTo: URL(string: "file://")!) // needed in order to load local files from anywhere
//    }
//
//    func loadLocalApp(_ appPath: String) {
//        let localURL = URL(fileURLWithPath: appPath)
//
//        webView?.loadFileURL(localURL, allowingReadAccessTo: URL(string: "file://")!) // needed in order to load local files from anywhere
//    }
//
//    //""
//    func loadRemoteApp(at url: URL) {
//        webView?.load(URLRequest(url: url))
//    }
    

    
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
        print("hello")

        ShellBridge.injectStringIntoTerminal(notification.object as! String, runImmediately: true, completion: {
            if let currentMouseLocation = self.mouseLocation {
               print("mouseLocation:", currentMouseLocation)
               print("mouseInWindow", self.view.bounds.contains(currentMouseLocation))
               if (self.view.bounds.contains(currentMouseLocation)) {
                   NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
               }
           }
        })
    }
    
    
}

extension WebViewController: ShellBridgeEventListener {
    
    @objc func recievedDataFromPipe(_ notification: Notification) {
        //Bring FIG to front when triggered explictly from commandline
        NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
        
        let msg = (notification.object as! ShellMessage)
        DispatchQueue.main.async {
            FigCLI.route(msg,
                         webView: self.webView!,
                         companionWindow: self.view.window as! CompanionWindow)
        }
    }
}


extension WebViewController : WKNavigationDelegate {
    func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
        print(error.localizedDescription)
    }
    
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        print("didFinishNavigation")
        var scriptContent = "var meta = document.createElement('meta');"
        scriptContent += "meta.name='viewport';"
        scriptContent += "meta.content='width=device-width';"
        scriptContent += "document.getElementsByTagName('head')[0].appendChild(meta);"

        webView.evaluateJavaScript(scriptContent, completionHandler: nil)
        
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
        self.unregisterDraggedTypes()
//        self.autoresizingMask = NSView.AutoresizingMask.init(arrayLiteral: [.height, .width])
//        NSEvent.addLocalMonitorForEvents(matching: NSEvent.EventTypeMask.mouseEntered) { event -> NSEvent? in
//            print(event)
//            return event
//        }
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    override func mouseDown(with event: NSEvent) {
        NSApp.preventWindowOrdering()
        super.mouseDown(with: event)
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
    
    override func mouseEntered(with event: NSEvent) {
        print("mouse entered")
        //NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
    }
    
    override func mouseExited(with event: NSEvent) {
        print("mouse exited")
        //ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
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
        self.loadFileURL(url, allowingReadAccessTo: URL(string: "file://")!) // needed in order to load local files from anywhere
    }
    
    func loadRemoteApp(at url: URL) {
        print(url.absoluteString)
        self.load(URLRequest(url: url))
    }
    
    func loadHomeScreen() {
        self.load(URLRequest(url: URL(string: "https://app.withfig.com")!))

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
