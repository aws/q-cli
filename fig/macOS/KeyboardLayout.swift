//
//  KeyboardLayout.swift
//  fig
//
//  Created by Matt Schrage on 9/3/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Carbon
import Cocoa
class KeyboardLayout: NSObject {
  static let shared = KeyboardLayout()

  //https://stackoverflow.com/questions/1918841/how-to-convert-ascii-character-to-cgkeycode
  //https://stackoverflow.com/a/35138823
  static func keyName(scanCode: UInt16) -> String? {
    let maxNameLength = 4
    var nameBuffer = [UniChar](repeating: 0, count: maxNameLength)
    var nameLength = 0

    let modifierKeys = UInt32(alphaLock >> 8) & 0xFF // Caps Lock
    var deadKeys: UInt32 = 0
    let keyboardType = UInt32(LMGetKbdType())

    let source = TISCopyCurrentKeyboardLayoutInputSource().takeRetainedValue()
    guard let ptr = TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData) else {
      NSLog("Could not get keyboard layout data")
      return nil
    }
    let layoutData = Unmanaged<CFData>.fromOpaque(ptr).takeUnretainedValue() as Data
    let osStatus = layoutData.withUnsafeBytes {
      UCKeyTranslate($0.bindMemory(to: UCKeyboardLayout.self).baseAddress, scanCode, UInt16(kUCKeyActionDown),
                     modifierKeys, keyboardType, UInt32(kUCKeyTranslateNoDeadKeysMask),
                     &deadKeys, maxNameLength, &nameLength, &nameBuffer)
    }
    guard osStatus == noErr else {
      NSLog("Code: 0x%04X  Status: %+i", scanCode, osStatus)
      return nil
    }

    return String(utf16CodeUnits: nameBuffer, count: nameLength).lowercased()
  }

  static func humanReadableKeyName(_ event: CGEvent) -> String? {
    guard let event = NSEvent(cgEvent: event) else {
      return nil
    }

    var modifiers: [String] = []

    if event.modifierFlags.contains(.command) {
      modifiers.append("command")
    }
    if event.modifierFlags.contains(.control) {
      modifiers.append("control")
    }
    if event.modifierFlags.contains(.option) {
      modifiers.append("option")
    }
    if event.modifierFlags.contains(.shift) {
      modifiers.append("shift")
    }

    let characters: String = {
      switch Keycode(rawValue: event.keyCode) ?? Keycode.zero {
      case Keycode.upArrow:
        return "up"
      case Keycode.downArrow:
        return "down"
      case Keycode.leftArrow:
        return "left"
      case Keycode.rightArrow:
        return "right"
      case Keycode.delete:
        return "delete"
      case Keycode.tab:
        return "tab"
      case Keycode.escape:
        return "esc"
      case Keycode.returnKey:
        return "enter"
      default:
        return event.charactersIgnoringModifiers ?? ""
      }
    }()

    modifiers.append(characters.lowercased())

    return modifiers.joined(separator: "+")
  }

  func keyCode(for ascii: String) -> CGKeyCode? {
    return self.mapping[ascii]
  }

  var mapping: [String: CGKeyCode] = KeyboardLayout.generateMapping()

  override init() {
    super.init()
    // swiftlint:disable line_length
    DistributedNotificationCenter.default().addObserver(self,
                                                        selector: #selector(KeyboardLayout.keyboardLayoutDidChange),
                                                        name:
                                                          NSNotification.Name(rawValue: NSNotification.Name.RawValue(kTISNotifySelectedKeyboardInputSourceChanged as NSString)),
                                                        object: nil)
  }

  static func generateMapping() -> [String: UInt16] {
    var layout: [String: UInt16] = [:]
    for idx in 0...127 {
      if let key = KeyboardLayout.keyName(scanCode: UInt16(idx)) {
        layout[key] = CGKeyCode(idx)
      }
    }
    return layout
  }

  static let keyboardLayoutDidChangeNotification = Notification.Name("keyboardLayoutDidChange")

  @objc func keyboardLayoutDidChange() {
    // Delay is added to make sure TISCopyCurrentKeyboardLayoutInputSource returns the current layout!
    Timer.delayWithSeconds(0.15) {
      self.mapping = KeyboardLayout.generateMapping()
      NotificationCenter.default.post(Notification(name: KeyboardLayout.keyboardLayoutDidChangeNotification))
    }
  }

  func currentLayoutName() -> String? {
    let inputSource = TISCopyCurrentKeyboardInputSource().takeRetainedValue() as TISInputSource
    return getProperty(inputSource, kTISPropertyLocalizedName) as? String
  }

  private func getProperty(_ source: TISInputSource, _ key: CFString) -> AnyObject? {
    guard let cfType = TISGetInputSourceProperty(source, key) else { return nil }
    return Unmanaged<AnyObject>.fromOpaque(cfType).takeUnretainedValue()
  }
}
