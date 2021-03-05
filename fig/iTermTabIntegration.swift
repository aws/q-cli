//
//  iTermTabIntegration.swift
//  fig
//
//  Created by Matt Schrage on 1/20/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class iTermTabIntegration: IntegrationProvider {
  static func install(withRestart: Bool, inBackground: Bool, completion: (() -> Void)?) {
        guard !inBackground else { return }
        NSApp.appDelegate.iTermSetup()
    }
    
    static func promptToInstall(completion: (() -> Void)?) {
        NSApp.appDelegate.iTermSetup()
    }
  
    static var keyHandler: Any? = nil
    static let path = "\(NSHomeDirectory())/Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.py"
    static func listenForHotKey() {
        guard !iTermTabIntegration.isInstalled else { return }
        guard !iTermTabIntegration.hasBeenPromptedBefore else { return }

        if let handler = Self.keyHandler {
          NSEvent.removeMonitor(handler)
        }
        
        Self.keyHandler = NSEvent.addGlobalMonitorForEvents(matching: [ .keyUp], handler: { (event) in
          guard Defaults.loggedIn else { return }
          guard event.keyCode == Keycode.t && event.modifierFlags.contains(.command) else { return }
          guard NSWorkspace.shared.frontmostApplication?.bundleIdentifier == "com.googlecode.iterm2" else { return }
          guard !iTermTabIntegration.isInstalled else { return }
          guard !iTermTabIntegration.hasBeenPromptedBefore else { return }
            
            iTermTabIntegration.promptToInstall()
        })
    }
    
    static var isInstalled: Bool {
        return FileManager.default.fileExists(atPath: iTermTabIntegration.path)
    }
    static func promptToInstall() {
        iTermTabIntegration.hasBeenPromptedBefore = true
      
        DispatchQueue.global(qos: .background).async {
          TelemetryProvider.track(event: .iTermSetupPrompted, with: [:])
        }
      
        let install = (NSApp.delegate as! AppDelegate).dialogOKCancel(question: "Using tabs in iTerm?", text: "Fig can't distinguish between iTerm tabs by default and requires the use of a plugin.\n", prompt: "Setup", icon: NSImage(imageLiteralResourceName: NSImage.applicationIconName))
        
        if (install) {
            (NSApp.delegate as! AppDelegate).iTermSetup()
        }
        
        if let handler = Self.keyHandler {
          NSEvent.removeMonitor(handler)
        }
    }
    
    static var hasBeenPromptedBefore: Bool {
        get {
            return UserDefaults.standard.bool(forKey: "promptedToAddItermTabIntegration")
        }
        
        set(flag) {
            UserDefaults.standard.set(flag, forKey: "promptedToAddItermTabIntegration")
            UserDefaults.standard.synchronize()
        }
    }
}
