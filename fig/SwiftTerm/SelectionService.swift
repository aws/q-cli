//
//  SelectionService.swift
//  iOS
//
//  Created by Miguel de Icaza on 3/5/20.
//  Copyright © 2020 Miguel de Icaza. All rights reserved.
//

import Foundation

/**
 * Tracks the selection state in the terminal
 */
class SelectionService {
    var terminal: Terminal
    
    public init (terminal: Terminal)
    {
        self.terminal = terminal
        _active = false
        start = Position(col: 0, row: 0)
        end = Position(col: 0, row: 0)
        hasSelectionRange = false
    }
    
    /**
     * Controls whether the selection is active or not.   Changing the value will invoke the `selectionChanged`
     * method on the terminal's delegate if the state changes.
     */
    var _active: Bool = false
    public var active: Bool {
        get {
            return _active
        }
        set(newValue) {
            if _active != newValue {
                _active = newValue

                terminal.tdel.selectionChanged (source: terminal)
            }
        }
    }

    /**
     * Whether any range is selected
     */
    public private(set) var hasSelectionRange: Bool

    /**
     * Returns the selection starting point in buffer coordinates
     */
    public private(set) var start: Position {
        didSet {
          hasSelectionRange = start != end
        }
    }

    /**
     * Returns the selection ending point in buffer coordinates
     */
    public private(set) var end: Position {
        didSet {
          hasSelectionRange = start != end
        }
    }
    
    /**
     * Starts the selection from the specific location
     */
    public func startSelection (row: Int, col: Int)
    {
        setSoftStart(row: row, col: col)
        active = true
    }
        
    func clamp (_ p: Position) -> Position {
        let buffer = terminal.buffer
        
        return Position(col: min (p.col, buffer.cols-1), row: min (p.row, buffer.rows-1))
    }
    /**
     * Sets the selection, this is validated against the
     */
    public func setSelection (start: Position, end: Position) {
        let buffer = terminal.buffer
        let sclamped = clamp (start)
        let eclamped = clamp (end)
        
        self.start = sclamped
        self.end = eclamped
    }
    
    /**
     * Starts selection, the range is determined by the last start position
     */
    public func startSelection ()
    {
        end = start
    }
    
    /**
     * Sets the start and end positions but does not start selection
     * this lets us record the last position of mouse clicks so that
     * drag and shift+click operations know from where to start selection
     * from
     */
    public func setSoftStart (row: Int, col: Int)
    {
        guard row < terminal.buffer.rows && col < terminal.buffer.cols else {
            return
        }
        let p = Position(col: col, row: row + terminal.buffer.yDisp)
        start = p
        end = p
    }
    
    /**
     * Extends the selection based on the user "shift" clicking. This has
     * slightly different semantics than a "drag" extension because we can
     * shift the start to be the last prior end point if the new extension
     * is before the current start point.
     */
    public func shiftExtend (row: Int, col: Int)
    {
        active = true
        let newEnd = Position  (col: col, row: row + terminal.buffer.yDisp)
        
        var shouldSwapStart = false
        if Position.compare (start, end) == .before {
            // start is before end, is the new end before Start
            if Position.compare (newEnd, start) == .before {
                // yes, swap Start and End
                shouldSwapStart = true
            }
        } else if Position.compare (start, end) == .after {
            if Position.compare (newEnd, start) == .after {
                // yes, swap Start and End
                shouldSwapStart = true
            }
        }
        
        if (shouldSwapStart) {
            start = end
        }
        
        end = newEnd
        terminal.tdel.selectionChanged(source: terminal)
    }
    
    /**
     * Extends the selection by moving the end point to the new point.
     */
    public func dragExtend (row: Int, col: Int)
    {
        end = Position(col: col, row: row + terminal.buffer.yDisp)
        terminal.tdel.selectionChanged(source: terminal)
    }
    
    /**
     * Selects the entire buffer and triggers the selection
     */
    public func selectAll ()
    {
        start = Position(col: 0, row: 0)
        end = Position(col: terminal.cols-1, row: terminal.buffer.lines.maxLength - 1)
        active = true
    }
    
    /**
     * Selectss the specified row and triggers the selection
     */
    public func select(row: Int)
    {
        start = Position(col: 0, row: row)
        end = Position(col: terminal.cols-1, row: row)
        active = true
    }
    
    /**
     * Performs a simple "word" selection based on a function that determines inclussion into the group
     */
    func simpleScanSelection (from position: Position, in buffer: Buffer, includeFunc: (Character)-> Bool)
    {
        // Look backward
        var colScan = position.col
        var left = colScan
        while colScan >= 0 {
            let ch = buffer.getChar(at: Position (col: colScan, row: position.row)).getCharacter()
            if !includeFunc (ch) {
                break
            }
            left = colScan
            colScan -= 1
        }
        
        // Look forward
        colScan = position.col
        var right = colScan
        let limit = terminal.cols
        while colScan < limit {
            let ch = buffer.getChar(at: Position (col: colScan, row: position.row)).getCharacter()
            if !includeFunc (ch) {
                break
            }
            colScan += 1
            right = colScan
        }
        start = Position (col: left, row: position.row)
        end = Position(col: right, row: position.row)
    }
    
    /**
     * Performs a forward search for the `end` character, but this can extend across matching subexpressions
     * made of pais of parenthesis, braces and brackets.
     */
    func balancedSearchForward (from position: Position, in buffer: Buffer)
    {
        var startCol = position.col
        var wait: [Character] = []
        
        start = position
        
        for line in position.row..<terminal.rows {
            for col in startCol..<terminal.cols {
                let p =  Position(col: col, row: line)
                let ch = buffer.getChar (at: p).getCharacter ()
                
                if ch == "(" {
                    wait.append (")")
                } else if ch == "[" {
                    wait.append ("]")
                } else if ch == "{" {
                    wait.append ("}")
                } else if let v = wait.last {
                    if v == ch {
                        wait.removeLast()
                        if wait.count == 0 {
                            end = Position(col: p.col+1, row: p.row)
                            return
                        }
                    }
                }
            }
            startCol = 0
        }
        start = position
        end = position
    }

    /**
     * Performs a forward search for the `end` character, but this can extend across matching subexpressions
     * made of pais of parenthesis, braces and brackets.
     */
    func balancedSearchBackward (from position: Position, in buffer: Buffer)
    {
        var startCol = position.col
        var wait: [Character] = []

        end = position
        
        for line in (0...position.row).reversed() {
            for col in (0...startCol).reversed() {
                let p =  Position(col: col, row: line)
                let ch = buffer.getChar (at: p).getCharacter ()
                
                if ch == ")" {
                    wait.append ("(")
                } else if ch == "]" {
                    wait.append ("[")
                } else if ch == "}" {
                    wait.append ("{")
                } else if let v = wait.last {
                    if v == ch {
                        wait.removeLast()
                        if wait.count == 0 {
                            end = Position(col: end.col+1, row: end.row)
                            start = p
                            return
                        }
                    }
                }
            }
            startCol = terminal.cols-1
        }
        start = position
        end = position
    }

    let nullChar = Character(UnicodeScalar(0))
    /**
     * Implements the behavior to select the word at the specified position or an expression
     * which is a balanced set parenthesis, braces or brackets
     */
    public func selectWordOrExpression (at uncheckedPosition: Position, in buffer: Buffer)
    {
        let position = Position(
            col: max (min (uncheckedPosition.col, buffer.cols-1), 0),
            row: max (min (uncheckedPosition.row, buffer.rows-1), 0))
        switch buffer.getChar(at: position).getCharacter() {
        case Character(UnicodeScalar(0)):
            simpleScanSelection (from: position, in: buffer) { ch in ch == nullChar }
        case " ":
            // Select all white space
            simpleScanSelection (from: position, in: buffer) { ch in ch == " " }
        case let ch where ch.isLetter || ch.isNumber:
            simpleScanSelection (from: position, in: buffer) { ch in ch.isLetter || ch.isNumber || ch == "." }
        case "{":
            fallthrough
        case "(":
            fallthrough
        case "[":
            balancedSearchForward (from: position, in: buffer)
        case ")":
            fallthrough
        case "]":
            fallthrough
        case "}":
            balancedSearchBackward(from: position, in: buffer)
        default:
            // For other characters, we just stop there
            start = position
            end = position
        }
        active = true

    }
    
    /**
     * Clears the selection
     */
    public func selectNone ()
    {
        active = false
    }
    
    public func getSelectedText () -> String
    {
        let lines = getSelectedLines()
        if lines.count == 0 {
            return ""
        }
        var r = ""
        for line in lines {
            r += line.toString()
        }
        return r
    }
    
    func getSelectedLines() -> [Line]
    {
        var start = self.start
        var end = self.end
        
        switch Position.compare (start, end) {
        case .equal:
            return []
        case .after:
            start = end
            end = start
        case .before:
            break
        }
        if start.row < 0 || start.row > terminal.buffer.lines.count {
            return []
        }
        
        if end.row >= terminal.buffer.lines.count {
            end.row = terminal.buffer.lines.count-1
        }
        return getSelectedLines(start, end)
    }
    
    func getSelectedLines(_ start: Position, _ end: Position) -> [Line]
    {
        var lines: [Line] = []
        let buffer = terminal.buffer
        var str = ""
        var currentLine = Line ()
        lines.append(currentLine)
        
        // keep a list of blank lines that we see. if we see content after a group
        // of blanks, add those blanks but skip all remaining / trailing blanks
        // these will be blank lines in the selected text output
        var blanks: [LineFragment] = []
        
        func addBlanks () {
            var lastLine = -1;
            for b in blanks {
                if lastLine != -1 && b.line != lastLine {
                    currentLine = Line ()
                    lines.append(currentLine)
                }
                
                lastLine = b.line
                currentLine.add(fragment: b)
            }
            blanks = []
        };
        
        // get the first line
        var bufferLine = buffer.lines [start.row]
        if bufferLine.hasAnyContent() {
            let str: String = translateBufferLineToString (buffer: buffer, line: start.row, start: start.col, end: start.row < end.row ? -1 : end.col)
            
            let fragment = LineFragment (text: str, line: start.row, location: start.col, length: str.count)
            currentLine.add (fragment: fragment)
        }
        
        // get the middle rows
        var line = start.row + 1
        var isWrapped = false
        while line < end.row {
            bufferLine = buffer.lines [line]
            isWrapped = bufferLine.isWrapped
            
            str = translateBufferLineToString (buffer: buffer, line: line, start: 0, end: -1)
            
            if bufferLine.hasAnyContent () {
                // add previously gathered blank fragments
                addBlanks ()
                
                if !isWrapped {
                    // this line is not a wrapped line, so the
                    // prior line has a hard linefeed
                    // add a fragment to that line
                    currentLine.add (fragment: LineFragment.newLine (line: line - 1))
                    
                    // start a new line
                    currentLine = Line ()
                    lines.append(currentLine)
                }
                
                // add the text we found to the current line
                currentLine.add (fragment: LineFragment (text: str, line: line, location: 0, length: str.count))
            } else {
                // this line has no content, which means that it's a blank line inserted
                // somehow, or one of the trailing blank lines after the last actual content
                // make a note of the line
                // check that this line is a wrapped line, if so, add a line feed fragment
                if !isWrapped {
                    blanks.append (LineFragment.newLine (line: line - 1))
                }
                
                blanks.append(LineFragment (text: str, line: line, location: 0, length: str.count))
            }
            
            line += 1
        }
        
        // get the last row
        if end.row != start.row {
            bufferLine = buffer.lines [end.row]
            if bufferLine.hasAnyContent () {
                addBlanks ()
                
                isWrapped = bufferLine.isWrapped
                str = translateBufferLineToString (buffer: buffer, line: end.row, start: 0, end: end.col)
                if !isWrapped {
                    currentLine.add(fragment: LineFragment.newLine (line: line - 1))
                    currentLine = Line ()
                    lines.append(currentLine)
                }
                
                currentLine.add (fragment: LineFragment (text: str, line: line, location: 0, length: str.count))
            }
        }
        return lines
    }

    func translateBufferLineToString (buffer: Buffer, line: Int, start: Int, end: Int) -> String
    {
        buffer.translateBufferLineToString(lineIndex: line, trimRight: true, startCol: start, endCol: end).replacingOccurrences(of: "\u{0}", with: " ")
    }
}
