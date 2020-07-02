//
//  CharData.swift
//  SwiftTerm
//
//  Created by Miguel de Icaza on 3/19/19.
//  Copyright © 2019 Miguel de Icaza. All rights reserved.
//

import Foundation

/// This option set describes the character style for a cell, this includes
/// information about the font to use as well as decorations on the text
public struct CharacterStyle : OptionSet, Hashable {
    public let rawValue: UInt8
    
    /**
     * Constructs a character attribute from a raw value.
     */
    public init (rawValue: UInt8)
    {
        self.rawValue = rawValue
    }
    
    /**
     * Constructs the CharacterStyle from a CharData.attribute that encodes the foreground, background and flags
     */
    public init (attribute: Int32)
    {
        rawValue = UInt8 ((attribute >> 18) & 0xff)
    }
    
    /// Empty style
    public static let none = CharacterStyle ([])
    /// Use a bold font
    public static let bold = CharacterStyle (rawValue: 1)
    /// Underline the currentlin line
    public static let underline = CharacterStyle (rawValue: 2)
    /// The text should blink
    public static let blink = CharacterStyle (rawValue: 4)
    /// The text should be inverted (background and foreground are swapped)
    public static let inverse = CharacterStyle (rawValue: 8)
    /// The text should be replaced with white space - there is a debate as to what to do about it when copy/pasting
    /// code as different terminal emulators have taken conflicting takes, so your UI driver might have to choose
    public static let invisible = CharacterStyle (rawValue: 16)
    /// Font should be rendered more lightly, implementation specific
    public static let dim = CharacterStyle (rawValue: 32)
    /// Use italic fonts
    public static let italic = CharacterStyle (rawValue: 64)
    /// Cross out the text
    public static let crossedOut = CharacterStyle (rawValue: 128)
}

///
/// Attribute contains the foreground and background color information for the invidual
/// cells, as well as the character style of the cell (bold, underline, inverse) that the character
/// should be drawn as.
///
public struct Attribute: Equatable, Hashable {
    /// The various ways in which the color was expressed
    public enum Color: Equatable, Hashable {
        /// This means that the foreground color stores 8 bits of information
        /// for the color (the original ANSI colors, plus a crop of colors
        /// and greys - those defined in Color.setupDefaultAnsiColors and additionally
        /// we reserve two values "defaultForeground" and "defaultBackground" that
        /// indicate that the terminal can pick the right values for those.
        case ansi256(code: UInt8)
        
        /// This means that the color has been configured to be a 24-bit true color
        /// and has 8 bits for red, green and blue
        case trueColor(red: UInt8, green: UInt8, blue: UInt8)
        
        /// Indicates that the cell uses the default foreground color
        case defaultColor
        
        /// Indicates that the cell uses teh default backgrond color (also used as the inverse color)
        case defaultInvertedColor
        
        public static func ==(lhs: Color, rhs: Color) -> Bool
        {
            switch (lhs, rhs) {
            case (.ansi256(let lc), .ansi256(let rc)):
                return lc == rc
            case (.defaultColor, .defaultColor):
                return true
            case (.defaultInvertedColor, .defaultInvertedColor):
                return true
            case (.trueColor(let lr, let lg, let lb), .trueColor(let rr, let rg, let rb)):
                return lr == rr && lg == rg && lb == rb
            default:
                return false
            }
        }
    }
    
    /// The empty attribute is configured to be use the defaultColor for the foreground, and the
    /// defaultInvertedColor for the background and an emptu style
    public static let empty = Attribute (fg: .defaultColor, bg: .defaultInvertedColor, style: .none)
    
    /// Foreground and background colors
    public private(set) var fg, bg: Color
    // The cell attributes
    public private(set) var style: CharacterStyle
    
    public static func ==(lhs: Attribute, rhs: Attribute) -> Bool
    {
        lhs.style == rhs.style && lhs.fg == rhs.fg && lhs.bg == rhs.bg
    }
    
    // Returns an attribute with just the colors
    func justColor () -> Attribute
    {
        Attribute (fg: fg, bg: bg, style: .none)
    }
    
    // Temporary, longer term in Attribute we will add a proper encoding
    func toSgr () -> String
    {
        var result = "0"
        
        if style.contains(.bold) {
            result += ";1"
        }
        if style.contains (.underline) {
            result += ";4"
        }
        if style.contains (.blink) {
            result += ";5"
        }
        if style.contains (.inverse) {
            result += ";7"
        }
        if style.contains (.invisible) {
            result += ";8"
        }
        
        print ("Attribute.toSgr() BROKEN - THIS ONLY HANDLES 8 bits")
        switch fg {
        case .ansi256(let c):
            if c > 16 {
                result += ";38;5;\(c)"
            } else {
                result += ";\(c >= 8 ? 9 : 3)\(c >= 8 ? c - 8 : c);"
            }
        case .trueColor(let r, let g, let b):
            print ("Here  is where truecolor needs to be handled \(r), \(g), \(b)")
            break
        default:
            break
        }
        
        switch bg {
        case .ansi256(let c):
            if c > 16 {
                result += ";48;5;\(c)"
            } else {
                result += ";\(c >= 8 ? 10 : 4)\(c >= 8 ? c - 8 : c);"
            }
        case .trueColor(let r, let g, let b):
            print ("Here  is where truecolor needs to be handled \(r), \(g), \(b)")
            break
        default:
            break
        }
        result += "m"
        return result
    }
}

/// TinyAtoms are 16-bit values that can be used to represent a string as a number
/// you create them by calling TinyAtom.lookup (String) and retrieve the
/// value using the `target` property.   They are used to store the urls and any
/// additional parameter information in the OSC 8 scenario.
///
/// This is kept to 16 bits for now, so that we keep the CharData to less than 15 bytes
/// it could in theory be changed to be 24 bits without much trouble
public struct TinyAtom {
    var code: UInt16
    static var stringMap: [UInt16:String] = [:]
    static var lastUsed: Int = 0
    static var empty = TinyAtom (code: 0)
   
    private init(code: UInt16)
    {
        self.code = code
    }
    
    /// Returns the TinyAtom associated with the specified url, or nil if we ran out of space
    public static func lookup (text: String) -> TinyAtom? {
        let next = lastUsed + 1
        if next < UInt16.max {
            stringMap [UInt16 (next)] = text
            lastUsed = next
            return TinyAtom (code: UInt16 (next))
        }
        return nil
    }
    
    /// Returns the target for the TinyAtom
    public var target: String? {
        get {
            if code == 0 {
                return nil
            }
            return TinyAtom.stringMap [code]
        }
    }
}

/**
 * Stores a cell with both the character being displayed as well as the color attribute.
 * This uses an Int32 to store the value, if the value can not be encoded as a single Unicode.Scalar,
 * then an index is stored that is looked up in parallel, so that full grapheme clusters can be tracked.
 *
 * Use the `getCharacter` function to get the stored Character, and use the `attribute` property
 * to retrieve the color and other character attributes.   The `width` property contains the number of
 * columns used by the `Character` stored in this `CharData` on the screen.
 *
 * It is possible to change the value of the stored character by calling the `setValue` method.
 */
public struct CharData {
    static let maxRune = 1 << 22
    
    // Contains the character to index mapping
    static var charToIndexMap: [Character:Int32] = [:]
    
    // Contains the index to character mapping, could be a plain array
    static var indexToCharMap: [Int32: Character] = [:]
    static var lastCharIndex: Int32 = (1 << 22)+1
    
    
    static let defaultAttr = Attribute(fg: .defaultColor, bg: .defaultColor, style: .none)
    static let invertedAttr = Attribute(fg: .defaultInvertedColor, bg: .defaultInvertedColor, style: .none)
    
    // Contains a rune, or a pointer into a Grapheme Cluster
    var code: Int32
    
    ///Contains the number of columns used by the `Character` stored in this `CharData` on the screen.
    public private(set) var width: Int8
    
    // This contains an assigned key
    var urlPayload: TinyAtom
    
    var unused: UInt8 // Purely here to align to 16 bytes
    
    /// The color and character attributes for the cell
    public var attribute: Attribute
    
    /// Initializes a new instance of the CharData structure with the provided attribute, character and the dimension
    /// - Parameter attribute: an attribute containing the color and style attributes for the cell
    /// - Parameter char: the character that will be stored in this cell
    /// - Parameter size: the number of columns used by the `Character` stored in this `CharData` on the screen.
    init (attribute: Attribute, char: Character, size: Int8 = 1)
    {
        self.attribute = attribute
        if char.utf16.count == 1 {
            code = Int32 (char.utf16.first!)
        } else {
            if let existingIdx = CharData.charToIndexMap [char] {
                code = existingIdx
            } else {
                CharData.charToIndexMap [char] = CharData.lastCharIndex
                CharData.indexToCharMap [CharData.lastCharIndex] = char
                code = CharData.lastCharIndex
                CharData.lastCharIndex = CharData.lastCharIndex + 1
            }
        }
        width = Int8 (size)
        urlPayload = TinyAtom.empty
        unused = 0
    }

    // Empty cell sets the code to zero
    init (attribute: Attribute)
    {
        self.attribute = attribute
        code = 0
        width = 1
        urlPayload = TinyAtom.empty
        unused = 0
    }
    
    public var isSimpleRune: Bool {
        get {
            return code < CharData.maxRune
        }
    }

    /// Sets the Url token for the this CharData.
    mutating public func setUrlPayload (atom: TinyAtom)
    {
        self.urlPayload = atom
    }
    
    public func getPayload () -> String?
    {
         urlPayload.target
    }
    
    public var hasUrl: Bool {
        get {
            return urlPayload.code != 0
        }
    }
    
    /// The `Null` character can be used when filling up parts of the screeb
    public static var Null : CharData = CharData (attribute: defaultAttr)
    
    /// Updates the contents of this CharData with a new character.
    /// - Parameter char: the new character that will be stored
    /// - Paramerter size: the number of fixed sized columns this character will take on the screen
    mutating public func setValue (char: Character, size: Int32)
    {
        if char.utf16.count == 1 {
            self.code = Int32 (char.utf16.first!)
        } else {
            if let existingIdx = CharData.charToIndexMap [char] {
                code = existingIdx
            } else {
                CharData.charToIndexMap [char] = CharData.lastCharIndex
                CharData.indexToCharMap [CharData.lastCharIndex] = char
                code = CharData.lastCharIndex
                CharData.lastCharIndex = CharData.lastCharIndex + 1
            }
        }
        width = Int8 (size)
    }
    
    /// Use this method to retrieve the Character stored in the CharData
    public func getCharacter () -> Character
    {
        if code > CharData.maxRune {
            // This is an invariant - no code can be stored without the equivalent being tracked, but for the sake
            // of not having a "!" return a space.
            return CharData.indexToCharMap [code] ?? " "
        }
        if let c = Unicode.Scalar (UInt32 (code)) {
            return Character(c)
        } else {
            return " "
        }
    }
}
