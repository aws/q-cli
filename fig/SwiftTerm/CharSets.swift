//
//  CharSets.swift
//  SwiftTerm
//
//  Created by Miguel de Icaza on 3/1/20.
//  Copyright © 2020 Miguel de Icaza. All rights reserved.
//

import Foundation

class CharSets {
    public static var all: [UInt8:[UInt8:String]] = initAll ()
    
    // This is the "B" charset, null
    public static var defaultCharset: [UInt8:String]? = nil
    
    static func initAll () -> [UInt8:[UInt8:String]]
    {
        var all: [UInt8:[UInt8:String]] = [:]
        
        //
        // DEC Special Character and Line Drawing Set.
        // Reference: http://vt100.net/docs/vt102-ug/table5-13.html
        // A lot of curses apps use this if they see TERM=xterm.
        // testing: echo -e '\e(0a\e(B'
        // The xterm output sometimes seems to conflict with the
        // reference above. xterm seems in line with the reference
        // when running vttest however.
        // The table below now uses xterm's output from vttest.
        //
        all [Character("0").asciiValue!] = [
            Character("`").asciiValue!: "\u{25c6}",// '◆'
            Character("a").asciiValue!: "\u{2592}", // '▒'
            Character("b").asciiValue!: "\u{2409}", // [ht]
            Character("c").asciiValue!: "\u{240c}", // [ff]
            Character("d").asciiValue!: "\u{240d}", // [cr]
            Character("e").asciiValue!: "\u{240a}", // [lf]
            Character("f").asciiValue!: "\u{00b0}", // '°'
            Character("g").asciiValue!: "\u{00b1}", // '±'
            Character("h").asciiValue!: "\u{2424}", // [nl]
            Character("i").asciiValue!: "\u{240b}", // [vt]
            Character("j").asciiValue!: "\u{2518}", // '┘'
            Character("k").asciiValue!: "\u{2510}", // '┐'
            Character("l").asciiValue!: "\u{250c}", // '┌'
            Character("m").asciiValue!: "\u{2514}", // '└'
            Character("n").asciiValue!: "\u{253c}", // '┼'
            Character("o").asciiValue!: "\u{23ba}", // '⎺'
            Character("p").asciiValue!: "\u{23bb}", // '⎻'
            Character("q").asciiValue!: "\u{2500}", // '─'
            Character("r").asciiValue!: "\u{23bc}", // '⎼'
            Character("s").asciiValue!: "\u{23bd}", // '⎽'
            Character("t").asciiValue!: "\u{251c}", // '├'
            Character("u").asciiValue!: "\u{2524}", // '┤'
            Character("v").asciiValue!: "\u{2534}", // '┴'
            Character("w").asciiValue!: "\u{252c}", // '┬'
            Character("x").asciiValue!: "\u{2502}", // '│'
            Character("y").asciiValue!: "\u{2264}", // '≤'
            Character("z").asciiValue!: "\u{2265}", // '≥'
            Character("{").asciiValue!: "\u{03c0}", // 'π{'
            Character("|").asciiValue!: "\u{2260}", // '≠'
            Character("}").asciiValue!: "\u{00a3}", // '£{'
            Character("~").asciiValue!: "\u{00b7}"  // '·'
        ]
        
        // (DEC Alternate character ROM special graphics)
        all [Character("2").asciiValue!] = all [Character("0").asciiValue!]
        
        /**
         * British character set
         * ESC (A
         * Reference: http://vt100.net/docs/vt220-rm/table2-5.html
         */
        all [Character("A").asciiValue!] = [
            Character("#").asciiValue!: "£"
        ]
        
        /**
         * United States character set
         * ESC (B
         */
        all [Character("B").asciiValue!] = [:]
        
        /**
        * Dutch character set
        * ESC (4
        * Reference: http://vt100.net/docs/vt220-rm/table2-6.html
        */
        all [Character("4").asciiValue!] = [
            Character("#").asciiValue!: "£",
            Character("@").asciiValue!: "¾",
            Character("[").asciiValue!: "ĳ",
            Character("\\").asciiValue!: "½",
            Character("]").asciiValue!: "|",
            Character("{").asciiValue!: "¨",
            Character("|").asciiValue!: "f",
            Character("}").asciiValue!: "¼",
            Character("~").asciiValue!: "´"
        ]
        
        /**
         * Finnish character set
         * ESC (C or ESC (5
         * Reference: http://vt100.net/docs/vt220-rm/table2-7.html
         */
        all [Character("5").asciiValue!] = [
            Character("[").asciiValue!: "Ä",
            Character("\\").asciiValue!: "Ö",
            Character("]").asciiValue!: "Å",
            Character("^").asciiValue!: "Ü",
            Character("`").asciiValue!: "é",
            Character("{").asciiValue!: "ä",
            Character("|").asciiValue!: "ö",
            Character("}").asciiValue!: "å",
            Character("~").asciiValue!: "ü"
        ]
        all [Character("C").asciiValue!] = all [Character("5").asciiValue!]

        /**
        * French character set
        * ESC (R
        * Reference: http://vt100.net/docs/vt220-rm/table2-8.html
        */
        all [Character("R").asciiValue!] = [
            Character("#").asciiValue!: "£",
            Character("@").asciiValue!: "à",
            Character("[").asciiValue!: "°",
            Character("\\").asciiValue!: "ç",
            Character("]").asciiValue!: "§",
            Character("{").asciiValue!: "é",
            Character("|").asciiValue!: "ù",
            Character("}").asciiValue!: "è",
            Character("~").asciiValue!: "¨"
        ]
        
        /**
         * French Canadian character set
         * ESC (Q
         * Reference: http://vt100.net/docs/vt220-rm/table2-9.html
         */
        all [Character("Q").asciiValue!] = [
            Character("@").asciiValue!: "à",
            Character("[").asciiValue!: "â",
            Character("\\").asciiValue!: "ç",
            Character("]").asciiValue!: "ê",
            Character("^").asciiValue!: "î",
            Character("`").asciiValue!: "ô",
            Character("{").asciiValue!: "é",
            Character("|").asciiValue!: "ù",
            Character("}").asciiValue!: "è",
            Character("~").asciiValue!: "û"
        ]
        
        /**
         * German character set
         * ESC (K
         * Reference: http://vt100.net/docs/vt220-rm/table2-10.html
         */
        all [Character("K").asciiValue!] = [
            Character("@").asciiValue!: "§",
            Character("[").asciiValue!: "Ä",
            Character("\\").asciiValue!: "Ö",
            Character("]").asciiValue!: "Ü",
            Character("{").asciiValue!: "ä",
            Character("|").asciiValue!: "ö",
            Character("}").asciiValue!: "ü",
            Character("~").asciiValue!: "ß"
        ]
        
        /**
         * Italian character set
         * ESC (Y
         * Reference: http://vt100.net/docs/vt220-rm/table2-11.html
         */
        all [Character("Y").asciiValue!] = [
            Character("#").asciiValue!: "£",
            Character("@").asciiValue!: "§",
            Character("[").asciiValue!: "°",
            Character("\\").asciiValue!: "ç",
            Character("]").asciiValue!: "é",
            Character("`").asciiValue!: "ù",
            Character("{").asciiValue!: "à",
            Character("|").asciiValue!: "ò",
            Character("}").asciiValue!: "è",
            Character("~").asciiValue!: "ì"
        ]
    
        /**
         * Norwegian/Danish character set
         * ESC (E or ESC (6
         * Reference: http://vt100.net/docs/vt220-rm/table2-12.html
         */
        all [Character("6").asciiValue!] = [
            Character("@").asciiValue!: "Ä",
            Character("[").asciiValue!: "Æ",
            Character("\\").asciiValue!: "Ø",
            Character("]").asciiValue!: "Å",
            Character("^").asciiValue!: "Ü",
            Character("`").asciiValue!: "ä",
            Character("{").asciiValue!: "æ",
            Character("|").asciiValue!: "ø",
            Character("}").asciiValue!: "å",
            Character("~").asciiValue!: "ü"
        ]
        all [Character("E").asciiValue!] = all [Character("6").asciiValue!]
        
        /**
         * Spanish character set
         * ESC (Z
         * Reference: http://vt100.net/docs/vt220-rm/table2-13.html
         */
        all [Character("Z").asciiValue!] = [
            Character("#").asciiValue!: "£",
            Character("@").asciiValue!: "§",
            Character("[").asciiValue!: "¡",
            Character("\\").asciiValue!: "Ñ",
            Character("]").asciiValue!: "¿",
            Character("{").asciiValue!: "°",
            Character("|").asciiValue!: "ñ",
            Character("}").asciiValue!: "ç"
        ]

        /**
         * Swedish character set
         * ESC (H or ESC (7
         * Reference: http://vt100.net/docs/vt220-rm/table2-14.html
         */
        all [Character("7").asciiValue!] = [
            Character("@").asciiValue!: "É",
            Character("[").asciiValue!: "Ä",
            Character("\\").asciiValue!: "Ö",
            Character("]").asciiValue!: "Å",
            Character("^").asciiValue!: "Ü",
            Character("`").asciiValue!: "é",
            Character("{").asciiValue!: "ä",
            Character("|").asciiValue!: "ö",
            Character("}").asciiValue!: "å",
            Character("~").asciiValue!: "ü"
        ]
        all [Character("H").asciiValue!] = all [Character("7").asciiValue!]
        
        /**
         * Swiss character set
         * ESC (=
         * Reference: http://vt100.net/docs/vt220-rm/table2-15.html
         */
        all [Character("=").asciiValue!] = [
            Character("#").asciiValue!: "ù",
            Character("@").asciiValue!: "à",
            Character("[").asciiValue!: "é",
            Character("\\").asciiValue!: "ç",
            Character("]").asciiValue!: "ê",
            Character("^").asciiValue!: "î",
            Character("_").asciiValue!: "è",
            Character("`").asciiValue!: "ô",
            Character("{").asciiValue!: "ä",
            Character("|").asciiValue!: "ö",
            Character("}").asciiValue!: "ü",
            Character("~").asciiValue!: "û"
        ]
        return all
    }
}
