//
//  InputMethod.swift
//  fig
//
//  Created by Matt Schrage on 8/30/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa
// defaults read ~/Library/Preferences/com.apple.HIToolbox.plist AppleSelectedInputSources
// plutil -remove 'AppleEnabledInputSources.5'  ~/Library/Preferences/com.apple.HIToolbox.plist
// https://apple.stackexchange.com/questions/127246/mavericks-how-to-add-input-source-via-plists-defaults

// killall cfprefsd
/*
 defaults write com.apple.HIToolbox AppleEnabledInputSources
 -array-add '<dict><key>Bundle ID</key><string>io.fig.inputmethod.cursor</string>
 <key>InputSourceKind</key><string>Non Keyboard Input Method</string></dict>'
 */

class InputMethod {
  static let inputMethodDirectory = URL(fileURLWithPath: "\(NSHomeDirectory())/Library/Input Methods/")
  static let statusDidChange = Notification.Name("inputMethodStatusDidChange")
  static let supportURL = URL(string: "https://fig.io/docs/support/enabling-input-method")!
  @objc class func openSupportPage() {
    NSWorkspace.shared.open(supportURL)
  }

  static func getCursorRect() -> NSRect? {
    guard let raw = try? String(contentsOfFile: NSHomeDirectory()+"/.fig/tools/cursor") else {
      return nil
    }

    let tokens = raw.split(separator: ",")
    guard tokens.count == 4,
          // swiftlint:disable identifier_name
          let x = Double(tokens[0]),
          // swiftlint:disable identifier_name
          let y = Double(tokens[1]) else {
      return nil
    }
    InputMethod.log("cursor=\(x),\(y)")
    return NSRect(x: x, y: y, width: 10, height: 10).offsetBy(dx: 0, dy: 10)
  }

  static let `default` = InputMethod(
    bundlePath: Bundle.main.bundleURL.appendingPathComponent("Contents/Helpers/FigInputMethod.app").path
  )

  let bundle: Bundle
  let originalBundlePath: String
  var name: String {
    let url = self.bundle.bundleURL
    return url.lastPathComponent
  }

  var kvo: NSKeyValueObservation?

  var timer: Timer?
  var status: InstallationStatus {
    didSet {
      if oldValue != status {
        InputMethod.log("statusDidChange \(status)")
        NotificationCenter.default.post(name: InputMethod.statusDidChange, object: nil)
      }

      if status == .installed {
        timer?.invalidate()
        timer = nil
      }
    }
  }

  fileprivate let maxAttempts = 10
  fileprivate var remainingAttempts = 0
  fileprivate func startPollingForActivation() {
    guard Settings.shared.getValue(forKey: Settings.inputMethodShouldPollForActivation) as? Bool ?? true else {
      return
    }
    guard self.timer == nil else {
      return
    }

    self.remainingAttempts = maxAttempts
    self.timer = Timer.scheduledTimer(withTimeInterval: 3, repeats: true) { timer in
      self.remainingAttempts -= 1
      self.select()

      self.verifyAndUpdateInstallationStatus()
      InputMethod.log("ping!!!! (remaining attempts = \(self.remainingAttempts) - \(self.status)")

      if self.remainingAttempts == 0 && self.status != .installed {
        timer.invalidate()
        self.timer = nil

        let message = "This is required to locate the cursor in certain terminal emulators.\n\n" +
                      "Restart Fig and try again."
        let openSupportPage = Alert.show(title: "Could not install InputMethod",
                                         message: message,
                                         okText: "Learn more",
                                         hasSecondaryOption: true)

        if openSupportPage {
          InputMethod.openSupportPage()
        }
      }
    }
  }

  // defaults read ~/Library/Preferences/com.apple.HIToolbox.plist
  //https://developer.apple.com/library/archive/qa/qa1810/_index.html
  var source: TISInputSource? {
    let properties = [
      kTISPropertyInputSourceID as String: self.bundle.bundleIdentifier
    ] as CFDictionary

    // https://stackoverflow.com/questions/34120142/swift-cfarray-get-values-as-utf-strings/34121525
    // Use takeRetainedValue rather than takeUnretainedValue
    guard let rawSourceList = TISCreateInputSourceList(properties, true)?.takeRetainedValue() else {
      InputMethod.log("TISCreateInputSourceList failed.")
      return nil
    }

    let sourcesArray = rawSourceList as NSArray
    guard let sources = sourcesArray as? [TISInputSource] else {
      InputMethod.log("Could not list Input Sources matching properties")
      return nil
    }

    InputMethod.log("\(sources.count) input method(s) were found")
    guard let inputMethod = sources[safe: 0] else {
      InputMethod.log("No Input Sources matching properties were found")
      return nil
    }

    return inputMethod
  }

  init(bundlePath: String) {
    self.bundle = Bundle(path: bundlePath)!
    self.originalBundlePath = bundlePath
    self.status = InstallationStatus(
      data: UserDefaults.standard.data(forKey: self.bundle.bundleIdentifier! + ".integration")
    ) ?? .unattempted

    let center = DistributedNotificationCenter.default()

    let enabledInputSourcesChangedNotification = NSNotification.Name(
      kTISNotifyEnabledKeyboardInputSourcesChanged as String
    )
    center.addObserver(forName: enabledInputSourcesChangedNotification, object: nil, queue: nil) { _ in
      InputMethod.log("enabled Input Sources changed")
      self.verifyAndUpdateInstallationStatus()
    }

    let selectedInputSourcesChangedNotification = NSNotification.Name(
      kTISNotifySelectedKeyboardInputSourceChanged as String
    )
    center.addObserver(forName: selectedInputSourcesChangedNotification, object: nil, queue: nil) { _ in
      InputMethod.log("selected Input Sources changed")
      self.verifyAndUpdateInstallationStatus()
    }

    center.addObserver(self,
                       selector: #selector(selectedKeyboardInputSourceChanged),
                       name: selectedInputSourcesChangedNotification,
                       object: nil,
                       suspensionBehavior: .deliverImmediately)

    center.addObserver(self,
                       selector: #selector(enabledKeyboardInputSourcesChanged),
                       name: enabledInputSourcesChangedNotification,
                       object: nil,
                       suspensionBehavior: .deliverImmediately)

    verifyAndUpdateInstallationStatus()

  }

  @objc func selectedKeyboardInputSourceChanged() {
    InputMethod.log("selected Input Sources changed")
    self.verifyAndUpdateInstallationStatus()
  }

  @objc func enabledKeyboardInputSourcesChanged() {
    InputMethod.log("enabled Input Sources changed")
    self.verifyAndUpdateInstallationStatus()
  }

  @objc func updateStatus() {
    NotificationCenter.default.post(name: InputMethod.statusDidChange, object: nil)
  }

  func terminate() {
    if let runningInputMethod = NSRunningApplication.forBundleId(bundle.bundleIdentifier ?? "") {
      InputMethod.log(
        "Terminating input method \(bundle.bundleIdentifier ?? "") (\(runningInputMethod.processIdentifier))...")
      runningInputMethod.terminate()
    }

  }

  func uninstall() {

    InputMethod.log("Uninstalling...")

    let targetURL = InputMethod.inputMethodDirectory.appendingPathComponent(self.name)

    self.deselect()
    self.disable()

    try? FileManager.default.removeItem(at: targetURL)
    try? FileManager.default.removeItem(atPath: NSHomeDirectory()+"/.fig/tools/cursor")

    self.terminate()

    self.updateStatus()

    // If we attempt to reinstall the input method before restarting,
    // we'll recieve OSStatus -50 when trying to select the InputSource
    InputMethod.log("After uninstalling the input method, the macOS app" +
                    "must be restarted before it can be installed again")

  }

  var isInstalled: Bool {
    return self.verifyInstallation() == .installed
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
    center.postNotificationName(
      NSNotification.Name("io.fig.keypress"),
      object: nil,
      userInfo: ["bundleIdentifier": bundleIdentifier],
      deliverImmediately: true
    )
    print("Sending distributed notification!")
  }

  static func requestVersion() {
    let center: DistributedNotificationCenter = DistributedNotificationCenter.default()
    center.postNotificationName(
      NSNotification.Name("io.fig.report-ime-version"),
      object: nil,
      userInfo: nil,
      deliverImmediately: true
    )
  }
}

extension InputMethod: IntegrationProvider {
  var id: String {
    return "input-method"
  }

  func verifyInstallation() -> InstallationStatus {

    let targetURL = InputMethod.inputMethodDirectory.appendingPathComponent(name)

    guard let destination = try? FileManager.default.destinationOfSymbolicLink(atPath: targetURL.path),
          destination == self.originalBundlePath else {
      return .failed(error: "input method is not installed in \(InputMethod.inputMethodDirectory.path)")
    }

    guard NSRunningApplication.forBundleId(self.bundle.bundleIdentifier ?? "") != nil else {
      return .failed(error: "input method is not running.")
    }

    let inputMethodDefaults = UserDefaults(suiteName: "com.apple.HIToolbox")
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
      let error = NSError(domain: NSOSStatusErrorDomain, code: Int(err), userInfo: nil)
      return .failed(error: error.localizedDescription)
    }

    self.enable()
    self.select()

    // should we launch the application manually?
    if let bundleId = self.bundle.bundleIdentifier {
      let inputSource = Restarter(with: bundleId)
      inputSource.restart(launchingIfInactive: true) {
        self.select()
      }
    }

    self.startPollingForActivation()
    return .pending(event: .inputMethodActivation)
  }

  func verifyAndUpdateInstallationStatus() {
    let status = self.verifyInstallation()
    if self.status != status {
      self.status = status
    }
  }

  // Note: apps that rely on the input method to locate the cursor position must be restarted before the input method
  // will work
  func install() -> InstallationStatus {
    self.status = self._install()
    return self.status
  }
}

extension InputMethod {
  @discardableResult func register() -> String {
    let url = URL(fileURLWithPath: self.originalBundlePath)

    let targetURL = InputMethod.inputMethodDirectory.appendingPathComponent(name)

    // Remove previous symlink
    try? FileManager.default.removeItem(at: targetURL)

    try? FileManager.default.createSymbolicLink(at: targetURL, withDestinationURL: url)

    let err = TISRegisterInputSource(targetURL as CFURL)
    guard err != paramErr else {
      let error = NSError(domain: NSOSStatusErrorDomain, code: Int(err), userInfo: nil)
      return error.localizedDescription
    }

    if let bundleId = self.bundle.bundleIdentifier {
      let inputSource = Restarter(with: bundleId)
      inputSource.restart(launchingIfInactive: true)
    }

    return "Registered input method!"
  }

  @discardableResult func select() -> String {
    guard let inputMethod = self.source else {
      return "Could not load input source"
    }

    guard !inputMethod.isSelected else {
      let message = "Input method is already selected!"
      InputMethod.log(message)
      return message
    }

    let status = TISSelectInputSource(inputMethod)

    if status != noErr {
      let err = NSError(domain: NSOSStatusErrorDomain, code: Int(status), userInfo: nil)
      let message = "An error occured when selecting input method: \(err.localizedDescription)"
      InputMethod.log(message)

      if !inputMethod.isEnabled {
        InputMethod.log("Input method must be enabled before it can be selected!")
      }

      if !inputMethod.isSelectable {
        InputMethod.log("Input method must be selectable in order to be selected!")
      }

      return message
    }

    return "Selected input method!"
  }

  @discardableResult func deselect() -> String {
    guard let inputMethod = self.source else {
      return "Could not load input source"
    }

    let status = TISDeselectInputSource(inputMethod)

    if status != noErr {
      let err = NSError(domain: NSOSStatusErrorDomain, code: Int(status), userInfo: nil)
      let message = "An error occured when deselecting input method: \(err.localizedDescription)"
      InputMethod.log(message)
      return message
    }

    return "Deselected input method!"
  }

  // On macOS Monterrey, this opens System Preferences > Input Sources and prompts user!
  @discardableResult func enable() -> String {
    guard let inputMethod = self.source else {
      return "Could not load input source"
    }

    let status = TISEnableInputSource(inputMethod)

    if status != noErr {
      let err = NSError(domain: NSOSStatusErrorDomain, code: Int(status), userInfo: nil)
      let message = "An error occured when enabling input method: \(err.localizedDescription)"
      InputMethod.log(message)
      return message
    }

    return "Enabled input method!"
  }

  @discardableResult func disable() -> String {
    guard let inputMethod = self.source else {
      return "Could not load input source"
    }

    let status = TISDisableInputSource(inputMethod)

    if status != noErr {
      let err = NSError(domain: NSOSStatusErrorDomain, code: Int(status), userInfo: nil)
      let message = "An error occured when disabling input method: \(err.localizedDescription)"
      InputMethod.log(message)
      return message
    }

    return "Disabled input method!"
  }
}

extension InputMethod {
  static func log(_ message: String) {
    Logger.log(message: message, subsystem: .inputMethod)
  }
}

extension TISInputSource {
  func getProperty(_ key: CFString) -> AnyObject? {
      guard let cfType = TISGetInputSourceProperty(self, key) else { return nil }
      return Unmanaged<AnyObject>.fromOpaque(cfType).takeUnretainedValue()
  }

  var isSelectable: Bool {
    return getProperty(kTISPropertyInputSourceIsSelectCapable) as? Bool ?? false
  }

  var isEnablable: Bool {
    return getProperty(kTISPropertyInputSourceIsEnableCapable) as? Bool ?? false
  }

  var isEnabled: Bool {
    return getProperty(kTISPropertyInputSourceIsEnabled) as? Bool ?? false
  }

  var isSelected: Bool {
    return getProperty(kTISPropertyInputSourceIsSelected) as? Bool ?? false
  }

}
