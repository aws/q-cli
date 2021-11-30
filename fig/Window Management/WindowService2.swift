//
//  WindowService2.swift
//  fig
//
//  Created by Matt Schrage on 11/29/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

protocol WindowService2 {

    func topmostWhitelistedWindow() -> ExternalWindow?
    func topmostWindow(for app: NSRunningApplication) -> ExternalWindow?
    func previousFrontmostApplication() -> NSRunningApplication?
    func currentApplicationIsWhitelisted() -> Bool
    func allWindows(onScreen: Bool) -> [ExternalWindow]
    func allWhitelistedWindows(onScreen: Bool) -> [ExternalWindow]
    func previousWhitelistedWindow() -> ExternalWindow?
    func bringToFront(window: ExternalWindow)
    func takeFocus()
    func returnFocus()
    
    var isActivating: Bool { get }
    var isDeactivating: Bool { get }
  
    func lastTabId(for: CGWindowID) -> String
    

}
