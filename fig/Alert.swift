//
//  Alert.swift
//  fig
//
//  Created by Matt Schrage on 3/2/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Alert {
  static let appIcon = NSImage(imageLiteralResourceName: NSImage.applicationIconName)
  static let lockWithAppIcon = NSImage(imageLiteralResourceName: "NSSecurity").overlayAppIcon()
  static func lockWith3rdPartyIcon(for bundleId: String) -> NSImage? {
    guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleId) else {
         return nil
       }
       
       let icon = NSImage(imageLiteralResourceName: "NSSecurity")

       let app = NSWorkspace.shared.icon(forFile: url.path)
       
       return icon.overlayImage(app)
    
  }
  
  static func show(title: String,
                   message: String,
                   okText: String = "OK",
                   icon: NSImage = lockWithAppIcon,
                   hasSecondaryOption: Bool = false,
                   secondaryOptionTitle: String? = nil) -> Bool {
    
    let alert = NSAlert()
    alert.icon = icon
    alert.icon.size = NSSize(width: 32, height: 32)
    alert.messageText = title
    alert.informativeText = message
    alert.alertStyle = .warning
    let button = alert.addButton(withTitle: okText)
    button.highlight(true)
    if (hasSecondaryOption) {
        alert.addButton(withTitle: secondaryOptionTitle ?? "Not now")
    }
    return alert.runModal() == .alertFirstButtonReturn
  }
  
  
}
