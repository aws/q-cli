//
//  Buffer.swift
//  SwiftTerm
//
//  Created by Miguel de Icaza on 3/26/19.
//  Copyright © 2019 Miguel de Icaza. All rights reserved.
//

import Foundation

/**
 * The buffer object contains both the lines that are shwon the user (including the scorllback) as well
 * as attribuets like the cursor (x, y) position, the defined scroll region, the tab stops, the left and right
 * margins and the scrolling delta.
 *
 * Some of the saved state information is also tracked here.
 */
class Buffer {
    var _lines : CircularList<BufferLine>
    var xDisp, _yDisp, xBase : Int
    var _x, _y, _yBase : Int
    
    /// This is the index into the `lines` array that corresponds to the top row of displayed
    /// content in the terminal when the scroll is zero.   So the terminal contents that the application
    /// has access to are `lines [yBase..(yBase+rows)]`
    var yBase: Int {
        get { _yBase }
        set {
            if newValue > _lines.count {
                //abort ()
            }
            _yBase = newValue
        }
    }
    /// This property tracks the first row in the `lines` array that will be displayed as the top row
    /// when scrolling takes place, this variable is updated to move the window of visible content
    public var yDisp: Int {
        get { return _yDisp }
        set {
            _yDisp = newValue
            if _yDisp < 0 {
                abort()
            }
        }
    }
    /**
     * This is the cursor column 0-based, due to the way that the terminal must behave, buffer.x sometimes can
     * be beyond the boundary of the buffer, so it is important that any writes to a line with [buffer.x] first
     * does so by clamping the value to cols-1.
     *
     */
    public var x : Int {
        get { return _x }
        set(newValue) {
            if newValue < 0 {
                abort ()
            }
            _x = newValue
        }
    }
    
    /**
     * This is the cursor row 0-based
     */
    public var y : Int {
        get { return _y }
        set(newValue) {
            _y = newValue
        }
    }
    
    var _scrollBottom: Int
    /**
     * This sets the bottom of the scrolling region in the buffer when Origin Mode is turned on
     */
    public var scrollBottom: Int {
        get { _scrollBottom }
        set {
            _scrollBottom = newValue
            if newValue < 0 {
                abort()
            }
        }
    }
    
    var _scrollTop: Int

    /**
     * This sets the top scrolling region in the buffer when Origin Mode is turned on
     */
    public var scrollTop : Int {
        set(newValue) {
            if newValue >= 0 {
                _scrollTop = newValue
            }
        }
        get {
            return _scrollTop
        }
    }
    var tabStops : [Bool]
    
    /**
     * This records the saved X position
     */
    public var savedX: Int
    
    /**
     * This records the saved Y position
     */
    public var savedY: Int

    /**
     * The left margin, 0-indexed, used when marginMode is turned on
     */
    public var marginLeft: Int = 0

    /**
     * The right margin, 0-indexed, used when marginMode is turned on
     */
    public var marginRight: Int = 0
    
    /**
     * This represents the saved attributed
     */
    public var savedAttr = CharData.defaultAttr
    
    /**
     * This tracks the current charset
     */
    public var savedCharset: [UInt8:String]? = nil
    
    var hasScrollback : Bool
    var cols, rows: Int
    
    var terminal: Terminal
    
    var lines : CircularList<BufferLine> {
        get { return _lines }
    }
    
    public init (_ terminal : Terminal, hasScrollback : Bool = true)
    {
        self.terminal = terminal
        self.hasScrollback = hasScrollback
        _yDisp = 0
        xDisp = 0
        _yBase = 0
        tabStops = [Bool]()
        savedX = 0
        savedY = 0
        xBase = 0
        _scrollTop = 0
        _scrollBottom = terminal.rows - 1
        _x = 0
        _y = 0
        cols = terminal.cols
        rows = terminal.rows
        
        let len = hasScrollback ? (terminal.options.scrollback) + rows : rows
        _lines = CircularList<BufferLine> (maxLength: len)
        _lines.makeEmpty = makeEmptyLine
        setupTabStops ()
    }
    
    public func getCorrectBufferLength (_ rows: Int) -> Int
    {
        if hasScrollback {
            let correct = rows + (terminal.options.scrollback)
            return correct > Int32.max ? Int (Int32.max) : correct
        } else {
            return rows
        }
    }
    
    public func getNullCell (attribute: Attribute? = nil) -> CharData
    {
        let fgbg = attribute == nil ? Attribute.empty : attribute!.justColor ()
        return CharData(attribute: fgbg, char: " ", size: 1)
    }
    
    public func getBlankLine (attribute: Attribute, isWrapped: Bool = false) -> BufferLine
    {
        let cd = CharData (attribute: attribute)
        
        return BufferLine(cols: terminal.cols, fillData: cd, isWrapped: isWrapped)
    }
    
    func makeEmptyLine (_ line: Int) -> BufferLine
    {
        return getBlankLine(attribute: CharData.defaultAttr, isWrapped: false)
    }
    
    /**
     * Returns the CharData at the specified position, the screen coordinate is what the user
     * sees.
     */
    public func getChar (at: Position) -> CharData
    {
        let bufferRow = lines [at.row]
        let col = at.col
        if col >= bufferRow.count || col < 0 {
            return CharData.Null
        }
        return bufferRow [at.col]
    }
    
    public func clear ()
    {
        yDisp = 0
        xBase = 0
        x = 0
        y = 0
        
        _lines = CircularList<BufferLine> (maxLength: getCorrectBufferLength(terminal.rows))
        _lines.makeEmpty = makeEmptyLine
        scrollTop = 0
        scrollBottom = terminal.rows - 1
        
        // Figure out how to do this elegantly
        // SetupTabStops ()
    }
    
    public var isCursorInViewPort : Bool {
        get {
            let absoluteY = yBase + yDisp
            let relativeY = absoluteY + yDisp
            return relativeY >= 0 && relativeY < terminal.rows
        }
    }
    
    public func fillViewportRows (attribute : Attribute? = nil)
    {
        // TODO: limitation in original, this does not cope with partial fills, it is either zero or nothing
        if _lines.count != 0 {
            return
        }
        let attr = attribute != nil ? attribute! : CharData.defaultAttr
        for _ in 0..<terminal.rows {
            _lines.push (getBlankLine (attribute: attr))
        }
    }
    
    public var isReflowEnabled: Bool {
        return hasScrollback
    }
    
    public func resize (newCols : Int, newRows : Int)
    {
        if marginRight > newCols - 1 {
            marginRight = newCols - 1
        }
        if marginLeft >= marginRight {
            marginLeft = marginRight
        }
        let newMaxLength = getCorrectBufferLength(newRows)
        if newMaxLength > lines.maxLength {
            lines.maxLength = newMaxLength
        }
        if lines.count > 0 {
            // Deal with columns increasing (reducing needs to happen after reflow)
            
            if cols < newCols {
                for i in 0..<lines.maxLength {
                    lines [i].resize (cols: newCols, fillData: CharData.Null)
                }

            }

            // Resize rows in both directions as needed
            var addToY = 0
            if rows < newRows {
                for y in rows..<newRows {
                    if lines.count < newRows + yBase {
                        if yBase > 0 && lines.count <= yBase + y + addToY + 1 {
                            // There is room above the buffer and there are no empty elements below the line,
                            // scroll up
                            yBase -= 1
                            addToY += 1
                            if yDisp > 0 {
                                // Viewport is at the top of the buffer, must increase downwards
                                yDisp -= 1
                            }
                        } else {
                            // Add a blank line if there is no buffer left at the top to scroll to, or if there
                            // are blank lines after the cursor
                            lines.push (BufferLine (cols: newCols, fillData: CharData.Null))
                        }
                    }
                }
            } else { // (this._rows >= newRows)
                for _ in (newRows..<rows).reversed () {
                    if lines.count > newRows + yBase {
                        if lines.count > yBase + self.y + 1 {
                            // The line is a blank line below the cursor, remove it
                            lines.pop ()
                        } else {
                            // The line is the cursor, scroll down
                            yBase += 1
                            yDisp += 1
                        }
                    }
                }
            }

            // Reduce max length if needed after adjustments, this is done after as it
            // would otherwise cut data from the bottom of the buffer.
            if newMaxLength < lines.maxLength {
                // Trim from the top of the buffer and adjust ybase and ydisp.
                let amountToTrim = lines.count - newMaxLength
                if amountToTrim > 0 {
                    lines.trimStart(count: amountToTrim)
                    yBase = max (yBase - amountToTrim, 0)
                    yDisp = max (yDisp - amountToTrim, 0)
                    savedY = max (savedY - amountToTrim, 0)
                }

                lines.maxLength = newMaxLength
            }

            // Make sure that the cursor stays on screen
            x = min (x, newCols - 1)
            y = min (y, newRows - 1)
            if addToY != 0 {
                y += addToY
            }

            savedX = min (savedX, newCols - 1)

            scrollTop = 0
        }
        scrollBottom = newRows - 1
        if tabStops.count > newCols {
            tabStops.removeSubrange (newCols..<tabStops.count-1)
        } else {
            let n = newCols - tabStops.count
            for _ in 0..<n {
                tabStops.append (false)
            }
        }
        
        if isReflowEnabled {
            reflow (newCols, newRows)
            // Trim the end of the line off if cols shrunk
            if cols > newCols {
                for i in 0..<lines.maxLength {
                    lines [i].resize (cols: newCols, fillData: CharData.Null)
                }
            }
        }
        
        // DEBUG: Post-condition
        if lines.count > 0 {
            for i in 0..<lines.maxLength {
                let line = lines [i]
                if line.count < newCols {
                    print ("stop here newCols=\(newCols) but the element has: \(line.count)")
                    abort ()
                }
            }
        }
        rows = newRows
        cols = newCols
    }
    
    func translateBufferLineToString (lineIndex: Int, trimRight: Bool, startCol: Int = 0, endCol: Int = -1) -> String
    {
        let line = _lines [lineIndex]
        return line.translateToString(trimRight: trimRight, startCol: startCol, endCol: endCol)
    }
    
    func setupTabStops (index: Int = -1)
    {
        let cols = terminal.cols
        var idx = index
        
        if idx != -1 {
            if tabStops.count > cols {
                tabStops.removeSubrange(cols...)
            } else {
                for _ in cols..<tabStops.count {
                    tabStops.append(false)
                }
            }
            let from = min (index, cols - 1)
            if !tabStops [from] {
                idx = previousTabStop (from)
            }
        } else {
            tabStops = Array.init (repeating: false, count: cols)
            idx = 0
        }
        let tabStopWidth = terminal.tabStopWidth
        for i in stride(from: idx, to: cols, by: tabStopWidth) {
            tabStops [i] = true
        }
    }
    
    func tabSet (pos : Int)
    {
        if pos < tabStops.count {
            tabStops [pos] = true
        }
    }
    
    func tabClear (pos : Int)
    {
        if pos < tabStops.count {
            tabStops [pos] = false
        }
    }
    
    func clearTabStops ()
    {
        tabStops = Array.init (repeating: false, count: tabStops.count)
    }
    
    func previousTabStop (_ index : Int = -1) -> Int
    {
        var idx = index == -1 ? x : index
        while idx > 0 && !tabStops [idx-1] {
            idx = idx - 1
        }
        if idx > 0 {
            idx -= 1
        }
        return idx >= terminal.cols ? terminal.cols - 1 : idx
    }
    
    func nextTabStop (_ index : Int = -1) -> Int
    {
        // Users marginMode because apparently for tabs, there is no need to have originMode set
        let limit = terminal.marginMode ? marginRight : (terminal.cols-1)
        var idx = index == -1 ? x : index
        
        repeat {
            idx = idx + 1
            if idx > limit {
                break
            }
            if tabStops [idx] {
                break
            }
        } while idx < limit
        return idx >= limit ? limit : idx
    }
    
    func getWrappedLineTrimmedLength (_ lines: CircularList<BufferLine>, _ row: Int, _ cols: Int) -> Int
    {
        return getWrappedLineTrimmedLength (lines [row], row == lines.count - 1 ? nil : lines [row + 1], cols)
    }

    func getWrappedLineTrimmedLength (_ lines: [BufferLine], _ row: Int, _ cols: Int) -> Int
    {
        return getWrappedLineTrimmedLength (lines [row], row == lines.count - 1 ? nil : lines [row+1], cols)
    }

    func getWrappedLineTrimmedLength (_ line: BufferLine, _ nextLine: BufferLine?, _ cols: Int) -> Int
    {
        // If this is the last row in the wrapped line, get the actual trimmed length
        if nextLine == nil {
            return line.getTrimmedLength ()
        }

        // Detect whether the following line starts with a wide character and the end of the current line
        // is null, if so then we can be pretty sure the null character should be excluded from the line
        // length]
        let endsInNull = !(line.hasContent (index: cols - 1)) && line.getWidth (index: cols - 1) == 1
        let followingLineStartsWithWide = nextLine?.getWidth (index: 0) == 2

        if endsInNull && followingLineStartsWithWide {
            return cols - 1
        }

        return cols
    }

    func getLinesToRemove (oldCols: Int, newCols: Int, bufferAbsoluteY: Int, nullChar: CharData) -> [Int]
    {
        // Gather all BufferLines that need to be removed from the Buffer here so that they can be
        // batched up and only committed once
        var toRemove : [Int] = []

        var y = 0
        while y < lines.count-1 {
            defer { y = y + 1}
            // Check if this row is wrapped
            var i = y
            i = i + 1
            var nextLine = lines [i]
            if !nextLine.isWrapped {
                continue
            }

            // Check how many lines it's wrapped for
            var wrappedLines : [BufferLine] = []
            wrappedLines.append (lines [y])
            while i < lines.count && nextLine.isWrapped {
                wrappedLines.append (nextLine)
                i += 1
                nextLine = lines [i]
            }

            // If these lines contain the cursor don't touch them, the program will handle fixing up wrapped
            // lines with the cursor
            if bufferAbsoluteY >= y && bufferAbsoluteY < i {
                y += wrappedLines.count - 1
                continue
            }

            // Copy buffer data to new locations
            var destLineIndex = 0
            var destCol = getWrappedLineTrimmedLength (lines, destLineIndex, oldCols)
            var srcLineIndex = 1
            var srcCol = 0
            while srcLineIndex < wrappedLines.count {
                let srcTrimmedTineLength = getWrappedLineTrimmedLength (wrappedLines, srcLineIndex, oldCols)
                let srcRemainingCells = srcTrimmedTineLength - srcCol
                let destRemainingCells = newCols - destCol
                let cellsToCopy = min (srcRemainingCells, destRemainingCells)

                wrappedLines [destLineIndex].copyFrom (wrappedLines [srcLineIndex], srcCol: srcCol, dstCol: destCol, len: cellsToCopy)

                destCol += cellsToCopy;
                if destCol == newCols {
                    destLineIndex += 1
                    destCol = 0;
                }

                srcCol += cellsToCopy;
                if srcCol == srcTrimmedTineLength {
                    srcLineIndex += 1
                    srcCol = 0;
                }

                // Make sure the last cell isn't wide, if it is copy it to the current dest
                if destCol == 0 && destLineIndex != 0 {
                    if wrappedLines [destLineIndex - 1].getWidth(index: newCols - 1) == 2 {
                        wrappedLines [destLineIndex].copyFrom (wrappedLines [destLineIndex - 1], srcCol: newCols - 1, dstCol: destCol, len: 1)
                        destCol += 1
                        // Null out the end of the last row
                        wrappedLines [destLineIndex - 1].replaceCells (start: newCols - 1, end: 1, fillData: nullChar)
                    }
                }
            }

            // Clear out remaining cells or fragments could remain;
            wrappedLines [destLineIndex].replaceCells (start: destCol, end: newCols, fillData: nullChar)

            // Work backwards and remove any rows at the end that only contain null cells
            var countToRemove = 0
            var ix = wrappedLines.count-1
            
            while ix > 0 {
                defer { ix = ix - 1 }
                if ix > destLineIndex || wrappedLines [ix].getTrimmedLength () == 0 {
                    countToRemove += 1
                } else {
                    break
                }
            }

            if countToRemove > 0 {
                toRemove.append (y + wrappedLines.count - countToRemove) // index
                toRemove.append (countToRemove)
            }

            y += wrappedLines.count - 1
        }

        return toRemove
    }
    
    func reflowWider (_ oldCols: Int, _ oldRows: Int, _ newCols: Int, _ newRows: Int)
    {
        let toRemove = getLinesToRemove(oldCols: oldCols, newCols: newCols, bufferAbsoluteY: yBase + y, nullChar: CharData.Null)
        
        //print ("Lines to remove: \(toRemove) \(toRemove.count)")
        if toRemove.count > 0 {
            // Create new layout
            let layout = CircularList<Int> (maxLength: lines.count)
            layout.makeEmpty = { line in 0 }

            // First iterate through the list and get the actual indexes to use for rows
            var nextToRemoveIndex = 0
            var nextToRemoveStart = toRemove [nextToRemoveIndex]
            var countRemovedSoFar = 0

            var i = 0
            while i < lines.count {
                if nextToRemoveStart == i {
                    nextToRemoveIndex += 1
                    let countToRemove = toRemove [nextToRemoveIndex]

                    i += countToRemove - 1
                    countRemovedSoFar += countToRemove

                    nextToRemoveStart = Int.max
                    if nextToRemoveIndex < toRemove.count - 1 {
                        nextToRemoveIndex += 1
                        nextToRemoveStart = toRemove [nextToRemoveIndex]
                    }
                } else {
                    layout.push (i)
                }
                i += 1
            }

            // Apply the new layout
            let newLayoutLines = CircularList<BufferLine> (maxLength: lines.count)
            newLayoutLines.makeEmpty = makeEmptyLine
            for i in 0..<layout.count {
                  newLayoutLines.push (lines [layout [i]])
            }
                  
            // Rearrange the list
            for i in 0..<newLayoutLines.count {
                  lines [i] = newLayoutLines [i]
            }
            lines.count = layout.count
            
            // adjust viewport
            var viewportAdjustments = countRemovedSoFar
            while viewportAdjustments > 0 {
                viewportAdjustments -= 1
                if yBase == 0 {
                    if y > 0 {
                        y -= 1
                    }
    
                    if lines.count < newRows {
                        // Add an extra row at the bottom of the viewport
                        lines.push (BufferLine (cols: newCols, fillData: CharData.Null))
                    }
                } else {
                    if yDisp == yBase {
                        yDisp -= 1
                    }
                    yBase -= 1
                }
            }
            savedY = max (savedY - countRemovedSoFar, 0)
        }
    }
    
    // Gets the new line lengths for a given wrapped line. The purpose of this function it to pre-
    // compute the wrapping points since wide characters may need to be wrapped onto the following line.
    // This function will return an array of numbers of where each line wraps to, the resulting array
    // will only contain the values `newCols` (when the line does not end with a wide character) and
    // `newCols - 1` (when the line does end with a wide character), except for the last value which
    // will contain the remaining items to fill the line.
    // Calling this with a `newCols` value of `1` will lock up.
    func getNewLineLengths (wrappedLines: [BufferLine] , oldCols: Int, newCols: Int) -> [Int]
    {
        var newLineLengths : [Int] = []

        var cellsNeeded = 0
        for i in 0..<wrappedLines.count {
               cellsNeeded += getWrappedLineTrimmedLength (wrappedLines, i, oldCols)
        }

        // Use srcCol and srcLine to find the new wrapping point, use that to get the cellsAvailable and
        // linesNeeded
        var srcCol = 0;
        var srcLine = 0;
        var cellsAvailable = 0;
        while cellsAvailable < cellsNeeded {
               if cellsNeeded - cellsAvailable < newCols {
                       // Add the final line and exit the loop
                       newLineLengths.append (cellsNeeded - cellsAvailable)
                       break;
               }

               srcCol += newCols
               let oldTrimmedLength = getWrappedLineTrimmedLength (wrappedLines, srcLine, oldCols)
               if srcCol > oldTrimmedLength {
                       srcCol -= oldTrimmedLength
                       srcLine += 1
               }

               let endsWithWide = wrappedLines [srcLine].getWidth(index: srcCol - 1) == 2
               if endsWithWide {
                       srcCol -= 1
               }

               let lineLength = endsWithWide ? newCols - 1 : newCols
               newLineLengths.append (lineLength)
               cellsAvailable += lineLength
        }

        return newLineLengths
    }

    struct InsertionSet {
        var lines: [BufferLine]
        var start: Int
        var isNull: Bool
        public static func Null () -> InsertionSet { InsertionSet (lines: [], start: 0, isNull: true) }
    }
    
    func reflowNarrower (_ oldCols: Int, _ oldRows: Int, _ newCols: Int, _ newRows: Int)
    {
        // Gather all BufferLines that need to be inserted into the Buffer here so that they can be
        // batched up and only committed once
        var toInsert : [InsertionSet] = []
        var countToInsert = 0

        // Go backwards as many lines may be trimmed and this will avoid considering them
        var y = lines.count-1
        while y >= 0 {
            defer { y -= 1 }
            // Check whether this line is a problem or not, if not skip it
            var nextLine = lines [y]
            let lineLength = nextLine.getTrimmedLength ()
            if !nextLine.isWrapped && lineLength <= newCols {
                continue
            }

            // Gather wrapped lines and adjust y to be the starting line
            var wrappedLines : [BufferLine] = []
            wrappedLines.append (nextLine)
            while nextLine.isWrapped && y > 0 {
                y -= 1
                nextLine = lines [y]
                wrappedLines.insert (nextLine, at: 0)
            }

            // If these lines contain the cursor don't touch them, the program will handle fixing up
            // wrapped lines with the cursor
            let absoluteY = yBase + self.y

            if absoluteY >= y && absoluteY < y + wrappedLines.count {
                continue
            }

            let lastLineLength = wrappedLines [wrappedLines.count - 1].getTrimmedLength ()
            let destLineLengths = getNewLineLengths (wrappedLines: wrappedLines, oldCols: oldCols, newCols: newCols)
            let linesToAdd = destLineLengths.count - wrappedLines.count

            var trimmedLines: Int
            if yBase == 0 && self.y != lines.count - 1 {
                // If the top section of the buffer is not yet filled
                trimmedLines = max (0, self.y - lines.maxLength + linesToAdd)
            } else {
                trimmedLines = max (0, lines.count - lines.maxLength + linesToAdd)
            }

            // Add the new lines
            var newLines : [BufferLine] = []
            if linesToAdd > 0 {
                for _ in 0..<linesToAdd {
                    let newLine = getBlankLine (attribute: CharData.defaultAttr, isWrapped: true)
                    newLines.append (newLine)
                }
            }

            if newLines.count > 0 {
                toInsert.append (InsertionSet (lines: newLines, start: y + wrappedLines.count + countToInsert, isNull: false))
                
                countToInsert += newLines.count
            }
            for l in newLines {
                wrappedLines.append (l)
            }

            // Copy buffer data to new locations, this needs to happen backwards to do in-place
            var destLineIndex = destLineLengths.count - 1 // Math.floor(cellsNeeded / newCols)
            var destCol = destLineLengths [destLineIndex] // cellsNeeded % newCols;
            if destCol == 0 {
                destLineIndex -= 1
                destCol = destLineLengths [destLineIndex]
            }

            var srcLineIndex = wrappedLines.count - linesToAdd - 1
            var srcCol = lastLineLength
            while srcLineIndex >= 0 {
                let cellsToCopy = min (srcCol, destCol)
                wrappedLines [destLineIndex].copyFrom (wrappedLines [srcLineIndex], srcCol: srcCol - cellsToCopy, dstCol: destCol - cellsToCopy, len: cellsToCopy)
                destCol -= cellsToCopy
                if destCol == 0 {
                    destLineIndex -= 1
                    if destLineIndex >= 0 {
                        destCol = destLineLengths [destLineIndex]
                    }
                }

                srcCol -= cellsToCopy
                if srcCol == 0 {
                    srcLineIndex -= 1
                    let wrappedLinesIndex = max (srcLineIndex, 0)
                    srcCol = getWrappedLineTrimmedLength (wrappedLines, wrappedLinesIndex, oldCols)
                }
            }

            // Null out the end of the line ends if a wide character wrapped to the following line
            for i in 0..<wrappedLines.count {
                if destLineLengths [i] < newCols {
                    wrappedLines [i] [destLineLengths [i]] = CharData.Null
                }
            }

            // Adjust viewport as needed
            var viewportAdjustments = linesToAdd - trimmedLines
            while viewportAdjustments > 0 {
                viewportAdjustments -= 1
                if yBase == 0 {
                    if self.y < newRows - 1 {
                        self.y += 1
                        lines.pop ()
                    } else {
                        yBase += 1
                        yDisp += 1
                    }
                } else {
                    // Ensure ybase does not exceed its maximum value
                    if yBase < min (lines.maxLength, lines.count + countToInsert) - newRows {
                        if yBase == yDisp {
                            yDisp += 1
                        }

                        yBase += 1
                    }
                }
            }

            savedY = min (savedY + linesToAdd, yBase + newRows - 1)
        }

        rearrange (toInsert, countToInsert)
    }

    func rearrange (_ toInsert: [InsertionSet], _ countToInsert: Int)
    {
        // Rearrange lines in the buffer if there are any insertions, this is done at the end rather
        // than earlier so that it's a single O(n) pass through the buffer, instead of O(n^2) from many
        // costly calls to CircularList.splice.
        if toInsert.count > 0 {
            // Record buffer insert events and then play them back backwards so that the indexes are
            // correct
            // let insertEvents : [Int] = []

            // Record original lines so they don't get overridden when we rearrange the list
            let originalLines = CircularList<BufferLine> (maxLength: lines.maxLength)
            for i in 0..<lines.count {
                originalLines.push (lines [i])
            }

            let originalLinesLength = lines.count

            var originalLineIndex = originalLinesLength - 1
            var nextToInsertIndex = 0
            var nextToInsert = toInsert [nextToInsertIndex]
            lines.count = min (lines.maxLength, lines.count + countToInsert)
        
            var countInsertedSoFar = 0
            var i = min (lines.maxLength - 1, originalLinesLength + countToInsert - 1)
            while i >= 0 {
                defer { i = i-1 }
                if !nextToInsert.isNull && nextToInsert.start > originalLineIndex + countInsertedSoFar {
                        // Insert extra lines here, adjusting i as needed
                    for nexti in (0..<nextToInsert.lines.count).reversed() {
                        if i < 0 {
                            // if we reflow and the content has to be scrolled back past the beginning
                            // of the buffer then we end up loosing those lines
                            break
                        }
                        lines [i] = nextToInsert.lines [nexti]
                        i -= 1
                    }

                    i += 1

                    countInsertedSoFar += nextToInsert.lines.count
                    if nextToInsertIndex < toInsert.count - 1 {
                        nextToInsertIndex += 1
                       nextToInsert = toInsert [nextToInsertIndex]
                    } else {
                        nextToInsert = InsertionSet.Null ()
                    }
                } else {
                    lines [i] = originalLines [originalLineIndex]
                    originalLineIndex -= 1
                }
            }
        }
    }
    
    func reflow (_ newCols: Int, _ newRows: Int)
    {
        if cols == newCols {
            return
        }
        // iterate through rows, ignore the last one as it cannot be wrapped

        if newCols > cols {
            reflowWider (cols, rows, newCols, newRows)
        } else {
            reflowNarrower (cols, rows, newCols, newRows)
        }
    }
    
    static var n = 0
    
    func dump ()
    {
        var str = ""
        str += "xDisp=\(xDisp), yDisp=\(yDisp), xBase=\(xBase), yBase=\(yBase)\n"
        str += "scrollTop=\(scrollTop) scrollBottom=\(scrollBottom)\n"
        str += "count=\(lines.count) maxLength=\(lines.maxLength)\n"
        for i in 0..<_lines.array.count {
            var txt: String
            if let r = _lines.array[i] {
                txt = r.debugDescription.replacingOccurrences(of: "\u{0}", with: " ")
            } else {
                txt = "<empty>"
            }
            let flag = i >= yDisp ? ">>" : "  "
            let istr = String (format: "%03d", i)
            let cstr = String (format: "%03d", _lines.getCyclicIndex(i))
            str += "[\(istr):\(cstr)]\(flag)\(txt)\n"
        }
        do {
            try str.write(to: URL.init (fileURLWithPath: "/Users/miguel/Downloads/Logs/dump-\(Buffer.n)"), atomically: false, encoding: .utf8)
        } catch {
            print ("oops")
        }
        Buffer.n += 1
    }

    func dumpConsole ()
    {
        let debugBuffer = self
        for y in 0..<debugBuffer._lines.maxLength {
            let flag = y == debugBuffer.yDisp ? "D" : " "
            let yb   = y == debugBuffer.yBase ? "B" : " "
            let istr = String (format: "%03d", y)
            let cstr = String (format: "%03d", debugBuffer._lines.getCyclicIndex(y))
        
            print ("[\(istr):\(cstr)]\(flag)\(yb) \(debugBuffer._lines.array [y].debugDescription)")
        }
    }
}
