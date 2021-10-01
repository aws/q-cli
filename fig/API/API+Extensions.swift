//
//  API+Extensions.swift
//  fig
//
//  Created by Matt Schrage on 9/28/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import FigAPIBindings
extension NSEvent {
    var fig_keyEvent: Fig_KeyEvent {
        return Fig_KeyEvent.with {

            $0.appleKeyCode = Int32(self.keyCode)
            
            if let characters = self.characters {
                $0.characters = characters
            }
            
            if let charactersIgnoringModifiers = self.charactersIgnoringModifiers {
                $0.charactersIgnoringModifiers = charactersIgnoringModifiers
            }
            
            $0.isRepeat = self.isARepeat
            
            $0.modifiers = {
                var modifiers: [Fig_Modifiers] = []

                if self.modifierFlags.contains(.command) {
                    modifiers.append(.command)
                }
                
                if self.modifierFlags.contains(.option) {
                    modifiers.append(.option)
                }
                
                if self.modifierFlags.contains(.control) {
                    modifiers.append(.control)
                }
                
                if self.modifierFlags.contains(.function) {
                    modifiers.append(.function)
                }
                
                if self.modifierFlags.contains(.shift) {
                    modifiers.append(.shift)
                }
                
                return modifiers
            }()
        }
    }
}
