//
//  Colors.swift
//  SwiftTerm
//
//  Created by Miguel de Icaza on 3/1/20.
//  Copyright © 2020 Miguel de Icaza. All rights reserved.
//

import Foundation

/**
 * This represents the colors used in SwiftTerm, in particular for cells and backgrounds
 * in 16-bit RGB mode
 */
public class Color: Hashable {
    /// Red component 0..65535
    public var red: UInt16
    /// Green component 0..65535
    public var green: UInt16
    /// Blue component 0..65535
    public var blue: UInt16
        
    static var defaultForeground = Color (red: 35389, green: 35389, blue: 35389)
    static var defaultBackground = Color (red: 0, green: 0, blue: 0)
    
    public static func == (lhs: Color, rhs: Color) -> Bool {
        lhs.red == rhs.red && lhs.blue == rhs.blue && lhs.green == rhs.green
    }
    
    public func hash(into hasher: inout Hasher) {
        hasher.combine(red)
        hasher.combine(green)
        hasher.combine(blue)
    }
    
    static let paleColors: [Color] = [
        // dark colors
        Color (red8: 0x2e, green8: 0x34, blue8: 0x36),
        Color (red8: 0xcc, green8: 0x00, blue8: 0x00),
        Color (red8: 0x4e, green8: 0x9a, blue8: 0x06),
        Color (red8: 0xc4, green8: 0xa0, blue8: 0x00),
        Color (red8: 0x34, green8: 0x65, blue8: 0xa4),
        Color (red8: 0x75, green8: 0x50, blue8: 0x7b),
        Color (red8: 0x06, green8: 0x98, blue8: 0x9a),
        Color (red8: 0xd3, green8: 0xd7, blue8: 0xcf),
        
        // bright colors
        Color (red8: 0x55, green8: 0x57, blue8: 0x53),
        Color (red8: 0xef, green8: 0x29, blue8: 0x29),
        Color (red8: 0x8a, green8: 0xe2, blue8: 0x34),
        Color (red8: 0xfc, green8: 0xe9, blue8: 0x4f),
        Color (red8: 0x72, green8: 0x9f, blue8: 0xcf),
        Color (red8: 0xad, green8: 0x7f, blue8: 0xa8),
        Color (red8: 0x34, green8: 0xe2, blue8: 0xe2),
        Color (red8: 0xee, green8: 0xee, blue8: 0xec)
    ]
    
    static let vgaColors: [Color] = [
        // dark colors
        Color (red8: 0, green8: 0, blue8: 0),
        Color (red8: 170, green8: 0, blue8: 0),
        Color (red8: 0, green8: 170, blue8: 0),
        Color (red8: 170, green8: 85, blue8: 0),
        Color (red8: 0, green8: 0, blue8: 170),
        Color (red8: 170, green8: 0, blue8: 170),
        Color (red8: 0, green8: 170, blue8: 170),
        Color (red8: 170, green8: 170, blue8: 170),
        Color (red8: 85, green8: 85, blue8: 85),
        Color (red8: 255, green8: 85, blue8: 85),
        Color (red8: 85, green8: 255, blue8: 85),
        Color (red8: 255, green8: 255, blue8: 85),
        Color (red8: 85, green8: 85, blue8: 255),
        Color (red8: 255, green8: 85, blue8: 255),
        Color (red8: 85, green8: 255, blue8: 255),
        Color (red8: 255, green8: 255, blue8: 255),
    ]
    
    static let terminalAppColors: [Color] = [
        Color (red8: 0, green8: 0, blue8: 0),
        Color (red8: 194, green8: 54, blue8: 33),
        Color (red8: 37, green8: 188, blue8: 36),
        Color (red8: 173, green8: 173, blue8: 39),
        Color (red8: 73, green8: 46, blue8: 225),
        Color (red8: 211, green8: 56, blue8: 211),
        Color (red8: 51, green8: 187, blue8: 200),
        Color (red8: 203, green8: 204, blue8: 205),
        Color (red8: 129, green8: 131, blue8: 131),
        Color (red8: 252, green8: 57, blue8: 31),
        Color (red8: 49, green8: 231, blue8: 34),
        Color (red8: 234, green8: 236, blue8: 35),
        Color (red8: 88, green8: 51, blue8: 255),
        Color (red8: 249, green8: 53, blue8: 248),
        Color (red8: 20, green8: 240, blue8: 240),
        Color (red8: 233, green8: 235, blue8: 235),
    ]
    
    static let xtermColors: [Color] = [
        Color (red8: 0, green8: 0, blue8: 0),
        Color (red8: 205, green8: 0, blue8: 0),
        Color (red8: 0, green8: 205, blue8: 0),
        Color (red8: 205, green8: 205, blue8: 0),
        Color (red8: 0, green8: 0, blue8: 238),
        Color (red8: 205, green8: 0, blue8: 205),
        Color (red8: 0, green8: 205, blue8: 205),
        Color (red8: 229, green8: 229, blue8: 229),
        Color (red8: 127, green8: 127, blue8: 127),
        Color (red8: 255, green8: 0, blue8: 0),
        Color (red8: 0, green8: 255, blue8: 0),
        Color (red8: 255, green8: 255, blue8: 0),
        Color (red8: 92, green8: 92, blue8: 255),
        Color (red8: 255, green8: 0, blue8: 255),
        Color (red8: 0, green8: 255, blue8: 255),
        Color (red8: 255, green8: 255, blue8: 255),
    ]
    
    static let defaultInstalledColors: [Color] = [
        Color (red8: 0, green8: 0, blue8: 0),
        Color (red8: 153, green8: 0, blue8: 1),
        Color (red8: 0, green8: 166, blue8: 3),
        Color (red8: 153, green8: 153, blue8: 0),
        Color (red8: 3, green8: 0, blue8: 178),
        Color (red8: 178, green8: 0, blue8: 178),
        Color (red8: 0, green8: 165, blue8: 178),
        Color (red8: 191, green8: 191, blue8: 191),
        Color (red8: 138, green8: 137, blue8: 138),
        Color (red8: 229, green8: 0, blue8: 1),
        Color (red8: 0, green8: 216, blue8: 0),
        Color (red8: 229, green8: 229, blue8: 0),
        Color (red8: 7, green8: 0, blue8: 254),
        Color (red8: 229, green8: 0, blue8: 229),
        Color (red8: 0, green8: 229, blue8: 229),
        Color (red8: 229, green8: 229, blue8: 229),
    ]
    
    static func setupDefaultAnsiColors (initialColors: [Color]) -> [Color]
    {
        var colors = initialColors
        
        // Fill in the remaining 240 ANSI colors.
        let v = [ 0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff ];
        
        // Generate colors (16-231)
        for i in 0..<216 {
            let r = UInt16 (v [(i / 36) % 6])
            let g = UInt16 (v [(i / 6) % 6])
            let b = UInt16 (v [i % 6])

            colors.append(Color (red8: r, green8: g, blue8: b))
        }

        // Generate greys (232-255)
        for i in 0..<24 {
            let c = UInt16 (8 + i * 10)
            colors.append (Color (red8: c, green8: c, blue8: c))
        }
        return colors
    }
    
    // Contructs a color from 8 bit values, this can be made public,
    // but then we probably should enforce the values to not go
    // beyond 8 bits.   Otherwise, this can throw at runtime due to overflow.
    init(red8: UInt16, green8: UInt16, blue8: UInt16)
    {
        self.red = red8 * 257
        self.green = green8 * 257
        self.blue = blue8 * 257
    }

    // Contructs a color from 4 bit values, this can be made public,
    // but then we probably should enforce the values to not go
    // beyond 4 bits.  Otherwise, this can throw at runtime due to overflow.
    init(red4: UInt16, green4: UInt16, blue4: UInt16)
    {
        // The other one is 4369
        self.red = red4 * 0x1010
        self.green = green4 * 0x1010
        self.blue = blue4 * 0x1010
    }

    /// Initializes a color with the red, green and blue components in the 0...65535 range
    public init(red: UInt16, green: UInt16, blue: UInt16)
    {
        self.red = red
        self.green = green
        self.blue = blue
    }
    
    func formatAsXcolor () -> String
    {
        let rs = String(format:"%04x", red)
        let gs = String(format:"%04x", green)
        let bs = String(format:"%04x", blue)
        return "rgb:\(rs)/\(gs)/\(bs)"
    }
    
    static func parseColor (_ data: ArraySlice<UInt8>) -> Color?
    {
        // parses the hex value until the first "/" and returns both the value, and the number of bytes used
        func parseHex (_ data: ArraySlice<UInt8>, _ idx: inout Int) -> (UInt16, Int)
        {
            var ret: UInt16 = 0
            let limit = data.endIndex
            var count = 0
            idx = max (data.startIndex, idx)
            while count < 4 && idx < limit {
                let c = data [idx]
                idx += 1

                var n: UInt16 = 0
                if c >= UInt8(ascii: "0") && c <= UInt8 (ascii: "9"){
                    n = UInt16 (c - UInt8(ascii: "0"))
                } else if c >= UInt8(ascii: "a") && c <= UInt8 (ascii: "f") {
                    n = UInt16 ((c - UInt8(ascii:"a") + 10))
                } else if c >= UInt8(ascii: "A") && c <= UInt8 (ascii: "F") {
                    n = UInt16 ((c - UInt8(ascii:"A") + 10))
                } else if c == UInt8 (ascii: "/") {
                    break
                } else {
                    break
                }
                count += 1
                ret = ret * 16 + n
            }
            if idx < limit && data [idx] == UInt8(ascii: "/") {
                idx += 1
            }
            return (ret, count)
        }
        
        func makeColor (_ r: UInt16, _ g: UInt16, _ b: UInt16, scale: Int) -> Color?
        {
            switch scale {
            case 1:
                // 4 bit scaled
                return Color(red4: r, green4: g, blue4: b)
            case 2:
                // 8 bit scaled
                return Color(red8: r, green8: g, blue8: b)
            case 3:
                // 12 bit scaled
                return Color(red: (r << 4) | (r >> 4), green: (g << 4) | (g >> 4), blue: (b << 4) | (b >> 4))
            case 4:
                // 16 bits
                return Color(red: r, green: g, blue: b)
            default:
                return nil
            }
        }
        
        // Parse #XXX, #XXXXXX, #XXXXXXXXX color
        if data.first == UInt8 (ascii: "#") {
            let count = data.endIndex-(data.startIndex+1)
            let rest = data [(data.startIndex+1)...]
            let p = data.startIndex+1
            var idx = p
            switch count {
            case 3:
                let (r, _) = parseHex (rest [(p+0)..<(p+1)], &idx)
                let (g, _) = parseHex (rest [(p+1)..<(p+2)], &idx)
                let (b, _) = parseHex (rest [(p+1)..<(p+3)], &idx)
                return makeColor (r, g, b, scale: 1)
            case 6:
                let (r, _) = parseHex (rest [(p+0)..<(p+2)], &idx)
                let (g, _) = parseHex (rest [(p+2)..<(p+4)], &idx)
                let (b, _) = parseHex (rest [(p+4)..<(p+6)], &idx)
                return makeColor (r, g, b, scale: 2)
            case 9:
                let (r, _) = parseHex (rest[(p+0)..<(p+3)], &idx)
                let (g, _) = parseHex (rest[(p+3)..<(p+6)], &idx)
                let (b, _) = parseHex (rest[(p+6)..<(p+9)], &idx)
                return makeColor (r, g, b, scale: 3)
            case 12:
                let (r, _) = parseHex (rest [(p+0)..<(p+4)], &idx)
                let (g, _) = parseHex (rest [(p+4)..<(p+8)], &idx)
                let (b, _) = parseHex (rest [(p+8)..<(p+12)], &idx)
                return makeColor (r, g, b, scale: 4)
            default:
                break
            }
            
        } else if data.starts(with: [UInt8(ascii:"r"), UInt8(ascii:"g"), UInt8(ascii:"b"), UInt8(ascii:":")]) {
            // Parses rgb:X/X/X rgb:XX/XX/XX/XX, rgb:XXX/XXX/XXX, rgb:XXXX/XXXX/XXXX
            var nidx = data.startIndex + 4
            let (r, rlen) = parseHex (data, &nidx)
            let (g, glen) = parseHex (data, &nidx)
            let (b, blen) = parseHex (data, &nidx)
            
            return makeColor (r, g, b, scale: max (rlen, max (glen, blen)))
        }
        return nil
    }    
}
