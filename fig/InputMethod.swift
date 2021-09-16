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
    static let inputMethodDirectory = URL(fileURLWithPath: "\(NSHomeDirectory())/Library/Input Methods/")
    static let statusDidChange = Notification.Name("inputMethodStatusDidChange")
    
    static func getCursorRect() -> NSRect? {
        guard let raw = try? String(contentsOfFile: NSHomeDirectory()+"/.fig/tools/cursor") else {
            return nil
        }
        
        let tokens = raw.split(separator: ",")
        guard tokens.count == 4,
              let x = Double(tokens[0]),
              let y = Double(tokens[1])/*,
              let width = Double(tokens[2]),
              let height = Double(tokens[1])*/ else {
            return nil
        }
        print("ime:",x, y)
        return NSRect(x: x, y: y, width: 10, height: 10).offsetBy(dx: 0, dy: 10)
      }

    static let `default` = InputMethod(bundlePath: Bundle.main.bundleURL.appendingPathComponent("Contents/Helpers/FigInputMethod.app").path)//Bundle.main.url(forAuxiliaryExecutable: "FigInputMethod.app")!.path)//Bundle.main.path(forResource: "FigInputMethod", ofType: "app")!)

    let bundle: Bundle
    let originalBundlePath: String
    var name: String {
        let url = self.bundle.bundleURL
        return url.lastPathComponent
    }
    var timer: Timer?
    var status: InstallationStatus {
        didSet {
            if oldValue != status {
                print("InputMethod: statusDidChange \(status)")
                NotificationCenter.default.post(name: InputMethod.statusDidChange, object: nil)
            }
            
            if status == .installed {
                timer?.invalidate()
                timer = nil
            }
        }
    }
    
    fileprivate func startPollingForActivation() {
        
        guard self.timer == nil else {
            return
        }
        
        self.timer = Timer.scheduledTimer(withTimeInterval: 10, repeats: true) { _ in

            self.enable()
            self.select()
            
            self.status = self._verifyInstallation()
            print("InputMethod: ping!!!!", self.status)
        }
        
    }

    //defaults read ~/Library/Preferences/com.apple.HIToolbox.plist
    //https://developer.apple.com/library/archive/qa/qa1810/_index.html
    var source: TISInputSource? {
        get {
            let properties = [
                kTISPropertyInputSourceID as String : self.bundle.bundleIdentifier,
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
    
    init(bundlePath: String) {
        self.bundle = Bundle(path: bundlePath)!
        self.originalBundlePath = bundlePath
        self.status = InstallationStatus(data: UserDefaults.standard.data(forKey: self.bundle.bundleIdentifier! + ".integration")) ?? .unattempted
        let center = DistributedNotificationCenter.default()
        center.addObserver(self,
                           selector: #selector(KeyboardLayout.keyboardLayoutDidChange),
                           name: NSNotification.Name(rawValue: NSNotification.Name.RawValue(kTISNotifySelectedKeyboardInputSourceChanged as NSString)),
                           object: nil)
        
        self.status = verifyInstallation()

    }
    
    @objc func updateStatus() {
        NotificationCenter.default.post(name: InputMethod.statusDidChange, object: nil)
    }
    
    @discardableResult func toggleSource(on: Bool) -> Bool {
//        kTISCategoryPaletteInputSource
        
        
        if on {
            self.select()
            self.enable()

            // return TISEnableInputSource(inputMethod) != noErr
        } else {
            self.deselect()
            self.disable()
            //return TISDisableInputSource(inputMethod) != noErr
        }
        
        return true

    }
    
    func select() {
        guard let inputMethod = self.source else {
            return
        }
        
        TISSelectInputSource(inputMethod)
    }
    
    func deselect() {
        guard let inputMethod = self.source else {
            return
        }
        
        TISDeselectInputSource(inputMethod)
    }
    
    func enable() {
        guard let inputMethod = self.source else {
            return
        }
        
        TISEnableInputSource(inputMethod)
    }
    
    func disable() {
        guard let inputMethod = self.source else {
            return
        }
        
        TISDisableInputSource(inputMethod)
    }
    
    
    func uninstall() {
        let targetURL = InputMethod.inputMethodDirectory.appendingPathComponent(self.name)
        
        self.deselect()
        self.disable()

        try? FileManager.default.removeItem(at: targetURL)
        try? FileManager.default.removeItem(atPath: NSHomeDirectory()+"/.fig/tools/cursor")
        
        if let runningInputMethod = NSRunningApplication.forBundleId(bundle.bundleIdentifier ?? "") {
            print("Terminating input method \(bundle.bundleIdentifier ?? "") (\(runningInputMethod.processIdentifier))...")
            runningInputMethod.terminate()
        }
        
        self.updateStatus()

        
    }
    
    var isInstalled: Bool {
        get {
            return self.verifyInstallation() == .installed
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

extension InputMethod: IntegrationProvider {
    
    fileprivate func _verifyInstallation() -> InstallationStatus {
        
        let targetURL = InputMethod.inputMethodDirectory.appendingPathComponent(name)

        guard let destination = try? FileManager.default.destinationOfSymbolicLink(atPath: targetURL.path),
              destination == self.originalBundlePath else {
            return .failed(error: "input method is not installed in \(InputMethod.inputMethodDirectory.path)")
        }
        
        guard NSRunningApplication.forBundleId(self.bundle.bundleIdentifier ?? "") != nil else {
            return .failed(error: "input method is not running.")
        }
        
        let inputMethodDefaults = UserDefaults(suiteName: "com.apple.HIToolbox")
        guard let selectedSources = inputMethodDefaults?.array(forKey: "AppleSelectedInputSources") else {
            return .failed(error: "Could not read the list of selected input sources")
        }
        
        guard selectedSources.contains(where: { item in
            let object = item as AnyObject
            if let bundleId = object["Bundle ID"] as? String {
                return bundleId == self.bundle.bundleIdentifier
            }
            return false
        }) else {
            return .failed(error: "Input source is not selected ")
        }
        
        let enabledSources = inputMethodDefaults?.array(forKey: "AppleEnabledInputSources") ?? []

        guard enabledSources.contains(where: { item in
            let object = item as AnyObject
            if let bundleId = object["Bundle ID"] as? String {
                return bundleId == self.bundle.bundleIdentifier
            }
            return false
        }) else {
            return .failed(error: "Input source is not enabled ")
        }
        
        return .installed
    }
    
    fileprivate func _install() -> InstallationStatus {
        let url = URL(fileURLWithPath: self.originalBundlePath)
        let name = url.lastPathComponent
        let targetURL = InputMethod.inputMethodDirectory.appendingPathComponent(name)

        // Remove previous symlink
        try? FileManager.default.removeItem(at: targetURL)
        
        try? FileManager.default.createSymbolicLink(at: targetURL, withDestinationURL: url)

        guard let destination = try? FileManager.default.destinationOfSymbolicLink(atPath: targetURL.path),
              destination == self.originalBundlePath else {
            return .failed(error: "input method is not installed in \(InputMethod.inputMethodDirectory.path)")
        }
        
        let err = TISRegisterInputSource(targetURL as CFURL)
        guard err != paramErr else {
            return .failed(error: err.description)
        }
        
        self.enable()
        self.select()
        
        // should we launch the application manually?
        if let bundleId = self.bundle.bundleIdentifier {
            let inputSource = Restarter(with: bundleId)
            inputSource.restart(launchingIfInactive: true)
        }
        
        self.startPollingForActivation()
        return .pending(event: .inputMethodActivation)
    }
    
    func verifyInstallation() -> InstallationStatus {
        self.status = self._verifyInstallation()
        return self.status
    }
    // Note: apps that rely on the input method to locate the cursor position must be restarted before the input method will work
    func install() -> InstallationStatus {
        self.status = self._install()
        return self.status
    }
}
