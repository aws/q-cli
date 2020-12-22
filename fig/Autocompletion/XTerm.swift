//
//  Autocomplete.swift
//  fig
//
//  Created by Matt Schrage on 8/31/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class KeystrokeBuffer : NSObject {
    static let lineAcceptedInKeystrokeBufferNotification: NSNotification.Name = Notification.Name("lineAcceptedInXTermBufferNotification")
    static let lineResetInKeyStrokeBufferNotification: NSNotification.Name = Notification.Name("lineResetInKeyStrokeBufferNotification")
    static let firstCharacterInKeystrokeBufferNotification: NSNotification.Name = Notification.Name("firstCharacterInKeystrokeBufferNotification")

    static let shared = KeystrokeBuffer()
    
    override init() {
        buffer = ""
        index = buffer!.startIndex

    }
    
    var hazy: Bool = true {
        didSet {
            cursor = 0
            index = nil
        }
    }
    var buffer: String? = nil {
        didSet {
            if buffer == nil {
                if (Defaults.playSoundWhenContextIsLost) { NSSound.beep() }
                index = nil
            } else if (buffer == ""){
                NotificationCenter.default.post(name: KeystrokeBuffer.lineResetInKeyStrokeBufferNotification, object: nil)
                index = buffer!.startIndex
                dropStash()
            } else if (buffer?.count == 1) {
                NotificationCenter.default.post(name: KeystrokeBuffer.firstCharacterInKeystrokeBufferNotification, object: nil)

            }
        }
    }
    var cursor: Int = 0
    var historyIndex = 0
    var index: String.Index?
    
    var stashedIndex: String.Index?
    var stashedBuffer: String?
    func stash() {
        print("xterm: stash")

        stashedBuffer = buffer
        stashedIndex = index
        
        buffer = nil
    }
    
    func restore() {
        print("xterm: restore")
        buffer = stashedBuffer
        index = stashedIndex
        historyIndex = 0
    }
    
    func dropStash() {
        print("xterm: dropStash")
        stashedBuffer = nil
        stashedIndex = nil
        historyIndex = 0
    }
    
    
    func handleKeystroke(event: NSEvent) -> (String, Int)? {
        
        
//        if let lines = buffer?.split(separator: "\n"), lines.count > 1 {
//            let lastLine = String(lines.last!)
//            lines.dropLast()
//            let lastLineBuffer = KeystrokeBuffer()
//            lastLineBuffer.buffer = lastLine
//            if let (b, i) = lastLineBuffer.handleKeystroke(event: event) {
//                let combined = lines.joined(separator: "\n")
//                let buf = combined + b
//                let idx = buf.index(combined.endIndex, offsetBy: i)
//                return (buf, idx.utf16Offset(in: buf))
//            } else {
//                return nil
//            }
//        }
        
        
        if (event.modifierFlags.contains(.option)) {
            var char = UniChar()
            var length = 0
            event.cgEvent?.keyboardGetUnicodeString(maxStringLength: 1, actualStringLength: &length, unicodeString: &char)
            let string = String(UnicodeScalar(char)!)
            print("unichar: \(string)")
            // lost context
            if (string.count == 0) {
                buffer = nil
                hazy = true
                return nil
            }
//            else {
//                guard buffer != nil, index != nil else { return nil }
//            
//                buffer!.insert(contentsOf: string, at: index!)
//                index = buffer!.index(index!, offsetBy: string.count)
//                
//                print("xterm: insert option + character! (\(string))")
//            }

        }

        var exit = false
        let withCommand = true
        switch (event.keyCode,  event.modifierFlags.contains(.command)) {
            case (Keycode.v, withCommand):
                if let pasteboard = NSPasteboard.general.string(forType: .string) {
                    print("xterm: Paste! (\(pasteboard))")
                    guard buffer != nil, index != nil else { break }
                    buffer!.insert(contentsOf: pasteboard, at: index!)
                    index = buffer!.index(index!, offsetBy: pasteboard.count, limitedBy: buffer!.endIndex)
                    // Check if text is insert with Return key
//                    if let tail = pasteboard.last, tail == "\n" {
//                        buffer = nil
//                        hazy = true
//                        return nil
//                    }
                }
                
                exit = true
            case (_, false):
                break
            
            default:
                exit = true
                break;
        }
        
        if (exit) {
            if var logging = buffer, index != nil {
                 logging.insert("|", at: index!)
                 print("xterm-out: \(logging) ")
                 
                 return (buffer!, index!.utf16Offset(in: buffer!))

             } else {
                 print("xterm-out: <no context> ")
                 return nil
             }
        }
        
        let withControl = true
        let withoutControl = false
        
        switch (event.keyCode,  event.modifierFlags.contains(.control)) {
        case (Keycode.leftArrow, withoutControl),
             (Keycode.b, withControl):
            guard buffer != nil, index != nil,  index != buffer!.startIndex else { break }
            index = buffer!.index(before: index!)
            
            print("xterm: move cursor to the left by 1")
        case (Keycode.rightArrow, withoutControl),
             (Keycode.f, withControl):
            // handles zsh greyed out text
            if Defaults.deferToShellAutosuggestions && buffer != nil && index == buffer!.endIndex {
                buffer = nil
                print("xterm: possible zsh autosuggest, break context")
            }
            
            guard buffer != nil, index != nil, index != buffer!.endIndex else { break }
            index = buffer!.index(after: index!)
            print("xterm: move cursor to the right by 1")
        case (Keycode.upArrow, withoutControl),
             (Keycode.p, withControl):
            if (historyIndex ==  0) {
                stash()
            }
            historyIndex += 1
            buffer = nil
            print("xterm: previous history")
        case (Keycode.downArrow, withoutControl),
             (Keycode.n, withControl):
            if (historyIndex >= 0) {
                historyIndex -= 1
            }
            
            if (historyIndex == -1 && buffer == nil) {
                restore()
            }
            print("xterm: next history")
        case (Keycode.a, withControl):
            guard buffer != nil, index != nil else { break }
            index = buffer!.startIndex
            
            print("xterm: move to start of current line")
        case (Keycode.e, withControl):
            guard buffer != nil, index != nil else { break }
            index = buffer!.endIndex
            print("xterm: move to end of current line")
        case (Keycode.r, withControl):
            buffer = nil
            print("xterm: reverse search history") // lost context
        case (Keycode.s, withControl):
            buffer = nil
            print("xterm: forwards search history") // lost context
        case (Keycode.delete, withoutControl),
             (Keycode.h, withControl):
            guard buffer != nil, index != nil, index != buffer!.startIndex else { break }
//            print("xterm:", String(buffer!.split(separator: "\n").last!))
            index = buffer!.index(before: index!)
            buffer!.remove(at: index!)
            print("xterm: delete character")
        case (Keycode.d, withControl):
            guard buffer != nil, index != nil, index != buffer!.endIndex else { break }
            buffer!.remove(at: index!)

            print("xterm: delete following character")
        case (Keycode.forwardSlash, withControl):
            guard buffer != nil, index != nil, buffer!.count > 0 else { break }
            buffer!.remove(at: buffer!.index(before: buffer!.endIndex))
            index = buffer!.endIndex
            
            print("xterm: delete character from end")
        case (Keycode.t, withControl):
            guard buffer != nil, index != nil, buffer!.count >= 2 else { break }
            
            if (buffer!.count == 2) {
                index = buffer!.endIndex
            }
            
            
            let second = buffer!.index(before: index!)
            let first = buffer!.index(before: second)
            let a = buffer![first]
            let b = buffer![second]

            buffer!.replaceSubrange(first...second, with: "\(b)\(a)")
                

            print("xterm: transpose")
        case (Keycode.u, withControl):
            // C-k may also do this?
//            guard buffer != nil, index != nil else { break }

            buffer = ""
            index = buffer!.startIndex
            
            print("xterm: kill line")
        case (Keycode.w, withControl):
            guard buffer != nil, index != nil else { break }
            
            let prefix = String(buffer![..<index!])
            
            var allowed = CharacterSet()
            allowed.formUnion(.whitespaces)
            allowed.insert("\"")
            allowed.insert("\\")
            allowed.insert(",")
            allowed.insert("'")
            allowed.insert(":")
            allowed.insert("`")
            allowed.insert("@")


            print("xterm:", prefix.trimTrailingCharacters(in: allowed).split(separator: " "))
            var killed = prefix.trimTrailingCharacters(in: allowed).split(separator: " ", maxSplits: Int.max
                , omittingEmptySubsequences: false).dropLast().joined(separator: " ") + " "
            
            // If deleting the first word, don't add an additional space
            if (killed == " ") {
                killed = ""
            }
            
            buffer!.replaceSubrange(buffer!.startIndex..<index!, with: killed)
            index = killed.endIndex

            print("xterm: kill the word behind point")
        case (Keycode.y, withControl):
            buffer = nil
            
            print("xterm: yank from kill ring") // lost context
        case (Keycode.g, withControl):
            buffer = ""
            index = buffer!.startIndex
            
            print("xterm: abort") // clear buffer
        case (Keycode.returnKey, withoutControl),
             (Keycode.m, withControl),
             (Keycode.j, withControl):

            if buffer != nil, index != nil, buffer!.suffix(1) == "\\" {
//                buffer!.remove(at: buffer!.index(before: buffer!.endIndex))
//                buffer!.insert("\n", at: buffer!.endIndex)
//                index = buffer!.endIndex
                buffer = nil
                print("xterm: accept-line w/ newline")
            } else {
                buffer = ""
                NotificationCenter.default.post(name: KeystrokeBuffer.lineAcceptedInKeystrokeBufferNotification, object: nil)
                print("xterm: accept-line") //clear buffer
            }
            
        case (Keycode.two, withControl):
            buffer = nil
            
            print("xterm: set-mark") //lost context
        case (Keycode.tab, withoutControl),
             (Keycode.i, withControl):
            buffer = nil
            
            print("xterm: complete")
        case (Keycode.c, withControl):
            buffer = ""
            
            print("xterm: kill process")
        case (Keycode.l, withControl):
            print("xterm: clear screen")
        case (_, withControl):
            // Should not push character to buffer
            buffer = nil
            break;

        default:
            guard buffer != nil, index != nil else { break }
            
            if let characters = event.characters {
                var skip = 0
                for (idx, char) in characters.enumerated() {
                    guard char.asciiValue != nil else { break }
                    guard skip == 0 else { skip -= 1; break}
                    print("char:",char, char.utf16, char.asciiValue ?? 0)
                    switch char.asciiValue! {
                    case 8: //backspace literal
                        guard buffer != nil, index != nil, index != buffer!.startIndex else { break }
                        index = buffer!.index(before: index!)
                        buffer!.remove(at: index!)

                        
                        print("xterm: delete character")
                    case 27: // ESC
                        if let direction = characters.index(characters.startIndex, offsetBy: idx + 2, limitedBy: characters.endIndex) {
                            let esc = characters[direction]
                            print("char:",esc)
                            if (esc == "D") { //backward one
                                guard buffer != nil, index != nil,  index != buffer!.startIndex else { break }
                                index = buffer!.index(before: index!)
                                
                                print("xterm: move cursor to the left by 1 (ESC[D)")
                                skip = 2

                            } else if (esc == "C") { // forward one
                                guard buffer != nil, index != nil, index != buffer!.endIndex else { break }
                                index = buffer!.index(after: index!)
                                print("xterm: move cursor to the right by 1 (ESC[C)")
                                skip = 2
                            }
                        }
                        break
                    case 10: // newline literal
                        if buffer != nil, index != nil, buffer!.suffix(1) == "\\" {
                            buffer = nil
                            print("xterm: accept-line w/ newline")
                        } else {
                            buffer = ""
                            print("xterm: accept-line") //clear buffer
                            NotificationCenter.default.post(name: KeystrokeBuffer.lineAcceptedInKeystrokeBufferNotification, object: nil)
                        }
                        
                    default:
                        buffer!.insert(char, at: index!)
                        index = buffer!.index(index!, offsetBy: 1)

                        print("xterm: insert! (\(char))")
                    }


                }

   
//                buffer!.insert(contentsOf: characters, at: index!)
//                index = buffer!.index(index!, offsetBy: characters.count)
//
//                print("xterm: insert! (\(characters))")
            }
            
            break
        }
        
        if var logging = buffer, index != nil {
            // todo: check if index is within bounds
            logging.insert("|", at: index!)
            print("xterm-out: \(logging) ")
            
            return (buffer!, index!.utf16Offset(in: buffer!))

        } else {
            print("xterm-out: <no context> ")
            return nil
        }
        
        
    }
    
    var representation: String {
         if var logging = buffer, index != nil {
               // todo: check if index is within bounds
               logging.insert("|", at: index!)
               return logging
           } else {
               return "<no context>"
           }
    }
    
    func handleKeystroke2(event: NSEvent) -> (String, Int)? {

        if var characters = event.characters {
            print("keystroke:", characters, event.charactersIgnoringModifiers, event.keyCode)
            switch event.keyCode {
            case Keycode.delete: //backspace
                guard buffer != nil && cursor > 0 else {
                    return nil
                }
                // overflow errors
                cursor -= 1
                print("keystroke", cursor, buffer, buffer!.count)
                if let index = buffer!.index(buffer!.startIndex, offsetBy: cursor, limitedBy: buffer!.endIndex) {
                    print("keystroke", index)
                    buffer!.remove(at: index)
                } else {
                    return nil
                }
            
            // hotkeys for moving cursor
            // arrow keys
            case Keycode.leftArrow: //left
                cursor -= 1;
            case Keycode.rightArrow: //up
                cursor += 1;
            case Keycode.upArrow: //right
                buffer = nil
                hazy = true
            case Keycode.downArrow: //down
                buffer = nil
                hazy = true
            case Keycode.tab:
                buffer = nil
                hazy = true
            case Keycode.escape:
                break
            case Keycode.returnKey: // enter
                buffer = nil
                hazy = false
            default:
                guard !hazy else { return nil }
                
                if (buffer == nil) {
                    buffer = ""
                }
                
//                if (event.modifierFlags.contains(.command)) {
//                    print("keylogger CMD, \(event.characters) \(event.keyCode)")
//
//                    break;
//                }
                
                if (event.modifierFlags.contains(.control)) {
                    print("keylogger CONTROL, \(event.characters ?? "<none>") \(event.keyCode)")
                    break
                }
                
                // handle copy/paste
                let pasteboard = NSPasteboard.general.string(forType: .string)

                print("keylogger paste: \(pasteboard ?? "<none>")")
                if (event.keyCode == Keycode.v && event.modifierFlags.contains(.command)) {
                   characters = pasteboard!
                }
                
                
//                CharacterSet.controlCharacters.contains(characters)
                
                if (characters.canBeConverted(to: String.Encoding.ascii)) {
                    buffer!.insert(contentsOf: characters, at: buffer!.index(buffer!.startIndex, offsetBy: cursor))
                    cursor += characters.count
                }
            }
        }
        
        print("keylogger", buffer)
        // probably should return both a string and the cursor index
        return buffer == nil ? nil : (buffer!, cursor)
    }
}

//extension CharacterSet {
//    func containsUnicodeScalars(of character: Character) -> Bool {
//        return character.unicodeScalars.allSatisfy(contains(_:))
//    }
//}


//class xterm {
//    //https://linux.die.net/man/3/readline
//    static func handleEvent(_ event: NSEvent)  {
//
//
//        if (event.modifierFlags.contains(.option)) {
//            // lost context
//        }
//
//        let withCommand = true
//        switch (event.keyCode,  event.modifierFlags.contains(.command)) {
//            case (Keycode.v, withCommand):
//                let pasteboard = NSPasteboard.general.string(forType: .string)
//                print("xterm: Paste! (\(pasteboard))")
//            default:
//                break;
//        }
//
//        let withControl = true
//        let withoutControl = false
//
//        switch (event.keyCode,  event.modifierFlags.contains(.control)) {
//        case (Keycode.leftArrow, withoutControl),
//             (Keycode.b, withControl):
//            print("xterm: move cursor to the left by 1")
//        case (Keycode.rightArrow, withoutControl),
//             (Keycode.f, withControl):
//            print("xterm: move cursor to the right by 1")
//        case (Keycode.upArrow, withoutControl),
//             (Keycode.p, withControl):
//            print("xterm: previous history")
//        case (Keycode.downArrow, withoutControl),
//             (Keycode.n, withControl):
//            print("xterm: next history")
//        case (Keycode.a, withControl):
//            print("xterm: move to start of current line")
//        case (Keycode.e, withControl):
//            print("xterm: move to end of current line")
//        case (Keycode.r, withControl):
//            print("xterm: reverse search history") // lost context
//        case (Keycode.s, withControl):
//            print("xterm: forwards search history") // lost context
//        case (Keycode.delete, withoutControl),
//             (Keycode.h, withControl):
//            print("xterm: delete character")
//        case (Keycode.forwardSlash, withControl):
//            print("xterm: delete character from end")
//        case (Keycode.t, withControl):
//            print("xterm: transpose")
//        case (Keycode.u, withControl):
//            // C-k may also do this?
//            print("xterm: kill line")
//        case (Keycode.w, withControl):
//            print("xterm: kill the word behind point")
//        case (Keycode.y, withControl):
//            print("xterm: yank from kill ring") // lost context
//        case (Keycode.g, withControl):
//            print("xterm: abort") // clear buffer
//        case (Keycode.returnKey, withoutControl),
//             (Keycode.m, withControl),
//             (Keycode.j, withControl):
//            print("xterm: accept-line") //clear buffer
//        case (Keycode.two, withControl):
//            print("xterm: set-mark") //lost context
//        case (Keycode.tab, withoutControl),
//             (Keycode.i, withControl):
//            print("complete")
//        case (_, withControl):
//            // Should not push character to buffer
//            break;
//
//        default:
//            break
//        }
//    }
//}


extension String {
    mutating func swap(at index: String.Index, to character: Character) {
        let endIndex = self.index(after: index)
        let range = index ..< endIndex
        assert(indices.contains(index) && indices.contains(endIndex))
        replaceSubrange(range, with: String(character))
    }
    
    func trimTrailingCharacters(in characterSet : CharacterSet) -> String {
       if let range = rangeOfCharacter(from: characterSet, options: [.anchored, .backwards]) {
        return String(self[..<range.lowerBound]).trimTrailingCharacters(in:
            characterSet)
       }
       return self
    }
}
