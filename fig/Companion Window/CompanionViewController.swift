//
//  CompanionWindow.swift
//  fig
//
//  Created by Matt Schrage on 4/18/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa
import WebKit

class CompanionViewController: NSViewController {
  var webView: WKWebView = WKWebView(frame: .zero)
  let cornerRadius = 15

  override func loadView() {
    let effect = NSVisualEffectView(frame: .zero)
    effect.blendingMode = .behindWindow
    effect.state = .active
    effect.material = .mediumLight
    effect.maskImage = maskImage(cornerRadius: 15)
    self.view = effect
  }

  override func viewDidAppear() {
    view.window?.delegate = self
    webView.navigationDelegate = self
    webView.frame = self.view.bounds
    webView.setValue(false, forKey: "drawsBackground")
    self.view.addSubview(webView)

    self.view.window?.alphaValue = 0.9
  }

  override func viewDidLoad() {
    //        self.webView = WKWebView(frame:.zero, configuration: WebBridge(eventDelegate: self))
  }

}

extension CompanionViewController: WebBridgeEventDelegate {
  func requestExecuteCLICommand(script: String) {

  }

  func requestInsertCLICommand(script: String) {

  }

  func requestNextSection() {

  }

  func requestPreviousSection() {

  }

  func startTutorial(identifier: String) {

  }

}

extension CompanionViewController: NSWindowDelegate {
  func windowDidResize(_ notification: Notification) {
    self.webView.frame = self.view.bounds
  }
}

extension CompanionViewController: WKNavigationDelegate {

}

fileprivate extension CompanionViewController {
  //https://github.com/marcomasser/OverlayTest/blob/master/Overlay%20Test/AppDelegate.swift
  private func maskImage(cornerRadius: CGFloat) -> NSImage {
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
