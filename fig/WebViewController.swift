//
//  WebViewController.swift
//  fig
//
//  Created by Matt Schrage on 4/17/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
import WebKit

class WebViewController: NSViewController, NSWindowDelegate {
    var mouseLocation: NSPoint? { self.view.window?.mouseLocationOutsideOfEventStream }

    var webView: WKWebView? // = WKWebView(frame:.zero)

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
        

        

    }
    
    override func viewDidAppear() {

//        blur(view:self.view)
        view.window?.delegate = self
//        webView = WKWebView(frame: self.view.window?.frame ?? .zero)
        webView?.frame = self.view.frame
//        webView.ba
        webView?.navigationDelegate = self
        NotificationCenter.default.addObserver(self, selector: #selector(recievedDataFromPipe(_:)), name: .recievedDataFromPipe, object: nil)



//        webView?.setValue(false, forKey: "drawsBackground")
        self.view.addSubview(webView!)
        
        // add alpha when using NSVisualEffectView
        self.view.window?.alphaValue = 0.9
        
//        self.view.wantsLayer = true
//        self.view.layer?.cornerRadius = 15
//        self.view.layer?.masksToBounds = true
//        self.webView.alphaValue = 0.75
//        self.view.alphaValue = 0.5;
        
        

    

    }




    func windowDidResize(_ notification: Notification) {
        // This will print the window's size each time it is resized.
//        self.view.frame = self.view.window?.frame ?? .zero
        print(view.window?.frame.size ?? "<none>", self.webView!.frame.size)
        self.webView!.frame = self.view.frame
//        self.webView.reload()
    }
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
      
        
        
        webView = WebView(frame: self.view.frame, configuration: WebBridge(eventDelegate: self))

        // Do any additional setup after loading the view.
//        webView.loadHTMLString("<html><body bgcolor = \"red\"><p>Hello, World!</p></body></html>", baseURL: nil)
        
        let url = Bundle.main.url(forResource: "finder", withExtension: "html")!
        webView?.loadFileURL(url, allowingReadAccessTo: URL(string: "file://")!) // needed in order to load local files from anywhere
//        NSURL *baseURL = [NSURL fileURLWithPath: resourcePath];

//       let request = URLRequest(url: url)
//       webView?.load(request)
        
        //URL(fileURLWithPath: NSHomeDirectory(), isDirectory: true)
//        do {
//        //let path = try FileManager.default.url(for: .userDirectory, in: .allDomainsMask, appropriateFor: nil, create: false)
//            let path = URL(string: "file://")!
//        webView?.loadFileURL(url, allowingReadAccessTo: path) // needed in order to load local files from anywhere
////        NSURL *baseURL = [NSURL fileURLWithPath: resourcePath];
//
//               let request = URLRequest(url: url)
//               webView?.load(request)
//        } catch {
//            print("huh");
//    }
        //"https://medium.com/@dmytro.anokhin/command-line-tool-using-swift-package-manager-and-utility-package-e0984224fc04"
//        webView?.load(URLRequest(url: URL(string: "https://app.withfig.com")!))
    }
    
    func loadHTMLString(_ html: String) {
        webView?.loadHTMLString(html, baseURL: nil)
    }
    
    func loadLocalApp(_ app: String) {
        let url = Bundle.main.url(forResource: app, withExtension: "html")!
        webView?.loadFileURL(url, allowingReadAccessTo: URL(string: "file://")!) // needed in order to load local files from anywhere
    }
    
    //"https://app.withfig.com"
    func loadRemoteApp(at url: URL) {
        webView?.load(URLRequest(url: url))
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


extension WebViewController : WebBridgeEventDelegate {
    func requestExecuteCLICommand(script: String) {
        print(script)
        //  NSRunningApplication.current.hide()
        print(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "<none>");
       
        ShellBridge.injectStringIntoTerminal(script, runImmediately: true, completion: {
            if let currentMouseLocation = self.mouseLocation {
               print("mouseLocation:", currentMouseLocation)
               print("mouseInWindow", self.view.bounds.contains(currentMouseLocation))
               if (self.view.bounds.contains(currentMouseLocation)) {
                   NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
               }
           }
        })
    
        
       
        
//        self.view.window?.orderOut(nil)
//        ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
//        ShellBridge.delayWithSeconds(0.3) {
//               print(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "<none>");
//               ShellBridge.injectStringIntoTerminal(script, runImmediately: true)
//
//        }
        
    }
    
    func requestInsertCLICommand(script: String) {
//        NSApp.preventWindowOrdering()
//        NSApplication.shared.preventWindowOrdering()
        print(script)
//        NSRunningApplication.current.hide()
        print(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "<none>");
//        self.view.window?.resignKey()
//        self.view.window?.orderOut(nil)
//        ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
//        ShellBridge.delayWithSeconds(0.05) {
            ShellBridge.injectStringIntoTerminal(script, runImmediately: false)
//        }

        print(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "<none>");
//        ShellBridge.delayWithSeconds(0.5) {
//            print(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "<none>");
//        }
//        self.view.window?.orderOut(nil)

    }
    
    func requestNextSection() {
        print("next")

    }
    
    func requestPreviousSection() {
        print("prev")
    }
    
    func startTutorial(identifier: String) {
        print(identifier)

    }
    
}

extension WebViewController: ShellBridgeEventListener {
//    func recievedDataFromPipe(_ notification: NSNotification) {
//        <#code#>
//    }
    

    
    @objc func recievedDataFromPipe(_ notification: Notification) {
        let msg = (notification.object as! ShellMessage)
        let stdin = msg.data.replacingOccurrences(of: "`", with: "\\`")
        print("stdin: \(stdin)")
//        let trimmed = (notification.object as! String).trimmingCharacters(in:
////            NSCharacterSet.whitespacesAndNewlines
////        )
//        print("Open URL \(trimmed)")
        print(ShellBridge.commandLineOptionsToURL(msg.options ?? []))
        
        DispatchQueue.main.async {
            if let options = msg.options {
                switch options[0] {
                case "callback":
                    self.webView?.evaluateJavaScript("fig.\(options[1])(`\(stdin)`)", completionHandler: nil)
                    break;
                case "editor", "finder","viewer":
                    self.loadLocalApp(options[0])
                    ShellBridge.delayWithSeconds(0.5) {
                        self.webView?.evaluateJavaScript("fig.stdin(`\(stdin)`)", completionHandler: nil)
                    }

                    break
                case "web":
                    self.loadRemoteApp(at: URL(string: options[1]) ?? URL(string:"https://app.withfig.com")!)
                case "google":
                    self.loadRemoteApp(at: URL(string: "https://google.com/search?q=\(options.suffix(from: 1).joined(separator: " ").addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? "")") ?? URL(string:"https://app.withfig.com")!)
                case "home":
                      self.loadRemoteApp(at: URL(string:"https://app.withfig.com")!)
                default:
                    print("unrecognized option");
                }
            } else {
                self.webView?.evaluateJavaScript("fig.stdin(`\(stdin)`)", completionHandler: nil)
            }

//            self.webView?.load(URLRequest(url: URL(string: trimmed)!))
        }
    }
    
    
}


extension WebViewController : WKNavigationDelegate {
    func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
        print(error.localizedDescription)
    }
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        
//        self.webView?.evaluateJavaScript("document.body.style = document.body.style.cssText + \";background: transparent !important;\";", completionHandler: nil)
//        
        
        self.webView?.evaluateJavaScript("document.readyState", completionHandler: { (complete, error) in
            if complete != nil {
                self.webView?.evaluateJavaScript("document.body.scrollHeight", completionHandler: { (height, error) in
                    let h = height as! CGFloat
                    print(h)
                })
                
            }

            })
    }
}

class WebView : WKWebView {
    var trackingArea : NSTrackingArea?

    override func shouldDelayWindowOrdering(for event: NSEvent) -> Bool {
        return true
    }

    override init(frame: CGRect, configuration: WKWebViewConfiguration) {
        super.init(frame: frame, configuration: configuration)
        self.unregisterDraggedTypes()
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
//        self.window?.makeKeyAndOrderFront(nil)
        NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
    }
    
    override func mouseExited(with event: NSEvent) {
        print("mouse exited")
        ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)

//        self.window?.orderOut(self)
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
