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

  init(_ configuration: WKWebViewConfiguration = WKWebViewConfiguration()) {

    super.init(nibName: nil, bundle: nil)
    let settings = WebBridge.shared.configure(configuration)
    webView = WebView(frame: .zero, configuration: settings)
    webView?.drawsBackground = false
    self.view = webView!
    webView?.translatesAutoresizingMaskIntoConstraints = false

    webView?.uiDelegate = self
    webView?.navigationDelegate = self

    if UserDefaults.standard.string(forKey: "debugMode") != "enabled" {
      NSLayoutConstraint.activate([
        webView!.topAnchor.constraint(equalTo: view.topAnchor),
        webView!.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        webView!.leftAnchor.constraint(equalTo: view.leftAnchor),
        webView!.rightAnchor.constraint(equalTo: view.rightAnchor)
      ])
    }

  }

  deinit {
    NotificationCenter.default.removeObserver(self)
  }

  required init?(coder: NSCoder) {
    fatalError("init(coder:) has not been implemented")
  }

  //    override func loadView() {
  //        self.view = TransparentView(frame: .zero)
  //    }

  //    override func loadView() {
  //        self.view = webView
  //    }
  //    override func loadView() {
  //      self.view = webView!
  //        self.view = NSView(frame: .zero)
  //        self.view.wantsLayer = true
  //      self.view.layer?.backgroundColor = NSColor.clear.cgColor

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

  //        let effect = NSVisualEffectView(frame: .zero)
  //        effect.blendingMode = .behindWindow
  //        effect.state = .active
  //        effect.material = .dark
  //        effect.maskImage = _maskImage(cornerRadius: 5)
  //
  //
  //        self.view = effect// NSView(frame: .zero);
  ////         view.setValue(false, forKey: "drawsBackground")
  //        self.view.postsFrameChangedNotifications = true
  //        self.view.postsBoundsChangedNotifications = true
  //
  //
  //
  //
  //   }
  override func viewDidAppear() {
    // add alpha when using NSVisualEffectView
    // ADD ALPHA TO WINDOW
    // self.view.window?.alphaValue = 0.9

    print("ViewDidAppear -- \( webView?.url?.absoluteString ?? "no url")")

    if let url = webView?.defaultURL {
      webView?.loadRemoteApp(at: url)
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

  @objc func overlayDidBecomeIcon() {

  }

  @objc func overlayDidBecomeMain() {
    print("didBecomeMain")
    //        self.icon.isHidden = true
    //        self.webView?.loadHomeScreen()
    // (self.view as! NSVisualEffectView).maskImage = _maskImage(cornerRadius: 15)

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
       let satFilter = CIFilter(name: "CIColorControls") {

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
class TransparentView: NSView {

  var alphaLevel: Double = 0.0

  override func draw(_ dirtyRect: NSRect) {
    NSColor(deviceWhite: 255, alpha: CGFloat(alphaLevel)).set()
    dirtyRect.fill()
    //        NSRectFill(dirtyRect)
    //        NSRect.fil
  }

}

extension WebViewController {

  func cleanUp() {
    self.webView?.configuration.userContentController.removeAllUserScripts()

    for handler in WebBridgeScript.allCases {
      self.webView?.configuration.userContentController.removeScriptMessageHandler(forName: handler.rawValue)

    }

  }

}

extension WebViewController: WKUIDelegate {
  func webView(
    _ webView: WKWebView,
    createWebViewWith configuration: WKWebViewConfiguration,
    for navigationAction: WKNavigationAction,
    windowFeatures: WKWindowFeatures
  ) -> WKWebView? {
    print("hello")
    if navigationAction.targetFrame == nil {
      return self.webView
    }
    return nil
  }
}

extension WebViewController: WKNavigationDelegate {
  func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
    print(webView.url?.absoluteString ?? "?")
    // swiftlint:disable force_cast
    let webView = webView as! WebView

    for onNavigateCallback in webView.onNavigate {
      onNavigateCallback()
    }
    webView.onNavigate = []
  }

  func webView(
    _ webView: WKWebView,
    decidePolicyFor navigationAction: WKNavigationAction,
    decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
  ) {

    // decisionHandler(.cancel)
    // print(navigationAction.navigationType)

    if let url = navigationAction.request.url, navigationAction.modifierFlags.contains(.command) {
      NSWorkspace.shared.open(url)

      decisionHandler(.cancel)
      return
    }

    decisionHandler(.allow)

    // swiftlint:disable force_cast
    let webView = webView as! WebView

    for onNavigateCallback in webView.onNavigate {
      onNavigateCallback()
    }
    webView.onNavigate = []
    webView.requestedURL = navigationAction.request.url
    webView.window?.title = ""
    webView.window?.representedURL = nil
  }

  func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
    print("ERROR Loading URL: \(error.localizedDescription)")
    webView.window?.title = "Could not load URL..."

    if let webView = webView as? WebView {
      webView.loadArchivedURL()
    }

  }

  func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
    print("Loaded URL \(webView.url?.absoluteString ?? "<none>")")
    var scriptContent = "var meta = document.createElement('meta');"
    scriptContent += "meta.name='viewport';"
    scriptContent += "meta.content='width=device-width';"
    scriptContent += "document.getElementsByTagName('head')[0].appendChild(meta);"

    webView.evaluateJavaScript(scriptContent, completionHandler: nil)

    // swiftlint:disable force_cast
    let webView = webView as! WebView

    if let configureEnv = webView.configureEnvOnLoad {
      configureEnv()
    }

    for onLoadCallback in webView.onLoad {
      onLoadCallback()
    }
    webView.onLoad = []

    // Automatically archive this URL for offline use
    webView.archive()
  }
}

class WebView: WKWebView {
  var trackingArea: NSTrackingArea?
  var trackMouse = true
  var onLoad: [(() -> Void)] = []
  var onNavigate: [(() -> Void)] = []
  var configureEnvOnLoad: (() -> Void)?
  var defaultURL: URL? = Remote.baseURL.appendingPathComponent("sidebar")
  var dragShouldRepositionWindow = false
  private var dragging = false
  var drawsBackground: Bool = false {
    didSet {
      self.setValue(self.drawsBackground, forKey: "drawsBackground")
    }
  }

  override var canGoBack: Bool {
    return !(super.backForwardList.backItem?.initialURL.absoluteString
             == Remote.baseURL.appendingPathComponent("sidebar").absoluteString) && super.canGoBack
  }
  var requestedURL: URL?

  //    override func shouldDelayWindowOrdering(for event: NSEvent) -> Bool {
  //        return true
  //    }

  override init(frame: CGRect, configuration: WKWebViewConfiguration) {
    super.init(frame: frame, configuration: configuration)
    //        self.setValue(false, forKey: "drawsBackground")

    NotificationCenter.default.addObserver(
      self,
      selector: #selector(requestStopMonitoringMouseEvents(_:)),
      name: .requestStopMonitoringMouseEvents,
      object: nil
    )
    NotificationCenter.default.addObserver(
      self,
      selector: #selector(requestStartMonitoringMouseEvents(_:)),
      name: .requestStartMonitoringMouseEvents,
      object: nil
    )
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

    guard self.dragShouldRepositionWindow else { return }

    let loc = event.locationInWindow
    let height = self.window!.frame.height
    if loc.y > height - 28 {
      self.dragging = true
    }
  }

  override func mouseUp(with event: NSEvent) {
    super.mouseUp(with: event)
    dragging = false
  }

  override func mouseDragged(with event: NSEvent) {
    super.mouseDragged(with: event)

    if self.dragging {
      self.window?.performDrag(with: event)
    }
  }

  override func mouseEntered(with event: NSEvent) {
    print("mouse entered")
    guard let windowGeneric = self.window, let window = windowGeneric as? CompanionWindow else {
      return
    }
    if trackMouse &&
        !NSWorkspace.shared.frontmostApplication!.isFig &&
        window.positioning == CompanionWindow.defaultPassivePosition {
      print("current frontmost application \(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "")")

      self.evaluateJavaScript("fig.mouseEntered()", completionHandler: nil)
      print("Attempting to activate fig")
      if Defaults.shared.triggerSidebarWithMouse {
        WindowManager.shared.windowServiceProvider.takeFocus()
      }

      //            NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
    }
  }

  override func mouseExited(with event: NSEvent) {
    print("mouse exited")
    guard let windowGeneric = self.window, let window = windowGeneric as? CompanionWindow else {
      return
    }
    if trackMouse && (NSWorkspace.shared.frontmostApplication?.isFig ?? false
                  || WindowManager.shared.windowServiceProvider.isActivating)
                  && window.positioning == CompanionWindow.defaultPassivePosition {
      print("current frontmost application \(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "")")
      let identifier = ShellBridge.shared.previousFrontmostApplication?.bundleIdentifier ?? "<none>"
      print("Attempting to activate previous app \(identifier)")
      //            ShellBridge.shared.previousFrontmostApplication?.activate(options: .init())
      if Defaults.shared.triggerSidebarWithMouse {
        WindowManager.shared.windowServiceProvider.returnFocus()
      }

    }
  }

  func loadBundleApp(_ app: String) {

    if let url = Bundle.main.url(forResource: app, withExtension: "html") {
      // needed in order to load local files from anywhere
      self.loadFileURL(url, allowingReadAccessTo: URL(string: "file://")!)
    } else {
      print("Bundle app '\(app)' does not exist")
    }
  }

  func loadLocalApp(_ url: URL) {
    //        let localURL = URL(fileURLWithPath: appPath)
    self.evaluateJavaScript("document.documentElement.remove()") { (_, _) in
      // needed in order to load local files from anywhere
      self.loadFileURL(url, allowingReadAccessTo: URL(string: "file://")!)
    }
  }

  func loadRemoteApp(at url: URL) {
    print(url.absoluteString)
    //        self.load(URLRequest(url: URL(string:"about:blank")!))
    self.load(URLRequest(url: url, cachePolicy: .reloadIgnoringLocalAndRemoteCacheData))

    self.evaluateJavaScript("document.documentElement.remove()") { (_, _) in
    }
  }

  func loadHomeScreen() {
    self.evaluateJavaScript("document.documentElement.remove()") { (_, _) in

      self.load(URLRequest(url: Remote.baseURL, cachePolicy: .useProtocolCachePolicy))
    }

  }

  func loadAutocomplete(from urlString: String?
                        = Settings.shared.getValue(forKey: Settings.autocompleteURL) as? String) {

    let url: URL = {

      // Use value specified by developer.autocomplete.host if it exists
      if let urlString = urlString,
         let url = URL(string: urlString) {
        return url
      }

      // otherwise use fallback
      return Remote.baseURL.appendingPathComponent("autocomplete")
        .appendingPathComponent(Defaults.shared.autocompleteVersion ?? "")
    }()

    Logger.log(message: "Loading autocomplete (\(url.absoluteString))...")

    self.load(URLRequest(url: url, cachePolicy: .reloadIgnoringLocalAndRemoteCacheData))
  }

  func clearHistory() {
    self.backForwardList.perform(Selector(("_removeAllItems")))
  }

  func deleteCache() {
    WebView.deleteCache()
  }

  func openWebInspector() {
    // WKInspectorShowConsole(WKPageGetInspector((wkwebview.subviews.first as! WKView).pageRef))
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

  static func deleteCache() {
    let websiteDataTypes = NSSet(array: [
      WKWebsiteDataTypeDiskCache,
      WKWebsiteDataTypeMemoryCache,
      WKWebsiteDataTypeCookies,
      WKWebsiteDataTypeLocalStorage
    ])
    let date = Date(timeIntervalSince1970: 0)

    // swiftlint:disable force_cast
    WKWebsiteDataStore.default().removeData(ofTypes: websiteDataTypes as! Set<String>,
                                            modifiedSince: date,
                                            completionHandler: { })
  }

  // swiftlint:disable line_length
  // click through https://stackoverflow.com/questions/128015/make-osx-application-respond-to-first-mouse-click-when-not-focused/129148
  override func acceptsFirstMouse(for event: NSEvent?) -> Bool {
    return true
  }

  //    override func shouldDelayWindowOrdering(for event: NSEvent) -> Bool {
  //        return false
  //    }

}

extension WebView: MouseMonitoring {
  @objc func requestStopMonitoringMouseEvents(_ notification: Notification) {
    self.trackMouse = false
  }

  @objc func requestStartMonitoringMouseEvents(_ notification: Notification) {
    self.trackMouse = true

  }

}

extension NSView {
  // swiftlint:disable line_length
  /// Adds constraints to this `UIView` instances `superview` object to make sure this always has the same size as the superview.
  // swiftlint:disable line_length
  /// Please note that this has no effect if its `superview` is `nil` – add this `UIView` instance as a subview before calling this.
  func bindFrameToSuperviewBounds() {
    guard let superview = self.superview else {
      // swiftlint:disable line_length
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
