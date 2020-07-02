//
//  SixelDcsHandler.swift
//  
//
//  Created by Anders Borum on 28/04/2020.
//

import Foundation
import CoreGraphics

class SixelDcsHandler : DcsHandler {
    var data: [UInt8]
    var terminal: Terminal

    public init (terminal: Terminal)
    {
        self.terminal = terminal
        data = []
    }
    
    func hook (collect: cstring, parameters: [Int],  flag: UInt8)
    {
        data = []
    }
    
    func put (data : ArraySlice<UInt8>) {
        for x in data {
            self.data.append(x)
        }
    }
    
    private func nextInt(_ p: inout Int) -> Int? {
        var result: Int?
        while p < data.count {
            let c = data[p]
            guard c >= 48 && c <= 57 else {
                return result
            }
            
            let digit = Int(c) - 48
            if let existing = result {
                result = 10 * existing + digit
            } else {
                result = digit
            }
            
            p += 1
        }
        return nil
    }
    
    private func nextIntArray(_ p: inout Int) -> [Int] {
        var result = [Int]()
        while p < data.count {
            // jump past semicolon delimiting integers
            if data[p] == Character(";").asciiValue && p + 1 < data.count {
                p += 1
            }
            
            guard let next = nextInt(&p) else {
                break
            }
            
            result.append(next)
        }
        return result
    }
    
    private func skipToCharacter(_ p: inout Int, _ char: Character) {
        let value = char.asciiValue
        while p < data.count {
            if data[p] == value {
                return
            }
            p += 1
        }
    }
        
    func unhook () {
        var p = 0
        let palette = parsePalette(&p)
        let bitmap = readBitmap(&p)

        // convert bitmap into image for terminal
        if let image = buildImage(palette: palette, bitmap: bitmap) {
            terminal.sixel(image)
        }
    }
    
    // read palette from first line
    private func parsePalette(_ p: inout Int) -> [Int: Color] {
        // palette is sparse where we use default color values for unspecified entries
        var palette = [Int: Color]()

        // skip to # to read palette
        skipToCharacter(&p, "#")
        while p + 1 < data.count && data[p] == Character("#").asciiValue {
            p += 1 // jump past #
            let color = nextIntArray(&p)
            if color.count >= 5 && color[1] == 1 {
                let index = color[0]
                let hue = CGFloat(color[2]) // angle in the range 0 to 360 degrees
                let lightness = 0.01 * CGFloat(color[3]) // percentage from 0 to 100
                let saturation = 0.01 * CGFloat(color[4]) // percentage from 0 to 100
                
                // it isn't entirely clear if lightness == brightness and this page might help:
                //   https://en.wikipedia.org/wiki/HSL_and_HSV
                //
                // using CoreGraphics to convert between HSL and RGB is perhaps a little
                // wasteful but HSL colors in Sixels seems rarely used
                let color = TTColor.make(hue: hue, saturation: saturation,
                                         brightness: lightness, alpha: 1)
                var red = CGFloat(0)
                var green = CGFloat(0)
                var blue = CGFloat(0)
                color.getRed(&red, green: &green, blue: &blue, alpha: nil)
                
                palette[index] = Color(red: UInt16(65535.0 * red),
                                       green: UInt16(65535.0 * green),
                                       blue: UInt16(65535.0 * blue))
            }
            
            if color.count >= 5 && color[1] == 2 {
                let index = color[0]
                let red = 0.01 * CGFloat(color[2]) // percentage from 0 to 100
                let green = 0.01 * CGFloat(color[3]) // percentage from 0 to 100
                let blue = 0.01 * CGFloat(color[4]) // percentage from 0 to 100
                
                palette[index] = Color(red: UInt16(65535.0 * red),
                                       green: UInt16(65535.0 * green),
                                       blue: UInt16(65535.0 * blue))
            }
        }
        
        return palette
    }
    
    // read lines building up bitmap[y][x] with index into
    // or -1 to mean transparent
    private func readBitmap(_ p: inout Int) -> [[Int]] {
        var bitmap = [[Int]]()
        var y = 0
        var x = 0
        func write(color: Int, sixel: Int) {
            for k in 0..<6 {
                let on = (sixel & (1 << k)) != 0
                if on {
                    // make sure we have lines enough
                    while y + k >= bitmap.count {
                        bitmap.append([Int]())
                    }
                    
                    // make sure we have room for this sixel
                    while x >= bitmap[y+k].count {
                        bitmap[y+k].append(-1)
                    }

                    bitmap[y+k][x] = color
                }
            }
            
            x += 1
        }
        
        while p < data.count {
            // read the color entry to use
            skipToCharacter(&p, "#")
            p += 1 // skip past #
            guard let color = nextInt(&p) else {
                // aborting as we failed to read the color
                break
            }
            
            // everything inside loop is for this color
            var reps = 1
            while p < data.count && data[p] != Character("#").asciiValue {
                let c = data[p]
                p += 1
                
                switch c {
                    
                case 33: // ! repeats the next sixel a number of times
                    guard let value = nextInt(&p) else {
                        // ignore repeat
                        continue
                    }
                    reps = value

                case 36: // "$"  (dollar  sign)  character  moves  the  sixel  "cursor"  to the
                         // "beginning of  the current  (same) line
                    x = 0
                    
                case 45: // (hyphen or  minus sign)  character moves the sixel "cursor" to
                         // the "beginning of the next line
                    y += 6
                    x = 0
                    
                case 63...126:
                    for _ in 0..<reps {
                        write(color: color, sixel: Int(c) - 63)
                    }
                    
                    // back to not repeating
                    reps = 1
                    
                default:
                    ()
                }
            }
        }
        
        return bitmap
    }
    
    private func colorForIndex(_ index: Int, _ palette: [Int: Color]) -> Color? {
        guard index >= 0 else {
            // explicit transparency
            return nil
        }
        
        if let color = palette[index] {
            // defined in palette
            return color
        }
        
        // fall back to standard 8-but ANSI colors picking default (0) when outside palette bounds
        let standardIndex = index < terminal.defaultAnsiColors.count ? index : 0
        return terminal.defaultAnsiColors[standardIndex]
    }
    
    private func buildImage(palette: [Int: Color], bitmap: [[Int]]) -> TTImage? {
        // determine size of image
        let height = bitmap.count
        var width = 0
        for y in 0 ..< height {
            let w = bitmap[y].count
            if w > width {
                width = w
            }
        }
        
        guard width * height > 0 else {
            return nil
        }
        
        // build 8+24-bit representation
        var truecolor = [UInt8](repeating: 0, count: 4 * width * height)
        for y in 0 ..< height {
            let line = bitmap[y]
            for x in 0 ..< width {
                guard x < line.count,
                      let color = colorForIndex(line[x], palette) else {
                
                    // no color to write making pixel end up transparent (zero alpha)
                    continue
                }

                let offset = 4 * (width * y + x)
                truecolor[offset + 0] = UInt8(color.red/255)
                truecolor[offset + 1] = UInt8(color.green/255)
                truecolor[offset + 2] = UInt8(color.blue/255)
                truecolor[offset + 3] = 255
            }
        }
        
        // create image from RGB representation
        let rgbColorSpace = CGColorSpaceCreateDeviceRGB()
        let bitmapInfo: CGBitmapInfo = CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue)
        let data = NSData(bytes: &truecolor, length: truecolor.count)
        let providerRef: CGDataProvider? = CGDataProvider(data: data)
        let cgimage: CGImage? = CGImage(width: width, height: height, bitsPerComponent: 8, bitsPerPixel: 32,
                                        bytesPerRow: width * 4, space: rgbColorSpace, bitmapInfo: bitmapInfo,
                                        provider: providerRef!, decode: nil, shouldInterpolate: true,
                                        intent: .defaultIntent)
        return cgimage
    }
}
