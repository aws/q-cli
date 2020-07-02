//
//  AppleTerminalView.swift
//
// Shared code for UIKit and Appkit for the terminal view
//
//  Created by Miguel de Icaza on 4/21/20.
//
#if os(macOS) || os(iOS)
import Foundation
import CoreGraphics
import CoreText

#if os(iOS)
import UIKit
typealias TTColor = UIColor
typealias TTFont = UIFont
typealias TTRect = CGRect
typealias TTBezierPath = UIBezierPath
#endif

#if os(macOS)
import AppKit
typealias TTColor = NSColor
typealias TTFont = NSFont
typealias TTRect = CGRect
typealias TTBezierPath = NSBezierPath
#endif

public typealias TTImage = CGImage


extension TerminalView {
    typealias CellDimension = CGSize
    
    func resetCaches ()
    {
        self.attributes = [:]
        self.urlAttributes = [:]
        self.colors = Array(repeating: nil, count: 256)
        self.trueColors = [:]
    }
    
    // This is invoked when the font changes to recompute state
    func resetFont()
    {
        resetCaches()
        self.cellDimension = computeFontDimensions ()
        let newCols = Int(frame.width / cellDimension.width)
        let newRows = Int(frame.height / cellDimension.height)
        resize(cols: newCols, rows: newRows)
        updateCaretView()
    }
    
    func updateCaretView ()
    {
        caretView.frame.size = CGSize(width: cellDimension.width, height: cellDimension.height)
    }
    
    func setupOptions(width: CGFloat, height: CGFloat)
    {
        resetCaches ()
        // Calculation assume that all glyphs in the font have the same advancement.
        // Get the ascent + descent + leading from the font, already scaled for the font's size
        self.cellDimension = computeFontDimensions ()
        
        let terminalOptions = TerminalOptions(cols: Int(width / cellDimension.width),
                                              rows: Int(height / cellDimension.height))
        
        if terminal == nil {
            terminal = Terminal(delegate: self, options: terminalOptions)
        } else {
            terminal.options = terminalOptions
            terminal.setup(isReset: false)
        }
        terminal.backgroundColor = Color.defaultBackground
        terminal.foregroundColor = Color.defaultForeground
        attrStrBuffer = CircularList<NSAttributedString> (maxLength: terminal.buffer.lines.maxLength)
        attrStrBuffer.makeEmpty = makeEmptyLine
        fullBufferUpdate(terminal: terminal)
        
        selection = SelectionService(terminal: terminal)
        
        // Install carret view
        if caretView == nil {
            caretView = CaretView(frame: CGRect(origin: .zero, size: CGSize(width: cellDimension.width, height: cellDimension.height)))
            addSubview(caretView)
        } else {
            updateCaretView ()
        }
        
        search = SearchService (terminal: terminal)
        
        #if os(macOS)
        needsDisplay = true
        #else
        setNeedsDisplay(frame)
        #endif
    }

    /// Returns the underlying terminal emulator that the `TerminalView` is a view for
    public func getTerminal () -> Terminal
    {
        return terminal
    }
    
    //
    // Updates the contents of the NSAttributedString buffer from the contents of the terminal.buffer character array
    //
    func fullBufferUpdate (terminal: Terminal)
    {
        if terminal.buffer.lines.maxLength > attrStrBuffer.maxLength {
            attrStrBuffer.maxLength = terminal.buffer.lines.maxLength
        }
        
        let cols = terminal.cols
        for row in (terminal.buffer.yDisp)...(terminal.rows + terminal.buffer.yDisp) {
            attrStrBuffer [row] = buildAttributedString (row: row, line: terminal.buffer.lines [row], cols: cols, prefix: "")
        }
        attrStrBuffer.count = terminal.rows
    }
    
    /// Update selection attributes without rebuilding lines
    func updateSelectionInBuffer (terminal: Terminal)
    {
        if terminal.buffer.lines.maxLength > attrStrBuffer.maxLength {
            attrStrBuffer.maxLength = terminal.buffer.lines.maxLength
        }
        
        #if os(macOS)
        // This does not compile on iOS, due to
        // this not existing: attributedString.attributeKeys
        
        let cols = terminal.cols
        for row in (terminal.buffer.yDisp)...(terminal.rows + terminal.buffer.yDisp) {
            let attributedString = attrStrBuffer [row]
            
            if selection.hasSelectionRange == false {
                if attributedString.attributeKeys.contains(NSAttributedString.Key.selectionBackgroundColor.rawValue) {
                    let updatedString = NSMutableAttributedString(attributedString: attributedString)
                    updatedString.removeAttribute(.selectionBackgroundColor)
                    attrStrBuffer [row] = updatedString
                }
            }
            
            if selection.hasSelectionRange == true {
                if !attributedString.attributeKeys.contains(NSAttributedString.Key.selectionBackgroundColor.rawValue) {
                    let updatedString = NSMutableAttributedString(attributedString: attributedString)
                    updatedString.removeAttribute(.selectionBackgroundColor)
                    updateSelectionAttributesIfNeeded(attributedLine: updatedString, row: row, cols: cols)
                    attrStrBuffer [row] = updatedString
                }
            }
        }
        #endif
    }
    
    func makeEmptyLine (_ index: Int) -> NSAttributedString
    {
        let line = terminal.buffer.lines [index]
        return buildAttributedString (row: index, line: line, cols: terminal.cols, prefix: "")
    }
    
    // Computes the font dimensions once font.normal has been set
    func computeFontDimensions () -> CellDimension
    {
        let lineAscent = CTFontGetAscent (fontSet.normal)
        let lineDescent = CTFontGetDescent (fontSet.normal)
        let lineLeading = CTFontGetLeading (fontSet.normal)
        let cellHeight = ceil(lineAscent + lineDescent + lineLeading)
        #if os(macOS)
        let cellWidth = fontSet.normal.maximumAdvancement.width
        #else
        let fontAttributes = [NSAttributedString.Key.font: fontSet.normal]
        let cellWidth = "W".size(withAttributes: fontAttributes).width
        #endif
        return CellDimension(width: cellWidth, height: cellHeight)
    }
    
    func mapColor (color: Attribute.Color, isFg: Bool, isBold: Bool) -> TTColor
    {
        switch color {
        case .defaultColor:
            if isFg {
                return nativeForegroundColor
            } else {
                return nativeBackgroundColor
            }
        case .defaultInvertedColor:
            if isFg {
                return nativeForegroundColor.inverseColor()
            } else {
                return nativeBackgroundColor.inverseColor()
            }
        case .ansi256(let ansi):
            let midx = Int (ansi) + ((isBold && ansi < 248) ? 8: 0);
            if let c = colors [midx] {
                return c
            }
            let tcolor = terminal.ansiColors [midx]
            let newColor = TTColor.make (color: tcolor)
            colors [midx] = newColor
            return newColor
        case .trueColor(let r, let g, let b):
            if let tc = trueColors [color] {
                return tc
            }
            let newColor = TTColor.make(red: CGFloat (r) / 65535.0,
                                        green: CGFloat (g) / 65535.0,
                                        blue: CGFloat (b) / 65535.0,
                                        alpha: 1.0)
            
            trueColors [color] = newColor
            return newColor
        }
    }

    // Clears the cached state for colors and triggers a full display
    func colorsChanged ()
    {
        urlAttributes = [:]
        attributes = [:]
        terminal.updateFullScreen ()
    }
    
    /// Installs the new colors as the default colors and recomputes the
    /// current and ansi palette.   This installs both the colors into the terminal
    /// engine and updates the UI accordingly.
    /// 
    /// - Parameter colors: this should be an array of 16 values that correspond to the 16 ANSI colors,
    /// if the array does not contain 16 elements, it will not do anything
    public func installColors (_ colors: [Color])
    {
        terminal.installPalette(colors: colors)
        self.colorsChanged()
    }
    
    public func colorChanged (source: Terminal, idx: Int?)
    {
        if let index = idx {
            colors [index] = nil
        } else {
            colors = Array(repeating: nil, count: 256)
        }
        colorsChanged ()
    }

    public func setBackgroundColor(source: Terminal, color: Color) {
        // Can not implement this until I change the color to not be this struct
        nativeBackgroundColor = TTColor.make (color: color)
        colorsChanged()
    }
    
    public   
    func setForegroundColor(source: Terminal, color: Color) {
        nativeForegroundColor = TTColor.make (color: color)
        colorsChanged()
    }
    
    //
    // Given a vt100 attribute, return the NSAttributedString attributes used to render it
    //
    func getAttributes (_ attribute: Attribute, withUrl: Bool) -> [NSAttributedString.Key:Any]?
    {
        let flags = attribute.style
        var bg = attribute.bg
        var fg = attribute.fg
        
        if flags.contains (.inverse) {
            swap (&bg, &fg)
            
            if fg == .defaultColor {
                fg = .defaultInvertedColor
            }
            if bg == .defaultColor {
                bg = .defaultInvertedColor
            }
        }
        
        if let result = withUrl ? urlAttributes [attribute] : attributes [attribute] {
            return result
        }
        
        var tf: TTFont
        let isBold = flags.contains(.bold)
        if isBold {
            if flags.contains (.italic) {
                tf = fontSet.boldItalic
            } else {
                tf = fontSet.bold
            }
        } else if flags.contains (.italic) {
            tf = fontSet.italic
        } else {
            tf = fontSet.normal
        }
        
        let fgColor = mapColor (color: fg, isFg: true, isBold: isBold)
        var nsattr: [NSAttributedString.Key:Any] = [
            .font: tf,
            .foregroundColor: fgColor,
            .backgroundColor: mapColor(color: bg, isFg: false, isBold: false)
        ]
        if flags.contains (.underline) {
            nsattr [.underlineColor] = fgColor
            nsattr [.underlineStyle] = NSUnderlineStyle.single.rawValue
        }
        if flags.contains (.crossedOut) {
            nsattr [.strikethroughColor] = fgColor
            nsattr [.strikethroughStyle] = NSUnderlineStyle.single.rawValue
        }
        if withUrl {
            nsattr [.underlineStyle] = NSUnderlineStyle.single.rawValue | NSUnderlineStyle.patternDash.rawValue
            nsattr [.underlineColor] = fgColor
            
            // Add to cache
            urlAttributes [attribute] = nsattr
        } else {
            // Just add to cache
            attributes [attribute] = nsattr
        }
        return nsattr
    }
    
    //
    // Given a line of text with attributes, returns the NSAttributedString, suitable to be drawn
    //
    func buildAttributedString (row: Int, line: BufferLine, cols: Int, prefix: String = "") -> NSAttributedString
    {
        let res = NSMutableAttributedString ()
        var attr = Attribute.empty
        var hasUrl = false
        
        var str = prefix
        for col in 0..<cols {
            let ch: CharData = line[col]
            if col == 0 {
                attr = ch.attribute
                hasUrl = ch.hasUrl
            } else {
                let chhas = ch.hasUrl
                if attr != ch.attribute || chhas != hasUrl {
                    res.append(NSAttributedString (string: str, attributes: getAttributes (attr, withUrl: hasUrl)))
                    str = ""
                    attr = ch.attribute
                    hasUrl = chhas
                }
            }
            str.append(ch.code == 0 ? " " : ch.getCharacter ())
        }
        res.append (NSAttributedString(string: str, attributes: getAttributes(attr, withUrl: hasUrl)))
        updateSelectionAttributesIfNeeded(attributedLine: res, row: row, cols: cols)
        // This gives us a large chunk of our performance back, from 7.5 to 5.5 seconds on
        // time for x in 1 2 3 4 5 6; do cat UTF-8-demo.txt; done
        //res.fixAttributes(in: NSRange(location: 0, length: res.length))
        return res
    }
    
    /// Apply selection attributes
    /// TODO: Optimize the logic below
    func updateSelectionAttributesIfNeeded(attributedLine attributedString: NSMutableAttributedString, row: Int, cols: Int) {
        guard let selection = self.selection, selection.active else {
            attributedString.removeAttribute(.selectionBackgroundColor)
            return
        }
        
        let startRow = selection.start.row
        let endRow = selection.end.row
        
        let startCol = selection.start.col
        let endCol = selection.end.col
        
        var selectionRange: NSRange = .empty
        
        // single row
        if endRow == startRow && startRow == row {
            if startCol < endCol {
                selectionRange = NSRange(location: startCol, length: endCol - startCol)
            } else if startCol > endCol {
                selectionRange = NSRange(location: endCol, length: startCol - endCol)
            }
        } else if endRow > startRow {
            // first row
            if startRow == row && endRow > row {
                selectionRange = NSRange(location: startCol, length: cols - startCol)
            }
            
            // in between
            if startRow < row && endRow > row {
                selectionRange = NSRange(location: 0, length: cols)
            }
            
            // last row
            if startRow < row && endRow == row {
                selectionRange = NSRange(location: 0, length: endCol)
            }
        } else if endRow < startRow {
            
            // first row
            if endRow == row && startRow > row {
                selectionRange = NSRange(location: endCol, length: cols - endCol)
            }
            
            // in between
            if startRow > row && endRow < row {
                selectionRange = NSRange(location: 0, length: cols)
            }
            
            // last row
            if endRow < row && startRow == row {
                selectionRange = NSRange(location: 0, length: startCol)
            }
        }
        
        if selectionRange != .empty {
            attributedString.addAttribute(.selectionBackgroundColor, value: selectedTextBackgroundColor, range: selectionRange)
        }
    }

    func drawRunAttributes(_ attributes: [NSAttributedString.Key : Any], glyphPositions positions: [CGPoint], in currentContext: CGContext) {
        currentContext.saveGState()

        let scale = backingScaleFactor()

        if attributes.keys.contains(.underlineStyle) {
            // draw underline at font.normal.underlinePosition baseline
            let underlineStyle = NSUnderlineStyle(rawValue: attributes[.underlineStyle] as? NSUnderlineStyle.RawValue ?? 0)
            let underlineColor = attributes[.underlineColor] as? TTColor ?? nativeForegroundColor
            let underlinePosition = fontSet.underlinePosition ()

            // draw line at the baseline
            currentContext.setShouldAntialias(false)
            currentContext.setStrokeColor(underlineColor.cgColor)

            let underlineThickness = max(round(scale * fontSet.underlineThickness ()) / scale, 0.5)
            for p in positions {
                switch underlineStyle {
                case let style where style.contains(.single):
                    let path = TTBezierPath()
                    path.move(to: p.applying(.init(translationX: 0, y: underlinePosition)))
                    path.addLine(to: p.applying(.init(translationX: ceil(cellDimension.width), y: underlinePosition)))
                    path.lineWidth = underlineThickness
                    switch underlineStyle {
                    case let pattern where pattern.contains(.patternDash):
                        let pattern: [CGFloat] = [2.0]
                        path.setLineDash(pattern, count: pattern.count, phase: 0)
                    default:
                        break
                    }
                    path.stroke()
                case let style where style.contains(.double):
                    let path1 = TTBezierPath()
                    path1.move(to: p.applying(.init(translationX: 0, y: underlinePosition)))
                    path1.addLine(to: p.applying(.init(translationX: ceil(cellDimension.width), y: underlinePosition)))
                    path1.lineWidth = underlineThickness

                    let path2 = TTBezierPath()
                    path2.move(to: p.applying(.init(translationX: 0, y: underlinePosition - underlineThickness - 1)))
                    path2.addLine(to: p.applying(.init(translationX: ceil(cellDimension.width), y: underlinePosition - underlineThickness - 1)))
                    path2.lineWidth = underlineThickness

                    switch underlineStyle {
                    case let pattern where pattern.contains(.patternDash):
                        let pattern: [CGFloat] = [2.0]
                        path1.setLineDash(pattern, count: pattern.count, phase: 0)
                        path2.setLineDash(pattern, count: pattern.count, phase: 0)
                    default:
                        break
                    }
                    path1.stroke()
                    path2.stroke()
                default:
                    preconditionFailure("Unsupported underline style.")
                    break
                }
            }
        }
        currentContext.restoreGState()
    }

    // TODO: this should not render any lines outside the dirtyRect
    func drawTerminalContents (dirtyRect: TTRect, context: CGContext)
    {
        let lineDescent = CTFontGetDescent(fontSet.normal)
        let lineLeading = CTFontGetLeading(fontSet.normal)

        // draw lines
        for row in terminal.buffer.yDisp..<terminal.rows + terminal.buffer.yDisp {
            let lineOffset = cellDimension.height * (CGFloat(row - terminal.buffer.yDisp + 1))
            let lineOrigin = CGPoint(x: 0, y: frame.height - lineOffset)
            let ctline = CTLineCreateWithAttributedString(attrStrBuffer [row])

            var col = 0
            for run in CTLineGetGlyphRuns(ctline) as? [CTRun] ?? [] {
                let runGlyphsCount = CTRunGetGlyphCount(run)
                let runAttributes = CTRunGetAttributes(run) as? [NSAttributedString.Key: Any] ?? [:]
                let runFont = runAttributes[.font] as! TTFont

                let runGlyphs = [CGGlyph](unsafeUninitializedCapacity: runGlyphsCount) { (bufferPointer, count) in
                    CTRunGetGlyphs(run, CFRange(), bufferPointer.baseAddress!)
                    count = runGlyphsCount
                }

                var positions = runGlyphs.enumerated().map { (i: Int, glyph: CGGlyph) -> CGPoint in
                    CGPoint(x: lineOrigin.x + (cellDimension.width * CGFloat(col + i)), y: lineOrigin.y + ceil(lineLeading + lineDescent))
                }

                var backgroundColor: TTColor?
                if runAttributes.keys.contains(.selectionBackgroundColor) {
                    backgroundColor = runAttributes[.selectionBackgroundColor] as? TTColor
                } else if runAttributes.keys.contains(.backgroundColor) {
                    backgroundColor = runAttributes[.backgroundColor] as? TTColor
                }

                if let backgroundColor = backgroundColor {
                    context.saveGState ()

                    context.setShouldAntialias (false)
                    context.setLineCap (.square)
                    context.setLineWidth(0)
                    context.setFillColor(backgroundColor.cgColor)

                    let transform = CGAffineTransform (translationX: positions[0].x, y: 0)
                    let rect = CGRect (origin: lineOrigin, size: CGSize (width: CGFloat (cellDimension.width * CGFloat(runGlyphsCount)), height: cellDimension.height))
                    #if os(macOS)
                    rect.applying(transform).fill(using: .destinationOver)
                    #else
                    context.fill(rect.applying(transform))
                    #endif
                    context.restoreGState()
                }

                nativeForegroundColor.set()

                if runAttributes.keys.contains(.foregroundColor) {
                    let color = runAttributes[.foregroundColor] as! TTColor
                    let cgColor = color.cgColor
                    if let colorSpace = cgColor.colorSpace {
                        context.setFillColorSpace(colorSpace)
                    }
                    context.setFillColor(cgColor)
                }

                CTFontDrawGlyphs(runFont, runGlyphs, &positions, positions.count, context)

                // Draw other attributes
                drawRunAttributes(runAttributes, glyphPositions: positions, in: context)

                col += runGlyphsCount
            }

            // set caret position
            if terminal.buffer.y == row - terminal.buffer.yDisp {
                updateCursorPosition()
            }
        }
    }
    
    /// Update visible area
    func updateDisplay (notifyAccessibility: Bool)
    {
        updateCursorPosition()
        guard let (rowStart, rowEnd) = terminal.getUpdateRange () else {
            return
        }
        
        terminal.clearUpdateRange ()
        
        let cols = terminal.cols
        let tb = terminal.buffer
        
        for row in (rowStart + tb.yDisp)...(rowEnd + tb.yDisp) {
            let line = terminal.buffer.lines [row]
            
            attrStrBuffer [row] = buildAttributedString (row: row, line: line, cols: cols, prefix: "")
        }
        
        #if os(macOS)
        let baseLine = frame.height
        var region = CGRect (x: 0,
                             y: baseLine - (cellDimension.height + CGFloat(rowEnd) * cellDimension.height),
                             width: frame.width,
                             height: CGFloat(rowEnd-rowStart + 1) * cellDimension.height)
        
        // If we are the last line, we should also queue a refresh for the "remaining" bits at the
        // end which can be redrawn by large unicode
        if rowEnd == terminal.rows - 1 {
            let oh = region.height
            let oy = region.origin.y
            region = CGRect (x: 0, y: 0, width: frame.width, height: oh + oy)
        }
        //print ("Region: \(region)")
        setNeedsDisplay(region)
        #else
        // TODO iOS: need to update the code above, but will do that when I get some real
        // life data being fed into it.
        setNeedsDisplay(bounds)
        #endif
        
        pendingDisplay = false
        updateDebugDisplay ()
        
        if (notifyAccessibility) {
            accessibility.invalidate ()
            #if os(macOS)
            NSAccessibility.post (element: self, notification: .valueChanged)
            NSAccessibility.post (element: self, notification: .selectedTextChanged)
            #endif
        }
    }
    
    func updateCursorPosition()
    {
        //let lineOrigin = CGPoint(x: 0, y: frame.height - (cellDimension.height * (CGFloat(terminal.buffer.y - terminal.buffer.yDisp + 1))))
        //caretView.frame.origin = CGPoint(x: lineOrigin.x + (cellDimension.width * CGFloat(terminal.buffer.x)), y: lineOrigin.y)
        let buffer = terminal.buffer
        let vy = buffer.yBase + buffer.y
        
        if vy >= buffer.yDisp + buffer.rows {
            caretView.removeFromSuperview()
            return
        } else if terminal.cursorHidden == false {
            addSubview(caretView)
        }
        
        #if os(iOS)
        let offset = (cellDimension.height * (CGFloat(buffer.y-(buffer.yDisp-buffer.yBase))))
        let lineOrigin = CGPoint(x: 0, y: offset)
        #else
        let offset = (cellDimension.height * (CGFloat(buffer.y-(buffer.yDisp-buffer.yBase)+1)))
        let lineOrigin = CGPoint(x: 0, y: frame.height - offset)
        #endif
        caretView.frame.origin = CGPoint(x: lineOrigin.x + (cellDimension.width * CGFloat(buffer.x)), y: lineOrigin.y)
    }
    
    // Does not use a default argument and merge, because it is called back
    func updateDisplay ()
    {
        updateDisplay (notifyAccessibility: true)
        updateDebugDisplay()
        pendingDisplay = false
    }
    
    //
    // The code below is intended to not repaint too often, which can produce flicker, for example
    // when the user refreshes the display, and this repains the screen, as dispatch delivers data
    // in blocks of 1024 bytes, which is not enough to cover the whole screen, so this delays
    // the update for a 1/600th of a second.
    //
    // It is also cheap, so should be called when new data has been posted or received.
    func queuePendingDisplay ()
    {
        // throttle
        if !pendingDisplay {
            let fps60 = 16670000
            // let fps30 = 16670000*2
            let fpsDelay = fps60
            pendingDisplay = true
            DispatchQueue.main.asyncAfter(
                deadline: DispatchTime (uptimeNanoseconds: DispatchTime.now().uptimeNanoseconds + UInt64 (fpsDelay)),
                execute: updateDisplay)
        }
    }
    
    ///
    /// This takes a string returned by events (NSEvent or UIKey) as the 'charactersIngoringModifiers'
    /// and returns the control-version of that, and only applies to a handful of characters
    ///
    func applyControlToEventCharacters (_ ch: String) -> [UInt8]
    {
        let arr = [UInt8](ch.utf8)
        if arr.count == 1 {
            let ch = Character (UnicodeScalar (arr [0]))
            var value: UInt8
            switch ch {
            case "A"..."Z":
                value = (ch.asciiValue! - 0x40 /* - 'A' + 1 */)
            case "a"..."z":
                value = (ch.asciiValue! - 0x60 /* - 'a' + 1 */)
            case "\\":
                value = 0x1c
            case "_":
                value = 0x1f
            case "]":
                value = 0x1d
            case "[":
                value = 0x1b
            case "^":
                value = 0x1e
            case " ":
                value = 0
            default:
                return []
            }
            return [value]
        }
        return []
    }
    /**
     * Returns the thumb size in proportion to the visible content of the entire content, alternate buffers are not scrollable, so this returns 0
     */
    public var scrollThumbsize: CGFloat {
        get {
            if terminal.buffers!.isAlternateBuffer {
                return 0
            }
            
            // the thumb size is the proportion of the visible content of the
            // entire content but don't make it too small
            return max (CGFloat (terminal.rows) / CGFloat (terminal.buffer.lines.count), 0.01)
        }
    }
    
    /**
     * Gets a value indicating the relative position of the terminal viewport
     */
    public var scrollPosition: Double {
        get {
            if terminal.buffers.isAlternateBuffer || terminal.buffer.yDisp <= 0 {
                return 0
            }
            
            let maxScrollback = terminal.buffer.lines.count - terminal.rows
            if terminal.buffer.yDisp >= maxScrollback {
                return 1
            }
            
            return Double (terminal.buffer.yDisp) / Double (maxScrollback)
        }
    }
    
    /// <summary>
    /// Gets a value indicating whether or not the user can scroll the terminal contents
    /// </summary>
    public var canScroll: Bool {
        get {
            return !terminal.buffers.isAlternateBuffer &&
                terminal.buffer.hasScrollback &&
                terminal.buffer.lines.count > terminal.rows
        }
    }
    
    public func scroll (toPosition: Double)
    {
        userScrolling = true
        let oldPosition = terminal.buffer.yDisp
        
        let maxScrollback = terminal.buffer.lines.count - terminal.rows
        print ("maxScrollBack: \(maxScrollback)")
        var newScrollPosition = Int (Double (maxScrollback) * toPosition)
        
        if newScrollPosition < 0 {
            newScrollPosition = 0
        }
        if newScrollPosition > maxScrollback {
            newScrollPosition = maxScrollback
        }
        print ("newScrollpsitin: \(newScrollPosition)")
        
        if newScrollPosition != oldPosition {
            scrollTo(row: newScrollPosition)
        }
        userScrolling = false
    }
    
    func ensureCaretIsVisible ()
    {
        let realCaret = terminal.buffer.y + terminal.buffer.yBase
        let viewportEnd = terminal.buffer.yDisp + terminal.rows
        
        if realCaret >= viewportEnd || realCaret < terminal.buffer.yDisp {
            scrollTo (row: terminal.buffer.yBase)
        }
    }
    
    func scrollTo (row: Int, notifyAccessibility: Bool = true)
    {
        if row != terminal.buffer.yDisp {
            
            terminal.buffer.yDisp = row
            
            // tell the terminal we want to refresh all the rows
            terminal.refresh (startRow: 0, endRow: terminal.rows)
            
            // do the display update
            updateDisplay (notifyAccessibility: notifyAccessibility)
            //selectionView.notifyScrolled(source: terminal)
            terminalDelegate?.scrolled (source: self, position: scrollPosition)
            updateScroller()
            setNeedsDisplay(frame)
        }
    }
    
    /// Scrolls the content of the terminal one page up
    public func pageUp()
    {
        scrollUp (lines: terminal.rows)
    }
    
    /// Scrolls the content of the terminal one page down
    public func pageDown ()
    {
        scrollDown (lines: terminal.rows)
    }
    
    /// Scrolls up the content of the terminal the specified number of lines
    public func scrollUp (lines: Int)
    {
        let newPosition = max (terminal.buffer.yDisp - lines, 0)
        scrollTo (row: newPosition)
    }
    
    /// Scrolls down the content of the terminal the specified number of lines
    public func scrollDown (lines: Int)
    {
        let newPosition = max (0, min (terminal.buffer.yDisp + lines, terminal.buffer.lines.count - terminal.rows))
        scrollTo (row: newPosition)
    }
      
    // Sends data to the terminal emulator for interpretation
    public func feed (byteArray: ArraySlice<UInt8>)
    {
        search.invalidate ()
        terminal.feed (buffer: byteArray)
        queuePendingDisplay ()
    }
    
    // Sends data to the terminal emulator for interpretation
    public func feed (text: String)
    {
        search.invalidate ()
        terminal.feed (text: text)
        queuePendingDisplay ()
    }
         
    /**
     * Triggers a resize of the underlying terminal to the desired columsn and rows
     */
    public func resize (cols: Int, rows: Int)
    {
        terminal.resize (cols: cols, rows: rows)
        sizeChanged (source: terminal)
        terminal.resetToInitialState()
    }
    
    /**
     * Sends the specified slice of byte arrays to the program running under the terminal emulator
     * - Parameter data: the slice of an array to send to the client
     */
    public func send(data: ArraySlice<UInt8>)
    {
        ensureCaretIsVisible ()
        terminalDelegate?.send (source: self, data: data)
    }
    
    /**
     * Sends the specified string encoded at utf8 to the program running under the terminal emulator
     * - Parameter txt: the string to send to the client
     */
    public func send (txt: String) {
        let array = [UInt8] (txt.utf8)
        send (data: array[...])
    }
    
    /**
     * Sends the specified array of bytes to the program running under the terminal emulator
     * - Parameter bytes: the bytes to send to the client
     */
    public func send (_ bytes: [UInt8]) {
        send (data: (bytes)[...])
    }
    
    open func showCursor(source: Terminal) {
        if caretView.superview == nil {
            addSubview(caretView)
        }
    }

    open func hideCursor(source: Terminal) {
        caretView.removeFromSuperview()
    }
    
    open func bell(source: Terminal) {
        terminalDelegate?.bell (source: self)
    }

    func sendKeyUp ()
    {
        send (terminal.applicationCursor ? EscapeSequences.MoveUpApp : EscapeSequences.MoveUpNormal)
    }
    
    func sendKeyDown ()
    {
        send (terminal.applicationCursor ? EscapeSequences.MoveDownApp : EscapeSequences.MoveDownNormal)
    }
    
    func sendKeyLeft()
    {
        send (terminal.applicationCursor ? EscapeSequences.MoveLeftApp : EscapeSequences.MoveLeftNormal)
    }
    
    func sendKeyRight ()
    {
        send (terminal.applicationCursor ? EscapeSequences.MoveRightApp : EscapeSequences.MoveRightNormal)
    }
}
#endif
