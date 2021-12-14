//
//  AppDelegate.swift
//  InputMethod
//
//  Created by Matt Schrage on 9/1/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import InputMethodKit

// only need one candidates window for the entire input method
// because only one such window should be visible at a time
var candidatesWindow: IMKCandidates = IMKCandidates()

@main
class AppDelegate: NSObject, NSApplicationDelegate {

  var window: NSWindow!

  func applicationDidFinishLaunching(_ aNotification: Notification) {
    let version: String =
      Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? ""
    let buildNumber: String =
      Bundle.main.object(forInfoDictionaryKey: "CFBundleVersion") as? String ?? ""
    print("version \(version) (\(buildNumber))")
    Logger.appendToLog("hello")

    // no matter what Info.plist and goftam.entitlements say, the connection name
    // requested from the sandbox seems to be $(PRODUCT_BUNDLE_IDENTIFIER)_Connection,
    // so Info.plist and goftam.entitlements have been set to comply with this choice
    let server = IMKServer(name: Bundle.main.infoDictionary?["InputMethodConnectionName"] as? String,
                           bundleIdentifier: Bundle.main.bundleIdentifier)

    // scrolling to the bottom of the scrolling panel puts selection numbers out of alignment
    candidatesWindow = IMKCandidates(server: server,
                                     panelType: kIMKSingleColumnScrollingCandidatePanel)
    // panelType: kIMKSingleRowSteppingCandidatePanel)

    // as of 10.15.3, default candidates window key event handling is buggy
    // (number selector keys don't work). workaround involves bypassing default window handling.
    candidatesWindow.setAttributes([IMKCandidatesSendServerKeyEventFirst: NSNumber(booleanLiteral: true)])

    let center = DistributedNotificationCenter.default()
    let reportVersionNotification = NSNotification.Name("io.fig.report-ime-version")

    center.addObserver(forName: reportVersionNotification, object: nil, queue: nil) { _ in
      Logger.appendToLog("Version: \(version) (\(buildNumber))")
    }
  }

  func applicationWillTerminate(_ aNotification: Notification) {
    // Insert code here to tear down your application
  }

}
