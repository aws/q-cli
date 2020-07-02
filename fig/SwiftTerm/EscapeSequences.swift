//
//  EscapeSequences.swift
//  SwiftTerm
//
//  Created by Miguel de Icaza on 3/30/19.
//  Copyright © 2019 Miguel de Icaza. All rights reserved.
//

import Foundation

struct ControlCodes  {
    static let NUL : UInt8 = 0x00
    static let BEL : UInt8 = 0x07
    static let BS  : UInt8 = 0x08
    static let HT  : UInt8 = 0x09
    static let LF  : UInt8 = 0x0a
    static let VT  : UInt8 = 0x0b
    static let FF  : UInt8 = 0x0c
    static let CR  : UInt8 = 0x0d
    static let SO  : UInt8 = 0x0e
    static let SI  : UInt8 = 0x0f
    static let CAN : UInt8 = 0x18
    static let SUB : UInt8 = 0x1a
    static let ESC : UInt8 = 0x1b
    static let SP  : UInt8 = 0x20
    static let DEL : UInt8 = 0x7f
}

/**
 * Control codes - this structure provides variables that will return strings that are either the 7-bit version of the sequence or the 8-bit one
 * See = https://en.wikipedia.org/wiki/C0_and_C1_control_codes
 */
struct CC {
    var send8bit: Bool
    
    var PAD: [UInt8] { get { send8bit ? [0x80] : [0x1b, 0x40] } }
    var HOP: [UInt8] { get { send8bit ? [0x81] : [0x1b, 0x41] } }
    var BPH: [UInt8] { get { send8bit ? [0x82] : [0x1b, 0x42] } }
    var NBH: [UInt8] { get { send8bit ? [0x83] : [0x1b, 0x43] } }
    var IND: [UInt8] { get { send8bit ? [0x84] : [0x1b, 0x44] } }
    var NEL: [UInt8] { get { send8bit ? [0x85] : [0x1b, 0x45] } }
    var SSA: [UInt8] { get { send8bit ? [0x86] : [0x1b, 0x46] } }
    var ESA: [UInt8] { get { send8bit ? [0x87] : [0x1b, 0x47] } }
    var HTS: [UInt8] { get { send8bit ? [0x88] : [0x1b, 0x48] } }
    var HTJ: [UInt8] { get { send8bit ? [0x89] : [0x1b, 0x49] } }
    var VTS: [UInt8] { get { send8bit ? [0x8a] : [0x1b, 0x4A] } }
    var PLD: [UInt8] { get { send8bit ? [0x8b] : [0x1b, 0x4B] } }
    var PLU: [UInt8] { get { send8bit ? [0x8c] : [0x1b, 0x4C] } }
    var RI:  [UInt8] { get { send8bit ? [0x8d] : [0x1b, 0x4D] } }
    var SS2: [UInt8] { get { send8bit ? [0x8e] : [0x1b, 0x4E] } }
    var SS3: [UInt8] { get { send8bit ? [0x8f] : [0x1b, 0x4F] } }
    var DCS: [UInt8] { get { send8bit ? [0x90] : [0x1b, 0x50] } }
    var PU1: [UInt8] { get { send8bit ? [0x91] : [0x1b, 0x51] } }
    var PU2: [UInt8] { get { send8bit ? [0x92] : [0x1b, 0x52] } }
    var STS: [UInt8] { get { send8bit ? [0x93] : [0x1b, 0x53] } }
    var CCH: [UInt8] { get { send8bit ? [0x94] : [0x1b, 0x54] } }
    var MW:  [UInt8] { get { send8bit ? [0x95] : [0x1b, 0x55] } }
    var SPA: [UInt8] { get { send8bit ? [0x96] : [0x1b, 0x56] } }
    var EPA: [UInt8] { get { send8bit ? [0x97] : [0x1b, 0x57] } }
    var SOS: [UInt8] { get { send8bit ? [0x98] : [0x1b, 0x58] } }
    var SGCI:[UInt8] { get { send8bit ? [0x99] : [0x1b, 0x59] } }
    var SCI: [UInt8] { get { send8bit ? [0x9a] : [0x1b, 0x5A] } }
    var CSI: [UInt8] { get { send8bit ? [0x9b] : [0x1b, 0x5B] } }
    var ST:  [UInt8] { get { send8bit ? [0x9c] : [0x1b, 0x5C] } }
    var OSC: [UInt8] { get { send8bit ? [0x9d] : [0x1b, 0x5D] } }
    var PM:  [UInt8] { get { send8bit ? [0x9e] : [0x1b, 0x5E] } }
    var APC: [UInt8] { get { send8bit ? [0x9f] : [0x1b, 0x5F] } }
}

struct EscapeSequences {
    public static let CmdNewline: [UInt8] = [ 10 ]
    public static let CmdRet: [UInt8] = [ 13 ]
    public static let CmdEsc: [UInt8] = [ 0x1b ]
    public static let CmdDel: [UInt8] = [ 0x7f ]
    public static let CmdDelKey: [UInt8] = [ 0x1b, 0x5b, 0x33, 0x7e ]
    public static let MoveUpApp: [UInt8] = [ 0x1b, 0x4f, 0x41 ]
    public static let MoveUpNormal: [UInt8] = [ 0x1b, 0x5b, 0x41 ]
    public static let MoveDownApp: [UInt8] = [ 0x1b, 0x4f, 0x42 ]
    public static let MoveDownNormal: [UInt8] = [ 0x1b, 0x5b, 0x42 ]
    public static let MoveLeftApp: [UInt8] = [ 0x1b, 0x4f, 0x44 ]
    public static let MoveLeftNormal: [UInt8] = [ 0x1b, 0x5b, 0x44 ]
    public static let MoveRightApp: [UInt8] = [ 0x1b, 0x4f, 0x43 ]
    public static let MoveRightNormal: [UInt8] = [ 0x1b, 0x5b, 0x43 ]
    public static let MoveHomeApp: [UInt8] = [ 0x1b, 0x4f, 0x48 ]
    public static let MoveHomeNormal: [UInt8] = [ 0x1b, 0x5b, 0x48 ]
    public static let MoveEndApp: [UInt8] = [ 0x1b, 0x4f, 0x46 ]
    public static let MoveEndNormal: [UInt8] = [ 0x1b, 0x5b, 0x46 ]
    public static let CmdTab: [UInt8] = [ 9 ]
    public static let CmdBackTab: [UInt8] = [ 0x1b, 0x5b, 0x5a ]
    public static let CmdPageUp: [UInt8] = [ 0x1b, 0x5b, 0x35, 0x7e ]
    public static let CmdPageDown: [UInt8] = [ 0x1b, 0x5b, 0x36, 0x7e ]

    public static let CmdF: [[UInt8]] = [
         [ 0x1b, 0x4f, 0x50 ], /* F1 */
         [ 0x1b, 0x4f, 0x51 ], /* F2 */
         [ 0x1b, 0x4f, 0x52 ], /* F3 */
         [ 0x1b, 0x4f, 0x53 ], /* F4 */
         [ 0x1b, 0x5b, 0x31, 0x35, 0x7e ], /* F5 */
         [ 0x1b, 0x5b, 0x31, 0x37, 0x7e ], /* F6 */
         [ 0x1b, 0x5b, 0x31, 0x38, 0x7e ], /* F7 */
         [ 0x1b, 0x5b, 0x31, 0x39, 0x7e ], /* F8 */
         [ 0x1b, 0x5b, 0x32, 0x30, 0x7e ], /* F9 */
         [ 0x1b, 0x5b, 0x32, 0x31, 0x7e ], /* F10 */
         [ 0x1b, 0x5b, 0x32, 0x33, 0x7e ], /* F11 */
         [ 0x1b, 0x5b, 0x32, 0x34, 0x7e ], /* F12 */
    ]

}
