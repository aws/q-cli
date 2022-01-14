//
//  CompanionWindow.swift
//  fig
//
//  Created by Matt Schrage on 4/18/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa

extension Notification.Name {
  static let overlayDidBecomeIcon = Notification.Name("overlayDidBecomeIcon")
  static let overlayDidBecomeMain = Notification.Name("overlayDidBecomeMain")

}

class CompanionWindow: NSWindow, NSWindowDelegate {
  static let defaultActivePosition: OverlayPositioning = Defaults.shared.defaultActivePosition
  static let defaultPassivePosition: OverlayPositioning = .sidebar

  var floatingWindowLevel: NSWindow.Level {
    // .statusBar level is required in order to appear above iTerm in Quake mode
    guard let rawWindowLevel = tetheredWindow?.windowLevel else {
      return .floating
    }

    let windowLevel = NSWindow.Level(rawValue: Int(rawWindowLevel))

    if windowLevel == NSWindow.Level(rawValue: 0) {
      return .floating
    }
    return windowLevel
  }
  // hides companion window when target is moving
  var shouldTrackWindow = true

  var isDocked = true
  var isHidden: Bool {
    return self.frame.height == 1 || self.frame.height == 0 || !self.isVisible
  }

  var oneTimeUse = false

  var closeBtn: PointableButton?
  var backBtn: NSTextField?
  var untetherBtn: NSTextField?

  var tetheredWindowId: CGWindowID? {
    return tetheredWindow?.windowId
  }
  var tetheredWindow: ExternalWindow? {
    didSet {
      if let id = self.tetheredWindow?.hash {
        self.webView?.evaluateJavaScript("fig.windowId = '\(id)'", completionHandler: nil)
      }
    }
  }
  var sessionId: String?

  let windowManager: WindowManagementService
  let windowServiceProvider: WindowService = WindowServer.shared

  var width: CGFloat?
  var maxHeight: CGFloat?
  var anchorOffsetPoint: CGPoint = .zero
  var priorTargetFrame: NSRect = .zero
  var positioning: OverlayPositioning = CompanionWindow.defaultActivePosition {

    didSet {

      if oneTimeUse && positioning == .sidebar {
        self.windowManager.close(window: self)
        return
      }
      self.maxHeight = nil
      self.windowManager.requestWindowUpdate()

      if !positioning.hasTitleBar {
        isDocked = true
        closeBtn?.removeFromSuperview()
        backBtn?.removeFromSuperview()
        untetherBtn?.removeFromSuperview()
      } else {
        setupTitleBar()
      }

      self.repositionWindow(forceUpdate: true, explicit: true)
      if let webViewController = self.contentViewController as? WebViewController {
        if positioning == CompanionWindow.defaultPassivePosition {
          webViewController.overlayDidBecomeIcon()
          // NotificationCenter.default.post(name: .overlayDidBecomeIcon, object: nil)
        } else {
          webViewController.overlayDidBecomeMain()
          // NotificationCenter.default.post(name: .overlayDidBecomeMain, object: nil)
        }
      }
    }
  }

  var status: (String, String?, Bool)?
  var loaded: Bool = false

  var initialUntetheredFrame: NSRect?

  override public var canBecomeKey: Bool {
    return true
  }

  var timer: Timer?

  var webViewController: WebViewController? {
    return self.contentViewController as? WebViewController
  }

  func windowWillClose(_ notification: Notification) {
    if let timer = self.timer {
      timer.invalidate()
    }

    print("WindowWillClose")
  }

  init(viewController: NSViewController, windowManager: WindowManagementService = WindowManager.shared) {
    self.windowManager = windowManager
    super.init(
      contentRect: NSRect(x: 0, y: 0, width: 480, height: 300),
      styleMask: [.fullSizeContentView],
      backing: .buffered, defer: false)
    self.center()
    self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
    self.isMovableByWindowBackground = true
    self.isOpaque = true
    self.backgroundColor = .clear // NSColor.init(white: 1, alpha: 0.75)
    self.delegate = self
    self.level = self.floatingWindowLevel
    self.setFrameAutosaveName("Main Window")
    self.contentViewController = viewController
    self.setFrame(NSRect(x: 400, y: 400, width: 300, height: 300), display: true)
    //        self.makeKeyAndOrderFront(nil)

    self.delegate = self

    if let disableTransparency = Settings.shared.getValue(forKey: Settings.disableWebviewTransparency) as? Bool,
       disableTransparency {
      self.backgroundColor = .red
    }

    NSWorkspace.shared.notificationCenter.addObserver(
      self,
      selector: #selector(spaceChanged),
      name: NSWorkspace.activeSpaceDidChangeNotification,
      object: nil
    )
  }

  override func mouseEntered(with event: NSEvent) {
    print("mouse entered...")

  }

  override func mouseExited(with event: NSEvent) {
    print("mouse exited...")
  }

  // this was done to prevent untethered windows from jumping to the front when the application is activate (eg. when
  // the user mouses over the sidebar)
  override var canBecomeMain: Bool {
    return self.isDocked
  }

  @objc func activateApp() {
    print("didActivateApp", NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "<none>")
    forceReposition()
  }

  @objc func deactivateApp() {
    print("didDectivateApp", NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "<none>")
    forceReposition()
  }

  @objc func forceReposition() {
    repositionWindow(forceUpdate: true, explicit: false)
  }

  @objc func positionWindow() {
    repositionWindow(forceUpdate: false)
  }

  enum Style: Int {
    case tethered
    case untethered
  }

  enum OverlayPositioning: Int {
    case coveringTitlebar = 0
    case insideRightFull = 1
    case insideRightPartial = 2
    case outsideRight = 3
    case atPrompt = 4
    case icon = 5
    case notification = 6
    case sidebar = 7
    case fullscreen = 8
    case spotlight = 9
    case fullscreenInset = 10
    case hidden = 11
    case untethered = 12
    case fullwindow = 13
    case popover = 14
    case popoverCentered = 15
    case powerbar = 16

    func frame(targetWindowFrame: NSRect, screen: NSRect = .zero) -> NSRect {
      if targetWindowFrame.width < 100 || targetWindowFrame.height < 200 {
        return .zero
      }

      let t_size = targetWindowFrame.size
      switch self {
      case .coveringTitlebar:
        return NSRect(origin: targetWindowFrame.origin, size: CGSize.init(width: t_size.width, height: 100))
      case .insideRightFull:
        return targetWindowFrame.divided(atDistance: 300, from: .maxXEdge).slice
      case .insideRightPartial:
        return targetWindowFrame.divided(atDistance: 300, from: .maxXEdge).slice
          .divided(atDistance: t_size.height * ( 2 / 3 ), from: .maxYEdge).slice.offsetBy(dx: 0, dy: -t_size.height / 3)
      case .atPrompt:

        let inner = targetWindowFrame.insetBy(dx: 30, dy: 45)
        return NSRect(x: inner.origin.x, y: inner.origin.y - inner.height, width: inner.width, height: 100)
      case .outsideRight:
        let outerFrame = targetWindowFrame.insetBy(dx: -300, dy: 0).divided(atDistance: 300, from: .maxXEdge).slice

        let intersection = screen.intersection(outerFrame)
        var x = outerFrame.origin.x
        if intersection.width != outerFrame.width {
          x -= outerFrame.width - intersection.width
        }

        return NSRect(origin: NSPoint(x: x, y: outerFrame.origin.y), size: outerFrame.size)
      case .icon:
        let i_size =  CGSize(width: 50, height: 50)
        let i_padding = CGPoint(x: 12, y: -36)
        return NSRect(origin: CGPoint.init(x: targetWindowFrame.maxX - i_size.width - i_padding.x,
                                           y: targetWindowFrame.minY - i_size.height - i_padding.y), size: i_size)
      case .notification:
        let i_size =  CGSize(width: 300, height: 120)
        let i_padding = CGPoint(x: 12, y: -120 + 12)
        return NSRect(origin: CGPoint.init(x: targetWindowFrame.maxX - i_size.width - i_padding.x,
                                           y: targetWindowFrame.minY - i_size.height - i_padding.y), size: i_size)
      case .sidebar:
        let outerFrame =  targetWindowFrame.divided(atDistance: 50, from: .maxXEdge).slice.offsetBy(dx: 50, dy: 0)
        let intersection = screen.intersection(outerFrame)
        var x = outerFrame.origin.x
        if intersection.width != outerFrame.width {
          x -= outerFrame.width - intersection.width
        }

        return NSRect(origin: NSPoint(x: x, y: outerFrame.origin.y), size: outerFrame.size)
      case .fullscreen:
        return targetWindowFrame
      case .spotlight:
        let minWidth: CGFloat = 400.0
        let minHeight: CGFloat = 300.0

        let width = min(max(minWidth, t_size.width * 0.5), t_size.width)
        let height = min(max(minHeight, t_size.height * 0.5), t_size.height)
        let offset = max((t_size.width - width) / 2, 0)

        // let quarter = max(t_size.width * 0.25 - minWidth / 2.0, 0)
        return NSRect(
          origin: NSPoint(x: targetWindowFrame.origin.x + offset, y: targetWindowFrame.origin.y - 23),
          size: CGSize.init(width: width, height: height)
        )
      case .fullscreenInset:
        let inset: CGFloat = 23
        return targetWindowFrame.insetBy(dx: 0, dy: inset/2).offsetBy(dx: 0, dy: -1 * inset * 1.5)

      //                let inset: CGFloat = 30
      // return targetWindowFrame.insetBy(dx: inset, dy: inset).offsetBy(dx: 0, dy: -1 * inset - 24)
      case .hidden:
        return .zero
      case .untethered:
        let window: CGSize = CGSize(width: 715, height: 475)
        let inset: CGPoint = CGPoint(x: (screen.width - window.width) / 2, y: (screen.height - window.height) / 2)
        return screen.insetBy(dx: inset.x, dy: inset.y).offsetBy(dx: 0, dy: screen.height - (2 * inset.y))
      case .fullwindow:
        let inset: CGPoint = CGPoint(x: 250, y: 150)
        let calculated = screen.insetBy(dx: inset.x, dy: inset.y).offsetBy(dx: 0, dy: screen.height - (2 * inset.y))
        let maxWidth: CGFloat = 1100.0
        let maxHeight: CGFloat = 900.0

        let width: CGFloat =  min(maxWidth, calculated.width)
        let height: CGFloat =  min(maxHeight, calculated.height)
        let x = calculated.origin.x + (( calculated.width - width) / 2)
        let y = calculated.origin.y - (( calculated.height - height) / 2)

        return NSRect(x: x, y: y, width: width, height: height)
      case .popover:
        let minHeight: CGFloat = 300.0
        let height = min(max(minHeight, t_size.height / 3), t_size.height)
        let offset = t_size.height - height

        return NSRect(
          x: targetWindowFrame.origin.x,
          y: targetWindowFrame.origin.y - offset,
          width: targetWindowFrame.width,
          height: height
        )

      case .popoverCentered:
        let minWidth: CGFloat = 300.0
        let minHeight: CGFloat = 200.0

        let width: CGFloat =  min(max(minWidth, t_size.width * 2 / 3), t_size.width)
        let height: CGFloat =  min(max(minHeight, t_size.height * 2 / 3), t_size.height)
        let x = targetWindowFrame.origin.x + (( t_size.width - width) / 2)
        let y = targetWindowFrame.origin.y - (( t_size.height - height) / 2)

        return NSRect(x: x, y: y, width: width, height: height)

      case .powerbar:
        let bar = NSRect(
          x: targetWindowFrame.origin.x,
          y: targetWindowFrame.origin.y - targetWindowFrame.height,
          width: targetWindowFrame.width,
          height: 50
        )
        // let intersection = screen.intersection(bar)
        var y = bar.origin.y
        if y - bar.height < 0 {
          y += (bar.height - y)
        }

        return NSRect(origin: NSPoint(x: bar.origin.x, y: y), size: bar.size)
      }

    }

    var hasTitleBar: Bool {
      let titlebarStates: Set<OverlayPositioning> = [
        .outsideRight,
        .untethered,
        .fullscreenInset,
        .fullwindow,
        .popoverCentered
      ]
      return titlebarStates.contains(self)
    }

  }
  func setupTitleBar() {
    if self.backgroundColor == .clear {
      self.backgroundColor = .white
    }

    self.isOpaque = false

    closeBtn?.removeFromSuperview()
    backBtn?.removeFromSuperview()
    untetherBtn?.removeFromSuperview()

    closeBtn = PointableButton(title: "✕", target: self, action: #selector(toSidebar))
    closeBtn?.font = NSFont.systemFont(ofSize: 14)
    closeBtn?.bezelStyle = .circular
    closeBtn?.frame = CGRect(x: 0, y: 0, width: 22, height: 20)

    self.addViewToTitleBar(closeBtn!, at: 4, offset: 1)
    //        closeBtn?.addCursorRect(closeBtn?.bounds ?? .zero, cursor: NSCursor.pointingHand)

    if self.webView?.canGoBack ?? false {
      backBtn = NSTextField()
      backBtn?.frame = CGRect(origin: .zero, size: CGSize(width: 50, height: 44))
      backBtn?.stringValue = "←"
      backBtn?.font = NSFont.systemFont(ofSize: 18)
      backBtn?.alignment = .left
      backBtn?.backgroundColor = .clear
      backBtn?.isBezeled = false
      backBtn?.isEditable = false
      backBtn?.sizeToFit()
      self.addViewToTitleBar(backBtn!, at: 32, offset: 1)
    }
    //        backBtn?.addCursorRect(backBtn?.bounds ?? .zero, cursor: NSCursor.pointingHand)

    let backClick = NSClickGestureRecognizer(target: self, action: #selector(self.backButtonClicked))
    backBtn?.addGestureRecognizer(backClick)

    if self.isDocked {
      untetherBtn = NSTextField()
      untetherBtn?.frame = CGRect(origin: .zero, size: CGSize(width: 50, height: 44))
      untetherBtn?.stringValue = "↗"
      untetherBtn?.font = NSFont.systemFont(ofSize: 20)
      untetherBtn?.alignment = .right
      untetherBtn?.backgroundColor = .clear
      untetherBtn?.isBezeled = false
      untetherBtn?.isEditable = false
      untetherBtn?.sizeToFit()
      self.addViewToTitleBar(untetherBtn!, at: self.frame.width - 24, offset: 1) // 276
      //        untetherBtn?.addCursorRect(untetherBtn?.bounds ?? .zero, cursor: NSCursor.pointingHand)

      let toggleClick = NSClickGestureRecognizer(target: self, action: #selector(self.untether))
      untetherBtn?.addGestureRecognizer(toggleClick)
    }

    self.invalidateCursorRects(for: self.closeBtn!)
  }

  //    func toolbarConfig() {
  //        self.level = .floating
  //        self.collectionBehavior = [.managed]
  //        self.styleMask = [.titled, .resizable]
  //
  //        if (positioning == .untethered) {
  //            self.styleMask.insert(.resizable)
  //        }
  //
  //        if let closeButton = self.standardWindowButton(NSWindow.ButtonType.closeButton) {
  //            closeButton.isHidden = true
  //        }
  //
  //        if let miniaturizeButton = self.standardWindowButton(NSWindow.ButtonType.miniaturizeButton) {
  //            miniaturizeButton.isHidden = true
  //        }
  //
  //        if let zoomButton = self.standardWindowButton(NSWindow.ButtonType.zoomButton) {
  //            zoomButton.isHidden = true
  //        }
  //
  //        self.titlebarAppearsTransparent = true;
  //    }

  func configureWindow(for state: OverlayPositioning, initial: Bool = false ) {
    if state.hasTitleBar {
      self.level = (self.isDocked) ? self.floatingWindowLevel : .normal
      self.collectionBehavior = (self.isDocked) ? [.canJoinAllSpaces, .fullScreenAuxiliary] : []

      self.styleMask = [.titled]

      // this must be explictly inserted... cannot be included in a styleMask array(?!)
      self.styleMask.insert(.resizable)

      if let closeButton = self.standardWindowButton(NSWindow.ButtonType.closeButton) {
        closeButton.isHidden = true
      }

      if let miniaturizeButton = self.standardWindowButton(NSWindow.ButtonType.miniaturizeButton) {
        miniaturizeButton.isHidden = true
      }

      if let zoomButton = self.standardWindowButton(NSWindow.ButtonType.zoomButton) {
        zoomButton.isHidden = true
      }

      self.titlebarAppearsTransparent = true
      //            self.backgroundColor = .white
    } else {
      self.level = self.floatingWindowLevel
      self.styleMask = [.fullSizeContentView]
      self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
      self.representedURL = nil
    }
  }

  @objc func toggleTether(recognizer: NSClickGestureRecognizer) {
    print("toggleTether")
    guard let view = recognizer.view, let text = view as? NSTextField else {
      return
    }

    //        self.positioning = CompanionWindow.defaultActivePosition
    //        self.positioning = .untethered
    self.isDocked = !self.isDocked

    if isDocked {
      //            repositionWindow(forceUpdate: true, explicit: true)
      text.stringValue = "↗"
      self.windowManager.tether(window: self)

    } else {
      //            var newFrame = self.frame
      //            newFrame.origin = CGPoint(x: newFrame.origin.x + 10, y: newFrame.origin.y)
      //            self.setFrame(newFrame, display: true)
      self.positioning = .untethered

      recognizer.isEnabled = false
      text.stringValue = ""
      // text.stringValue = "↙"
      self.windowManager.untether(window: self)

    }
  }
  @objc func untether() {
    self.positioning = .untethered
    self.isDocked = false
    //        var newFrame = self.frame
    //        newFrame.origin = CGPoint(x: newFrame.origin.x + 10, y: newFrame.origin.y)
    //        self.setFrame(newFrame, display: true)
    self.untetherBtn?.stringValue = ""
    self.untetherBtn?.gestureRecognizers.forEach {
      self.untetherBtn?.removeGestureRecognizer($0)
    }
    // text.stringValue = "↙"
    self.windowManager.untether(window: self)
  }

  override var isMovable: Bool {
    get {
      return !isDocked
    }
    set(value) {

    }
  }

  var isSidebar: Bool {
    return self.windowManager.isSidebar(window: self)
  }

  var isAutocompletePopup: Bool {
    return WindowManager.shared.autocomplete == self
  }

  @objc func toSidebar() {
    if self.oneTimeUse {
      self.windowManager.close(window: self)
      //            self.orderOut(nil)
      //            self.close()
    } else {
      self.positioning = .sidebar
    }
  }

  @objc func backButtonClicked() {
    print("goBack")

    if let webView = self.webView {
      if webView.canGoBack {
        webView.goBack()
      } else {
        toSidebar()
      }
    }
  }

  var webView: WebView? {
    if let content = self.contentViewController as? WebViewController, let webView = content.webView {
      return webView
    }

    return nil
  }

  func repositionWindow( forceUpdate: Bool = true, explicit: Bool = false) {
    if !self.windowManager.shouldAppear(window: self, explicitlyRepositioned: explicit) {
      self.orderOut(self)
      print("Not showing window")
      return
    }

    if !isDocked {
      configureWindow(for: self.positioning)
      return
    }

    let allowlistedBundleIds = Integrations.allowlist
    guard let app = NSWorkspace.shared.frontmostApplication,
          let bundleId = app.bundleIdentifier else {
      return
    }

    if allowlistedBundleIds.contains(bundleId) {
      let targetFrame = topmostWindowFrameFor(app)
      let mouseDown = (NSEvent.pressedMouseButtons & (1 << 0)) != 0
      print("mouseDown \(mouseDown)")

      if !Defaults.shared.shouldTrackTargetWindow {
        guard shouldTrackWindow else { return }
        if !forceUpdate && !targetFrame.equalTo(priorTargetFrame) && mouseDown {
          self.animationBehavior = .utilityWindow
          self.orderOut(self)
          shouldTrackWindow = false
          NSEvent.addGlobalMonitorForEvents(matching: .leftMouseUp) { (_) -> Void in
            // prevent memory access error
            self.shouldTrackWindow = true
            self.repositionWindow(forceUpdate: true)
            //  if (self.positioning == .fullscreenInset &&
            //      self.windowManager.shouldAppear(window: self, explicitlyRepositioned: false)) {
            //    self.windowServiceProvider.takeFocus()
            //  }
          }
          return
        }
      }

      if forceUpdate || !targetFrame.equalTo(priorTargetFrame) {
        priorTargetFrame = targetFrame
        let candidates = NSScreen.screens.sorted { (a, b) -> Bool in
          let aSize = a.frame.intersection(targetFrame).size
          let bSize = b.frame.intersection(targetFrame).size
          return aSize.width * aSize.height >= bSize.width * bSize.height

        }
        let frame = self.positioning.frame(targetWindowFrame: targetFrame,
                                           screen: candidates.first!.frame)

        setOverlayFrame(frame.offsetBy(dx: 0, dy: frame.height * -1))
      }

    } else if app.isFig {
      if explicit {
        if let app = ShellBridge.shared.previousFrontmostApplication {
          guard forceUpdate else {
            return
          }
          let targetFrame = topmostWindowFrameFor(app)
          priorTargetFrame = targetFrame
          //                    let candidates = NSScreen.screens.sorted { (a, b) -> Bool in
          //                        let aSize = a.frame.intersection(targetFrame).size
          //                        let bSize = b.frame.intersection(targetFrame).size
          //                        return aSize.width * aSize.height >= bSize.width * bSize.height
          //
          //                    }
          let frame = self.positioning.frame(targetWindowFrame: targetFrame,
                                             screen: NSScreen.main!.frame)

          setOverlayFrame(frame.offsetBy(dx: 0, dy: frame.height * -1))

        }
      }
    } else {
      self.orderOut(self)
    }
  }

  func setOverlayFrame(_ frame: NSRect, makeVisible: Bool = true) {
    print("flicker: calling setOverlay")
    self.windowController?.shouldCascadeWindows = false

    let newFrame = frame // updated.offsetBy(dx: 0, dy: -1 * updated.height)
    print("flicker:", newFrame, self.frame)
    guard newFrame != self.frame else {
      print("flicker: same frame, aborting!")
      return
    }
    self.setFrame(newFrame, display: false, animate: false)

    guard self.frame.height > 0 else {
      self.orderOut(nil)
      return
    }
    //            self.setFrame(frame.offsetBy(dx: 0, dy: -1 * frame.height), display: true, animate: true)

    // adding the offset made this no longer necessary
    //            self.setFrameTopLeftPoint(frame.origin)

    //            self.orderFront(nil)

    // sometimes we might want to move the window without showing it visible
    // to make future updates less jaring.
    if makeVisible {
      self.makeKeyAndOrderFront(nil)
    }
    // This line is essential
    //            self.contentViewController?.view.frame = NSRect.init(origin: .zero, size:frame.size)

    // these don't seem to be needed?
    //            self.contentViewController?.view.setFrameSize(frame.size)
    //            self.contentViewController?.view.needsDisplay = true
    //            self.contentViewController?.view.needsLayout = true
    configureWindow(for: self.positioning)

  }

  func topmostWindowFrameFor(_ app: NSRunningApplication, includingTitleBar: Bool = false) -> NSRect {
    let appRef = AXUIElementCreateApplication(app.processIdentifier)

    var window: AnyObject?
    let result = AXUIElementCopyAttributeValue(appRef, kAXFocusedWindowAttribute as CFString, &window)

    if result == .apiDisabled {
      print("Accesibility needs to be enabled.")
      return .zero
    }

    var position: AnyObject?
    var size: AnyObject?

    guard window != nil else {
      print("Window does not exist.")
      return .zero
    }

    let windowId = PrivateWindow.getCGWindowID(fromRef: window as! AXUIElement)
    print("windowId", windowId)

    //        AXUIElementCopyAttributeValue(window as! AXUIElement, kAXTitleAttribute as CFString, &position)

    AXUIElementCopyAttributeValue(window as! AXUIElement, kAXPositionAttribute as CFString, &position)
    AXUIElementCopyAttributeValue(window as! AXUIElement, kAXSizeAttribute as CFString, &size)

    if let position = position, let size = size {
      let point = AXValueGetters.asCGPoint(value: position as! AXValue)
      let bounds = AXValueGetters.asCGSize(value: size as! AXValue)

      let titleBarHeight: CGFloat = 0// 23.0;

      //https://stackoverflow.com/a/19887161/926887
      return NSRect.init(x: point.x,
                         y: NSMaxY(NSScreen.screens[0].frame) - point.y - ((includingTitleBar) ? 0 : titleBarHeight),
                         width: bounds.width,
                         height: bounds.height - ((includingTitleBar) ? 0 : titleBarHeight))
    }
    return .zero
  }

  func overlayFrame( _ positioning: OverlayPositioning, terminalFrame: NSRect, screenBounds: NSRect) -> NSRect {
    if terminalFrame.width < 100 || terminalFrame.height < 200 {
      return .zero
    }
    let t_size = terminalFrame.size
    switch positioning {
    case .coveringTitlebar:
      return NSRect(origin: terminalFrame.origin, size: CGSize.init(width: t_size.width, height: 100))
    case .insideRightFull:
      return terminalFrame.divided(atDistance: 300, from: .maxXEdge).slice
    case .insideRightPartial:
      return terminalFrame.divided(atDistance: 300, from: .maxXEdge).slice
        .divided(atDistance: t_size.height * ( 2 / 3 ), from: .maxYEdge).slice.offsetBy(dx: 0, dy: -t_size.height / 3)
    case .atPrompt:

      let inner = terminalFrame.insetBy(dx: 30, dy: 45)
      return NSRect(x: inner.origin.x, y: inner.origin.y - inner.height, width: inner.width, height: 100)
    case .outsideRight:
      return terminalFrame.insetBy(dx: -300, dy: 0).divided(atDistance: 300, from: .maxXEdge).slice
    case .icon:
      let i_size =  CGSize(width: 50, height: 50)
      let i_padding = CGPoint(x: 12, y: -36)
      return NSRect(origin: CGPoint.init(x: terminalFrame.maxX - i_size.width - i_padding.x,
                                         y: terminalFrame.minY - i_size.height - i_padding.y), size: i_size)
    case .notification:
      let i_size =  CGSize(width: 275, height: 140)
      let i_padding = CGPoint(x: 12, y: -36)
      return NSRect(origin: CGPoint.init(x: terminalFrame.maxX - i_size.width - i_padding.x,
                                         y: terminalFrame.minY - i_size.height - i_padding.y), size: i_size)
    case .sidebar:
      return terminalFrame.divided(atDistance: 50, from: .maxXEdge).slice
    case .fullscreen:
      return .zero
    case .spotlight:
      return .zero
    case .fullscreenInset:
      return .zero
    case .hidden:
      return .zero
    case .untethered:
      return .zero
    case .fullwindow:
      return .zero
    case .popover:
      return .zero
    case .popoverCentered:
      return .zero
    case .powerbar:
      return .zero
    }

  }
}

// extension CompanionWindow : NSWindowDelegate {
//
// }

// extension NSRect {
//    func nonintersection() ->
// }

extension CompanionWindow {
  func addViewToTitleBar(_ view: NSView, at x: CGFloat, offset: CGFloat) {
    view.frame = NSRect(
      x: x,
      y: (self.contentView?.frame.height)! - offset,
      width: view.frame.width,
      height: self.heightOfTitleBar
    )
    var mask: UInt = 0
    if  x > self.frame.size.width / 2.0 {
      mask |= UInt(NSView.AutoresizingMask.minXMargin.rawValue)
    } else {
      mask |= UInt(NSView.AutoresizingMask.maxXMargin.rawValue)
    }

    view.autoresizingMask = NSView.AutoresizingMask(rawValue: mask | UInt(NSView.AutoresizingMask.minYMargin.rawValue))

    self.contentView?.superview!.addSubview(view, positioned: .above, relativeTo: nil)
  }

  var heightOfTitleBar: CGFloat {
    let outerFrame = self.contentView?.superview?.frame
    let innerFrame = self.contentView?.frame

    return outerFrame!.size.height - innerFrame!.size.height
  }

}

extension NSColor {
  public convenience init?(hex: String) {
    var cString: String = hex.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()

    if cString.hasPrefix("#") {
      cString.remove(at: cString.startIndex)
    }

    if (cString.count) != 6 {
      return nil
    }

    var rgbValue: UInt64 = 0
    Scanner(string: cString).scanHexInt64(&rgbValue)

    self.init(
      red: CGFloat((rgbValue & 0xFF0000) >> 16) / 255.0,
      green: CGFloat((rgbValue & 0x00FF00) >> 8) / 255.0,
      blue: CGFloat(rgbValue & 0x0000FF) / 255.0,
      alpha: CGFloat(1.0)
    )
  }
}

class PointableButton: NSButton {
  override func resetCursorRects() {
    self.addCursorRect(self.bounds, cursor: .pointingHand)
  }
}

extension CompanionWindow {

  // https://linear.app/fig/issue/ENG-145/multitouch-swiping-between-workspaces-loses-window
  // Imperfect workaround for when floating window disappears after multitouch swipe in exposé

  func updateCollectionBehavior() {
    // in exposé
    if self.isOnActiveSpace && !self.occlusionState.contains(.visible) {
      self.collectionBehavior = [ .managed ]
    } else {
      self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary ]
    }
  }

  func windowDidChangeOcclusionState(_ notification: Notification) {
    self.updateCollectionBehavior()
  }

  @objc func spaceChanged() {
    self.updateCollectionBehavior()
  }
}
