//
//  Keycodes.swift
//  fig
//
//  Created by Matt Schrage on 9/5/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

// swiftlint:disable identifier_name
//https://gist.github.com/swillits/df648e87016772c7f7e5dbed2b345066
enum Keycode: UInt16, CaseIterable {
  // Layout-independent Keys
  // eg.These key codes are always the same key on all layouts.
  case returnKey                 = 0x24
  case enter                     = 0x4C
  case tab                       = 0x30
  case space                     = 0x31
  case delete                    = 0x33
  case escape                    = 0x35
  case command                   = 0x37
  case shift                     = 0x38
  case capsLock                  = 0x39
  case option                    = 0x3A
  case control                   = 0x3B
  case rightShift                = 0x3C
  case rightOption               = 0x3D
  case rightControl              = 0x3E
  case leftArrow                 = 0x7B
  case rightArrow                = 0x7C
  case downArrow                 = 0x7D
  case upArrow                   = 0x7E
  case volumeUp                  = 0x48
  case volumeDown                = 0x49
  case mute                      = 0x4A
  case help                      = 0x72
  case home                      = 0x73
  case pageUp                    = 0x74
  case forwardDelete             = 0x75
  case end                       = 0x77
  case pageDown                  = 0x79
  case function                  = 0x3F
  case f1                        = 0x7A
  case f2                        = 0x78
  case f4                        = 0x76
  case f5                        = 0x60
  case f6                        = 0x61
  case f7                        = 0x62
  case f3                        = 0x63
  case f8                        = 0x64
  case f9                        = 0x65
  case f10                       = 0x6D
  case f11                       = 0x67
  case f12                       = 0x6F
  case f13                       = 0x69
  case f14                       = 0x6B
  case f15                       = 0x71
  case f16                       = 0x6A
  case f17                       = 0x40
  case f18                       = 0x4F
  case f19                       = 0x50
  case f20                       = 0x5A

  // US-ANSI Keyboard Positions
  // eg. These key codes are for the physical key (in any keyboard layout)
  // at the location of the named key in the US-ANSI layout.
  case a                         = 0x00
  case b                         = 0x0B
  case c                         = 0x08
  case d                         = 0x02
  case e                         = 0x0E
  case f                         = 0x03
  case g                         = 0x05
  case h                         = 0x04
  case i                         = 0x22
  case j                         = 0x26
  case k                         = 0x28
  case l                         = 0x25
  case m                         = 0x2E
  case n                         = 0x2D
  case o                         = 0x1F
  case p                         = 0x23
  case q                         = 0x0C
  case r                         = 0x0F
  case s                         = 0x01
  case t                         = 0x11
  case u                         = 0x20
  case v                         = 0x09
  case w                         = 0x0D
  case x                         = 0x07
  case y                         = 0x10
  case z                         = 0x06

  case zero                      = 0x1D
  case one                       = 0x12
  case two                       = 0x13
  case three                     = 0x14
  case four                      = 0x15
  case five                      = 0x17
  case six                       = 0x16
  case seven                     = 0x1A
  case eight                     = 0x1C
  case nine                      = 0x19

  case equals                    = 0x18
  case minus                     = 0x1B
  case semicolon                 = 0x29
  case apostrophe                = 0x27
  case comma                     = 0x2B
  case period                    = 0x2F
  case forwardSlash              = 0x2C
  case backslash                 = 0x2A
  case grave                     = 0x32
  case leftBracket               = 0x21
  case rightBracket              = 0x1E

  case keypadDecimal             = 0x41
  case keypadMultiply            = 0x43
  case keypadPlus                = 0x45
  case keypadClear               = 0x47
  case keypadDivide              = 0x4B
  case keypadMinus               = 0x4E
  case keypadEquals              = 0x51
  case keypad0                   = 0x52
  case keypad1                   = 0x53
  case keypad2                   = 0x54
  case keypad3                   = 0x55
  case keypad4                   = 0x56
  case keypad5                   = 0x57
  case keypad6                   = 0x58
  case keypad7                   = 0x59
  case keypad8                   = 0x5B
  case keypad9                   = 0x5C

  var keyname: String {
    return String(describing: self)
  }
}

extension CGEventFlags {
  var containsKeyboardModifier: Bool {
    return self.contains(.maskAlternate) || self.contains(.maskCommand) || self.contains(.maskControl) || self.contains(.maskSecondaryFn)
  }
}
