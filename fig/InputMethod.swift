//
//  InputMethod.swift
//  fig
//
//  Created by Matt Schrage on 8/30/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa
//defaults read ~/Library/Preferences/com.apple.HIToolbox.plist AppleSelectedInputSources
class InputMethod {
    static let bundleId = "io.fig.inputmethod.cursor"
    static let bundle = Bundle(path: Bundle.main.path(forResource: "FigInputMethod", ofType: "app")!)!
    static let inputMethodDirectory = URL(fileURLWithPath: "\(NSHomeDirectory())/Library/Input Methods/")
    //defaults read ~/Library/Preferences/com.apple.HIToolbox.plist
    //https://developer.apple.com/library/archive/qa/qa1810/_index.html
    static var `default`: TISInputSource? {
        get {
            let properties = [
                kTISPropertyInputSourceID as String : InputMethod.bundle.bundleIdentifier,
                kTISPropertyInputSourceType as String : kTISTypeCharacterPalette as String
            ] as CFDictionary

            
            guard let sources = TISCreateInputSourceList(properties, true)?.takeUnretainedValue() as? [TISInputSource] else {
                return nil
            }
            
            guard let inputMethod = sources[safe: 0] else {
                return nil
            }
            
            return inputMethod
        
        }
    }
    @discardableResult static func toggleSource(with id: String = InputMethod.bundle.bundleIdentifier ?? "", on: Bool) -> Bool {
//        kTISCategoryPaletteInputSource
        
        
        if on {
            InputMethod.select()
            InputMethod.enable()

            // return TISEnableInputSource(inputMethod) != noErr
        } else {
            InputMethod.deselect()
            InputMethod.disable()
            //return TISDisableInputSource(inputMethod) != noErr
        }
        
        return true

    }
    
    static func select() {
        guard let inputMethod = InputMethod.default else {
            return
        }
        
        TISSelectInputSource(inputMethod)
    }
    
    static func deselect() {
        guard let inputMethod = InputMethod.default else {
            return
        }
        
        TISDeselectInputSource(inputMethod)
    }
    
    static func enable() {
        guard let inputMethod = InputMethod.default else {
            return
        }
        
        TISEnableInputSource(inputMethod)
    }
    
    static func disable() {
        guard let inputMethod = InputMethod.default else {
            return
        }
        
        TISDisableInputSource(inputMethod)
    }
    
    // Note: apps that rely on the input method to locate the cursor position must be restarted before the input method will work
    static func install() -> Bool {
        let url = bundle.bundleURL
        let name = url.lastPathComponent
        let targetURL = inputMethodDirectory.appendingPathComponent(name)

        do {
            try FileManager.default.createSymbolicLink(at: targetURL, withDestinationURL: url)
        } catch {
            print("Could not create symlink!")
//            return false
        }
        
        let err = TISRegisterInputSource(targetURL as CFURL)
        guard err != paramErr else {return false}
        
        return true
    }
    
    static func uninstall() {
        let url = bundle.bundleURL
        let name = url.lastPathComponent
        let targetURL = inputMethodDirectory.appendingPathComponent(name)
        
        toggleSource(on: false)
        try? FileManager.default.removeItem(at: targetURL)
        
        if let runningInputMethod = NSRunningApplication.forBundleId(bundle.bundleIdentifier ?? "") {
            print("Terminating input method \(bundle.bundleIdentifier ?? "") (\(runningInputMethod.processIdentifier))...")
            runningInputMethod.terminate()
        }
        
    }
    
    static func keypressTrigger(_ event: CGEvent, _ window: ExternalWindow) -> EventTapAction {
        if [.keyDown, .keyUp ].contains(event.type) {
            requestCursorUpdate(for: window.bundleId)
        }

        return .ignore
    }
    
    static func requestCursorUpdate(for bundleIdentifier: String?) {
        guard let bundleIdentifier = bundleIdentifier else {
            return
        }
        let center: DistributedNotificationCenter = DistributedNotificationCenter.default()
        center.postNotificationName(NSNotification.Name("io.fig.keypress"), object: nil, userInfo: ["bundleIdentifier" : bundleIdentifier], deliverImmediately: true)
        print("Sending distributed notification!")
    }
    
    static func requestVersion() {
        let center: DistributedNotificationCenter = DistributedNotificationCenter.default()
        center.postNotificationName(NSNotification.Name("io.fig.report-ime-version"), object: nil, userInfo: nil, deliverImmediately: true)
    }
}
