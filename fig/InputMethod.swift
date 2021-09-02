//
//  InputMethod.swift
//  fig
//
//  Created by Matt Schrage on 8/30/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
//defaults read ~/Library/Preferences/com.apple.HIToolbox.plist AppleSelectedInputSources
class InputMethod {
    //defaults read ~/Library/Preferences/com.apple.HIToolbox.plist
    //https://developer.apple.com/library/archive/qa/qa1810/_index.html
    @discardableResult static func toggleSource(with id: String = "io.fig.inputmethod.cursor", on: Bool) -> Bool {
//        kTISCategoryPaletteInputSource
        
        let properties = [
            kTISPropertyInputSourceID as String : id,
            kTISPropertyInputSourceType as String : kTISTypeCharacterPalette as String
        ] as CFDictionary

        
        let sources = TISCreateInputSourceList(properties, true).takeUnretainedValue() as! [TISInputSource]
        
        guard let inputMethod = sources[safe: 0] else {
            return false
        }
        
        
        if on {
            return TISEnableInputSource(inputMethod) != noErr
        } else {
            return TISDisableInputSource(inputMethod) != noErr
        }

    }
    
    static func install() -> Bool {
        
        let url = URL(fileURLWithPath: "\(NSHomeDirectory())/Library/Input Methods/FigInputMethod.app")
        let err = TISRegisterInputSource(url as CFURL)
        guard err != paramErr else {return false}
        
        return true
    }
    
    static func keypressTrigger(_ event: CGEvent, _ window: ExternalWindow) -> EventTapAction {
        if [.keyDown, .keyUp ].contains(event.type) {
            let center: DistributedNotificationCenter = DistributedNotificationCenter.default()
            center.postNotificationName(NSNotification.Name("io.fig.keypress"), object: nil, userInfo: ["bundleIdentifier" : window.bundleId ?? ""], deliverImmediately: true)
            print("Sending distributed notification!")
        }

        return .ignore
    }
}
