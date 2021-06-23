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
class KeyboardLayout : NSObject {
    static let shared = KeyboardLayout()
    
    //https://stackoverflow.com/questions/1918841/how-to-convert-ascii-character-to-cgkeycode
    //https://stackoverflow.com/a/35138823
    static func keyName(scanCode: UInt16) -> String? {
       let maxNameLength = 4
       var nameBuffer = [UniChar](repeating: 0, count : maxNameLength)
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
           NSLog("Code: 0x%04X  Status: %+i", scanCode, osStatus);
           return nil
       }

       return  String(utf16CodeUnits: nameBuffer, count: nameLength)
   }
  
    static func humanReadableKeyName(_ event: CGEvent) -> String? {
      guard let event = NSEvent(cgEvent: event) else {
        return nil
      }
      
      switch(event.keyCode) {
        case Keycode.upArrow:
          return "↑"
        case Keycode.downArrow:
          return "↓"
        case Keycode.leftArrow:
          return "←"
        case Keycode.rightArrow:
          return "→"
        case Keycode.delete:
          return "⌫"
        case Keycode.tab:
          return "⇥"
        case Keycode.escape:
          return "<esc>"
        case Keycode.returnKey:
          return "↩"
        default:
          break
      }
      
      var out = ""
      
      if event.modifierFlags.contains(.command) {
        out += "⌘"
      } else if event.modifierFlags.contains(.control) {
        out += "⌃"
      } else if event.modifierFlags.contains(.option) {
        out += "⌥"
      }
     
      if let characters = event.characters {
        out += characters
      } else {
        out += keyName(scanCode: event.keyCode) ?? ""
      }
      
      
      return out
    }
    
    func keyCode(for ascii: String) -> CGKeyCode? {
        return self.mapping[ascii]
    }
    
    var mapping: [String: CGKeyCode] = KeyboardLayout.generateMapping()
    
    override init() {
        super.init()
        DistributedNotificationCenter.default().addObserver(self, selector: #selector(KeyboardLayout.keyboardLayoutDidChange), name: NSNotification.Name(rawValue: NSNotification.Name.RawValue(kTISNotifySelectedKeyboardInputSourceChanged as NSString)), object: nil)
    }
    
    static func generateMapping() -> [String: UInt16] {
        var layout: [String: UInt16] = [:]
        for i in 0...127 {
            if let key = KeyboardLayout.keyName(scanCode: UInt16(i)) {
                layout[key] = CGKeyCode(i)
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
    let inputSource = TISCopyCurrentKeyboardInputSource().takeRetainedValue() as TISInputSource;
    return getProperty(inputSource, kTISPropertyLocalizedName) as? String
  }
  
  private func getProperty(_ source: TISInputSource, _ key: CFString) -> AnyObject? {
      guard let cfType = TISGetInputSourceProperty(source, key) else { return nil }
      return Unmanaged<AnyObject>.fromOpaque(cfType).takeUnretainedValue()
  }
}
