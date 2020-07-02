//
//  MacTerminalView.swift
//
// This is the AppKit version of the TerminalView and holds the state
// variables in the `TerminalView` class, but as much of the terminal
// implementation details live in the Apple/AppleTerminalView which
// contains the shared AppKit/UIKit code
//
//  Created by Miguel de Icaza on 3/4/20.
//

#if os(OSX)
import Foundation
import AppKit
import CoreText
import CoreGraphics

/**
 * TerminalView provides an AppKit front-end to the `Terminal` termininal emulator.
 * It is up to a subclass to either wire the terminal emulator to a remote terminal
 * via some socket, to an application that wants to run with terminal emulation, or
 * wiring this up to a pseudo-terminal.
 *
 * Users are notified of interesting events in their implementation of the `TerminalViewDelegate`
 * methods - an instance must be provided to the constructor of `TerminalView`.
 *
 * Developers might want to surface UIs for `optionAsMetaKey` and `allowMouseReporting` in
 * their application.  They both default to true, but this means that Option-Letter is hijacked for
 * terminal purposes to send the sequence ESC-Letter, instead of the macOS specific character` and
 * means that when mouse-aware applications are running, they hijack the normal selection process.
 *
 * Call the `getTerminal` method to get a reference to the underlying `Terminal` that backs this
 * view.
 *
 * Use the `configureNativeColors()` to set the defaults colors for the view to match the OS
 * defaults, otherwise, this uses its own set of defaults colors.
 */
open class TerminalView: NSView, NSTextInputClient, NSUserInterfaceValidations {
    struct FontSet {
        public let normal: NSFont
        let bold: NSFont
        let italic: NSFont
        let boldItalic: NSFont
        
        static var defaultFont: NSFont {
            if #available(OSX 10.15, *)  {
                return NSFont.monospacedSystemFont(ofSize: NSFont.systemFontSize, weight: .regular)
            } else {
                return NSFont(name: "Menlo Regular", size: NSFont.systemFontSize) ?? NSFont(name: "Courier", size: NSFont.systemFontSize)!
            }
        }
        
        public init(font baseFont: NSFont, fontSize: CGFloat? = nil) {
            self.normal = baseFont
            self.bold = NSFontManager.shared.convert(baseFont, toHaveTrait: [.boldFontMask])
            self.italic = NSFontManager.shared.convert(baseFont, toHaveTrait: [.italicFontMask])
            self.boldItalic = NSFontManager.shared.convert(baseFont, toHaveTrait: [.italicFontMask, .boldFontMask])
        }

        // Expected by the shared rendering code
        func underlinePosition () -> CGFloat
        {
            return normal.underlinePosition
        }

        // Expected by the shared rendering code
        func underlineThickness () -> CGFloat
        {
            return normal.underlineThickness
        }
    }
    
    /**
     * The delegate that the TerminalView uses to interact with its hosting
     */
    public weak var terminalDelegate: TerminalViewDelegate?
    
    var accessibility: AccessibilityService = AccessibilityService()
    var search: SearchService!
    var debug: TerminalDebugView?
    var pendingDisplay: Bool = false
    
    var cellDimension: CellDimension!
    var caretView: CaretView!
    var terminal: Terminal!

    var selection: SelectionService!
    private var scroller: NSScroller!
    var attrStrBuffer: CircularList<NSAttributedString>!
    
    // Attribute dictionary, maps a console attribute (color, flags) to the corresponding dictionary
    // of attributes for an NSAttributedString
    var attributes: [Attribute: [NSAttributedString.Key:Any]] = [:]
    var urlAttributes: [Attribute: [NSAttributedString.Key:Any]] = [:]
    
    // Cache for the colors in the 0..255 range
    var colors: [NSColor?] = Array(repeating: nil, count: 256)
    var trueColors: [Attribute.Color:NSColor] = [:]
    var transparent = TTColor.transparent ()
    
    var fontSet: FontSet

    /// The font to use to render the terminal
    public var font: NSFont {
        get {
            return fontSet.normal
        }
        set {
            fontSet = FontSet (font: newValue)
            resetFont()
        }
    }
    
    public init(frame: CGRect, font: NSFont?) {
        self.fontSet = FontSet (font: font ?? FontSet.defaultFont)

        super.init (frame: frame)
        setup()
    }
    
    public override init (frame: CGRect)
    {
        self.fontSet = FontSet (font: FontSet.defaultFont)
        super.init (frame: frame)
        setup()
    }
    
    public required init? (coder: NSCoder)
    {
        self.fontSet = FontSet (font: FontSet.defaultFont)
        super.init (coder: coder)
        setup()
    }
    
    private func setup()
    {
        wantsLayer = true
        
        setupScroller()
        setupOptions()
    }
    
    func setupOptions ()
    {
        setupOptions (width: getEffectiveWidth (rect: bounds), height: bounds.height)
        layer?.backgroundColor = nativeBackgroundColor.cgColor
    }

    /// This controls whether the backspace should send ^? or ^H, the default is ^?
    public var backspaceSendsControlH: Bool = false
    
    var _nativeFg, _nativeBg: TTColor!
    var settingFg = false, settingBg = false
    /**
     * This will set the native foreground color to the specified native color (UIColor or NSColor)
     * and will have this reflected into the underlying's terminal `foregroundColor` and
     * `backgroundColor`
     */
    public var nativeForegroundColor: NSColor {
        get { _nativeFg }
        set {
            if settingFg { return }
            settingFg = true
            _nativeFg = newValue
            terminal.foregroundColor = nativeForegroundColor.getTerminalColor ()
            settingFg = false
        }
    }

    /**
     * This will set the native foreground color to the specified native color (UIColor or NSColor)
     * and will have this reflected into the underlying's terminal `foregroundColor` and
     * `backgroundColor`
     */
    public var nativeBackgroundColor: NSColor {
        get { _nativeBg }
        set {
            if settingBg { return }
            settingBg = true
            _nativeBg = newValue
            terminal.backgroundColor = nativeBackgroundColor.getTerminalColor ()
            settingBg = false
        }
    }
    
    /// Controls the color for the caret
    public var caretColor: NSColor {
        get { caretView.caretColor }
        set { caretView.caretColor = newValue }
    }
    
    var _selectedTextBackgroundColor = NSColor.selectedTextBackgroundColor
    /// The color used to render the selection
    public var selectedTextBackgroundColor: NSColor {
        get {
            return _selectedTextBackgroundColor
        }
        set {
            _selectedTextBackgroundColor = newValue
        }
    }

    func backingScaleFactor () -> CGFloat
    {
        window?.backingScaleFactor ?? NSScreen.main?.backingScaleFactor ?? 1
    }
    
    @objc
    func scrollerActivated ()
    {
        switch scroller.hitPart {
        case .decrementPage:
            pageUp()
            scroller.doubleValue =  scrollPosition
        case .incrementPage:
            pageDown()
            scroller.doubleValue =  scrollPosition
        case .knob:
            scroll(toPosition: scroller.doubleValue)
        case .knobSlot:
            print ("Scroller .knobSlot clicked")
        case .noPart:
            print ("Scroller .noPart clicked")
        case .decrementLine:
            print ("Scroller .decrementLine clicked")
        case .incrementLine:
            print ("Scroller .incrementLine clicked")
        default:
            print ("Scroller: New value introduced")
        }
    }
    
    
    func setupScroller()
    {
        let style: NSScroller.Style = .legacy
        let scrollerWidth = NSScroller.scrollerWidth(for: .regular, scrollerStyle: style)
        scroller = NSScroller(frame: NSRect(x: bounds.maxX - scrollerWidth, y: 0, width: scrollerWidth, height: bounds.height))
        scroller.autoresizingMask = [.minXMargin, .height]
        scroller.scrollerStyle = style
        scroller.knobProportion = 0.1
        scroller.isEnabled = false
        addSubview (scroller)
        scroller.action = #selector(scrollerActivated)
        scroller.target = self
    }
    
    /// This method sents the `nativeForegroundColor` and `nativeBackgroundColor`
    /// to match macOS default colors for text and its background.
    public func configureNativeColors ()
    {
        self.nativeForegroundColor = NSColor.textColor
        self.nativeBackgroundColor = NSColor.textBackgroundColor

    }
    
    open func bufferActivated(source: Terminal) {
        updateScroller ()
    }
    
    open func send(source: Terminal, data: ArraySlice<UInt8>) {
        terminalDelegate?.send (source: self, data: data)
    }
        
    /**
     * Given the current set of columns and rows returns a frame that would host this control.
     */
    open func getOptimalFrameSize () -> NSRect
    {
        return NSRect (x: 0, y: 0, width: cellDimension.width * CGFloat(terminal.cols) + scroller.frame.width, height: cellDimension.height * CGFloat(terminal.rows))
    }
    
    func getEffectiveWidth (rect: NSRect) -> CGFloat
    {
        return (rect.width-scroller.frame.width)
    }
    
    open func scrolled(source terminal: Terminal, yDisp: Int) {
        //selectionView.notifyScrolled(source: terminal)
        updateScroller()
        terminalDelegate?.scrolled(source: self, position: scrollPosition)
    }
    
    open func linefeed(source: Terminal) {
        selection.selectNone()
    }
    
    /// This vaiable controls whether mouse events are sent to the application running under the
    /// terminal if it has requested the data.   This poses a problem for selection, so users
    /// need a way of toggling this behavior.
    public var allowMouseReporting: Bool = true
        
    func updateDebugDisplay()
    {
        debug?.update()
    }
    
    func updateScroller ()
    {
        scroller.isEnabled = canScroll
        scroller.doubleValue = scrollPosition
        scroller.knobProportion = scrollThumbsize
    }
    
    var userScrolling = false

    #if false
    override open func setNeedsDisplay(_ invalidRect: NSRect) {
        print ("setNeeds: \(invalidRect)")
        super.setNeedsDisplay(invalidRect)
    }
    #endif
    
    func getCurrentGraphicsContext () -> CGContext?
    {
        NSGraphicsContext.current?.cgContext
    }
    
    override public func draw (_ dirtyRect: NSRect) {
        guard let currentContext = getCurrentGraphicsContext() else {
            return
        }
        
        drawTerminalContents (dirtyRect: dirtyRect, context: currentContext)
    }
    
    public override func cursorUpdate(with event: NSEvent)
    {
        NSCursor.iBeam.set ()
    }
    
    func makeFirstResponder ()
    {
        window?.makeFirstResponder (self)
    }

    open override var frame: NSRect {
        get {
            return super.frame
        }
        set(newValue) {
            super.frame = newValue
            
            let newRows = Int (newValue.height / cellDimension.height)
            let newCols = Int (getEffectiveWidth (rect: newValue) / cellDimension.width)
            
            if newCols != terminal.cols || newRows != terminal.rows {
                terminal.resize (cols: newCols, rows: newRows)
                fullBufferUpdate (terminal: terminal)
            }
            
            accessibility.invalidate ()
            search.invalidate ()
            
            terminalDelegate?.sizeChanged (source: self, newCols: newCols, newRows: newRows)
            needsDisplay = true
        }
    }
    
    public override func resizeSubviews(withOldSize oldSize: NSSize) {
        super.resizeSubviews(withOldSize: oldSize)
        updateScroller()
        selection.active = false
    }
    
    private var _hasFocus = false
    open var hasFocus : Bool {
        get { _hasFocus }
        set {
            _hasFocus = newValue
            caretView.focused = newValue
        }
    }

    //
    // NSTextInputClient protocol implementation
    //
    public override func becomeFirstResponder() -> Bool {
        let response = super.becomeFirstResponder()
        if response {
            hasFocus = true
        }
        return response
    }
    
    public override func resignFirstResponder() -> Bool {
        let response = super.resignFirstResponder()
        if response {
            hasFocus = false
        }
        return response
    }
    
    public override var acceptsFirstResponder: Bool {
        get {
            return true
        }
    }
    
    // Tracking object, maintained by `startTracking` and `deregisterTrackingInterest`
    var tracking: NSTrackingArea? = nil
    
    // Turns on AppKit mouse event tracking - used both by the url highlighter and the mouse move,
    // when the client application has set MouseMove.anyEvent
    //
    // Can be invoked multiple times, use the "deregisterTrackingInterest" method to turn it off
    // which will take into account both the url highlighter state (which is bound to the command
    // key being pressed) and the client requirements
    func startTracking ()
    {
        if tracking == nil {
            tracking = NSTrackingArea (rect: frame, options: [.activeAlways, .mouseMoved, .mouseEnteredAndExited], owner: self, userInfo: [:])
            addTrackingArea(tracking!)
        }
    }
    
    // Can be invoked by both the keyboard handler monitoring the command key, and the
    // mouse tracking system, only when both are off, this is turned off.
    func deregisterTrackingInterest ()
    {
        if commandActive == false && terminal.mouseMode != .anyEvent {
            if tracking != nil {
                removeTrackingArea(tracking!)
                tracking = nil
            }
        }
    }
    
    func turnOffUrlPreview ()
    {
        if commandActive {
            deregisterTrackingInterest()
            removePreviewUrl()
            commandActive = false
        }
    }
    
    // If true, the Command key has been pressed
    var commandActive = false
    
    // We monitor the flags changed to enable URL previews on mouse-hover like iTerm
    // when the Command key is pressed.
    
    public override func flagsChanged(with event: NSEvent) {
        if event.modifierFlags.contains(.command){
            commandActive = true
            startTracking()
            
            if let payload = getPayload(for: event) {
                previewUrl (payload: payload)
            }
        } else {
            turnOffUrlPreview ()
        }
        super.flagsChanged(with: event)
    }
    
    public override func mouseExited(with event: NSEvent) {
        turnOffUrlPreview()
        super.mouseExited(with: event)
    }
    
    /// If set to true, the terminal treats the "Option" key as the Meta key in old terminals,
    /// which has the effect of sending the ESC character before the character that was
    /// entered.  Applications use this to provide bindings for Alt-keys, or in Emacs terms
    /// the Meta key (M-x stands for Meta-x, or pressing the option key and x).
    ///
    /// If this is set to `false`, then the key is passed to the OS, which produces the
    /// OS specific feature.
    public var optionAsMetaKey: Bool = true
    
    //
    // We capture a handful of keydown events and pre-process those, and then let
    // interpretKeyEvents do the rest of the work, that includes text-insertion, and
    // keybinding mapping.
    //
    // That is why we do not handle things like the return key here, instead those are
    // handled by doCommand below.
    //
    // This currently handles the function keys here, but probably should be done in
    // doCommand/noop: - but more research needs to take place to figure out the priority
    // of those keys.
    //
    public override func keyDown(with event: NSEvent) {
        selection.active = false
        let eventFlags = event.modifierFlags
        
        // Handle Option-letter to send the ESC sequence plus the letter as expected by terminals
        if optionAsMetaKey && eventFlags.contains (.option) {
            if let rawCharacter = event.charactersIgnoringModifiers {
                send (EscapeSequences.CmdEsc)
                send (txt: rawCharacter)
            }
            return
        } else if eventFlags.contains (.control) {
            // Sends the control sequence
            if let ch = event.charactersIgnoringModifiers {
                send (applyControlToEventCharacters (ch))
                return
            }
        } else if eventFlags.contains (.function) {
            if let str = event.charactersIgnoringModifiers {
                if let fs = str.unicodeScalars.first {
                    let c = Int (fs.value)
                    switch c {
                    case NSF1FunctionKey:
                        send (EscapeSequences.CmdF [0])
                    case NSF2FunctionKey:
                        send (EscapeSequences.CmdF [1])
                    case NSF3FunctionKey:
                        send (EscapeSequences.CmdF [2])
                    case NSF4FunctionKey:
                        send (EscapeSequences.CmdF [3])
                    case NSF5FunctionKey:
                        send (EscapeSequences.CmdF [4])
                    case NSF6FunctionKey:
                        send (EscapeSequences.CmdF [5])
                    case NSF7FunctionKey:
                        send (EscapeSequences.CmdF [6])
                    case NSF8FunctionKey:
                        send (EscapeSequences.CmdF [7])
                    case NSF9FunctionKey:
                        send (EscapeSequences.CmdF [8])
                    case NSF10FunctionKey:
                        send (EscapeSequences.CmdF [9])
                    case NSF11FunctionKey:
                        send (EscapeSequences.CmdF [10])
                    case NSF12FunctionKey:
                        send (EscapeSequences.CmdF [11])
                    case NSDeleteFunctionKey:
                        send (EscapeSequences.CmdDelKey)
                        //                    case NSUpArrowFunctionKey:
                        //                        send (EscapeSequences.MoveUpNormal)
                        //                    case NSDownArrowFunctionKey:
                        //                        send (EscapeSequences.MoveDownNormal)
                        //                    case NSLeftArrowFunctionKey:
                        //                        send (EscapeSequences.MoveLeftNormal)
                        //                    case NSRightArrowFunctionKey:
                    //                        send (EscapeSequences.MoveRightNormal)
                    case NSPageUpFunctionKey:
                        pageUp ()
                    case NSPageDownFunctionKey:
                        pageDown()
                    default:
                        interpretKeyEvents([event])
                    }
                }
            }
            return
        }
        
        interpretKeyEvents([event])
    }
    
    public override func doCommand(by selector: Selector) {
        switch selector {
        case #selector(insertNewline(_:)):
            send (EscapeSequences.CmdRet)
        case #selector(cancelOperation(_:)):
            send (EscapeSequences.CmdEsc)
        case #selector(deleteBackward(_:)):
            send ([backspaceSendsControlH ? 8 : 0x7f])
        case #selector(moveUp(_:)):
            sendKeyUp()
        case #selector(moveDown(_:)):
            sendKeyDown()
        case #selector(moveLeft(_:)):
            sendKeyLeft()
        case #selector(moveRight(_:)):
            sendKeyRight()
        case #selector(insertTab(_:)):
            send (EscapeSequences.CmdTab)
        case #selector(insertBacktab(_:)):
            send (EscapeSequences.CmdBackTab)
        case #selector(moveToBeginningOfLine(_:)):
            send (terminal.applicationCursor ? EscapeSequences.MoveHomeApp : EscapeSequences.MoveHomeNormal)
        case #selector(moveToEndOfLine(_:)):
            send (terminal.applicationCursor ? EscapeSequences.MoveEndApp : EscapeSequences.MoveEndNormal)
        case #selector(scrollPageUp(_:)):
            fallthrough
        case #selector(pageUp(_:)):
            if terminal.applicationCursor {
                send (EscapeSequences.CmdPageUp)
            } else {
                pageUp()
            }
        case #selector(scrollPageDown(_:)):
            fallthrough
        case #selector(pageDown(_:)):
            if terminal.applicationCursor {
                send (EscapeSequences.CmdPageDown)
            } else {
                pageDown()
            }
        case #selector(pageDownAndModifySelection(_:)):
            if terminal.applicationCursor {
                // TODO: view should scroll one page up.
            } else {
                send (EscapeSequences.CmdPageDown)
            }
            break;
        default:
            print ("Unhandle selector \(selector)")
        }
    }
    
    // NSTextInputClient protocol implementation
    open func insertText(_ string: Any, replacementRange: NSRange) {
        if let str = string as? NSString {
            send (txt: str as String)
        }
        // TODO: I do not think we actually need this needsDisplay, the data fed should bubble this up
        // needsDisplay = true
    }
    
    // NSTextInputClient protocol implementation
    open func setMarkedText(_ string: Any, selectedRange: NSRange, replacementRange: NSRange) {
        // nothing
    }
    
    // NSTextInputClient protocol implementation
    open func unmarkText() {
        // nothing
    }
    
    // NSTextInputClient protocol implementation
    open func selectedRange() -> NSRange {
        guard let selection = self.selection, selection.active else {
            // This means "no selection":
            return NSRange.empty
        }
        
        var startLocation = (selection.start.row * terminal.buffer.rows) + selection.start.col
        var endLocation = (selection.end.row * terminal.buffer.rows) + selection.end.col
        if startLocation > endLocation {
            swap(&startLocation, &endLocation)
        }
        let length = endLocation - startLocation
        if length == 0 {
            return NSRange.empty
        }
        return NSRange(location: startLocation, length: endLocation - startLocation)
    }
    
    // NSTextInputClient protocol implementation
    open func markedRange() -> NSRange {
        print ("markedRange: This should return the actual range from the selection")
        
        // This means "no marked" - when we fix, we should address
        return NSRange.empty
    }
    
    // NSTextInputClient protocol implementation
    open func hasMarkedText() -> Bool {
        // print ("hasMarkedText: This should return the actual range from the selection")
        // TODO
        return false
    }
    
    // NSTextInputClient protocol implementation
    open func attributedSubstring(forProposedRange range: NSRange, actualRange: NSRangePointer?) -> NSAttributedString? {
        print ("Attribuetd string")
        return nil
    }
    
    // NSTextInputClient Protocol implementation
    open func validAttributesForMarkedText() -> [NSAttributedString.Key] {
        // TODO print ("validAttributesForMarkedText: This should return the actual range from the selection")
        return []
    }
    
    // NSTextInputClient protocol implementation
    open func firstRect(forCharacterRange range: NSRange, actualRange: NSRangePointer?) -> NSRect {
        actualRange?.pointee = range
        
        if let r = window?.convertToScreen(convert(caretView!.frame, to: nil)) {
            return r
        }
        
        return .zero
    }
    
    // NSTextInputClient protocol implementation
    open func characterIndex(for point: NSPoint) -> Int {
        print ("characterIndex:for point: This should return the actual range from the selection")
        return NSNotFound
    }
    
    open func validateUserInterfaceItem(_ item: NSValidatedUserInterfaceItem) -> Bool {
        //print ("Validating selector: \(item.action)")
        switch item.action {
        case #selector(performTextFinderAction(_:)):
            if let fa = NSTextFinder.Action (rawValue: item.tag) {
                switch fa {
                case .showFindInterface:
                    return true
                case .showReplaceInterface:
                    return true
                case .hideReplaceInterface:
                    return true
                default:
                    return false
                }
            }
            return false
        case #selector(paste(_:)):
            return true
        case #selector(selectAll(_:)):
            return true
        case #selector(copy(_:)):
            return selection.active
        default:
            print ("Validating User Interface Item: \(item)")
            return false
        }
    }
    
    open func selectionChanged(source: Terminal) {
        updateSelectionInBuffer(terminal: source)
        needsDisplay = true
    }
    
    func cut (sender: Any?) {}
    
    @objc
    open func paste(_ sender: Any)
    {
        let clipboard = NSPasteboard.general
        let text = clipboard.string(forType: .string)
        insertText(text ?? "", replacementRange: NSRange(location: 0, length: 0))
    }
    
    @objc
    open func copy(_ sender: Any)
    {
        // find the selected range of text in the buffer and put in the clipboard
        let str = selection.getSelectedText()
        
        let clipboard = NSPasteboard.general
        clipboard.clearContents()
        clipboard.setString(str, forType: .string)
    }
    
    public override func selectAll(_ sender: Any?)
    {
        selection.selectAll()
    }
    
    //func undo (sender: Any) {}
    //func redo (sender: Any) {}
    func zoomIn (sender: Any) {}
    func zoomOut (sender: Any) {}
    func zoomReset (sender: Any) {}
    
    // Returns the vt100 mouseflags
    func encodeMouseEvent (with event: NSEvent) -> Int
    {
        let flags = event.modifierFlags
        let isReleaseEvent = [NSEvent.EventType.leftMouseUp, .otherMouseUp, .rightMouseUp].contains(event.type)
        
        return terminal.encodeButton(button: event.buttonNumber, release: isReleaseEvent, shift: flags.contains(.shift), meta: flags.contains(.option), control: flags.contains(.control))
    }
    
    func calculateMouseHit (with event: NSEvent) -> Position
    {
        let point = convert(event.locationInWindow, from: nil)
        let col = Int (point.x / cellDimension.width)
        let row = Int ((frame.height-point.y) / cellDimension.height)
        if row < 0 {
            return Position(col: 0, row: 0)
        }
        return Position(col: min (max (0, col), terminal.cols-1), row: min (row, terminal.rows-1))
    }
    
    private func sharedMouseEvent (with event: NSEvent)
    {
        let hit = calculateMouseHit(with: event)
        let buttonFlags = encodeMouseEvent(with: event)
        terminal.sendEvent(buttonFlags: buttonFlags, x: hit.col, y: hit.row)
    }
    
    private var autoScrollDelta = 0
    // Callback from when the mouseDown autoscrolling timer goes off
    private func scrollingTimerElapsed (source: Timer)
    {
        if autoScrollDelta == 0 {
            return
        }
        if autoScrollDelta < 0 {
            scrollUp(lines: autoScrollDelta * -1)
        } else {
            scrollUp(lines: autoScrollDelta)
        }
    }
    
    public override func mouseDown(with event: NSEvent) {
        if allowMouseReporting && terminal.mouseMode.sendButtonPress() {
            sharedMouseEvent(with: event)
            return
        }
        
        let hit = calculateMouseHit(with: event)
        
        switch event.clickCount {
        case 1:
            if selection.active == true {
                if event.modifierFlags.contains(.shift) {
                    selection.shiftExtend(row: hit.row, col: hit.col)
                } else {
                    selection.active = false
                }
            }
        case 2:
            selection.selectWordOrExpression(at: Position(col: hit.col, row: hit.row + terminal.buffer.yDisp), in: terminal.buffer)
        default:
            // 3 and higher
            selection.select(row: hit.row + terminal.buffer.yDisp)
        }
    }
    
    func getPayload (for event: NSEvent) -> String?
    {
        let hit = calculateMouseHit(with: event)
        let cd = terminal.buffer.lines [terminal.buffer.yDisp+hit.row][hit.col]
        return cd.getPayload()
    }
    
    var didSelectionDrag: Bool = false
    
    public override func mouseUp(with event: NSEvent) {
        if event.modifierFlags.contains(.command){
            if let payload = getPayload(for: event) {
                if let (url, params) = urlAndParamsFrom(payload: payload) {
                    terminalDelegate?.requestOpenLink(source: self, link: url, params: params)
                }
            }
        }
        if allowMouseReporting && terminal.mouseMode.sendButtonRelease() {
            sharedMouseEvent(with: event)
            return
        }
        
        #if DEBUG
        // let hit = calculateMouseHit(with: event)
        //print ("Up at col=\(hit.col) row=\(hit.row) count=\(event.clickCount) selection.active=\(selection.active) didSelectionDrag=\(didSelectionDrag) ")
        #endif
        
        didSelectionDrag = false
    }
    
    public override func mouseDragged(with event: NSEvent) {
        let hit = calculateMouseHit(with: event)
        if allowMouseReporting {
            if terminal.mouseMode.sendMotionEvent() {
                let flags = encodeMouseEvent(with: event)
            
                terminal.sendMotion(buttonFlags: flags, x: hit.col, y: hit.row)
            
                return
            }
            if terminal.mouseMode != .off {
                return
            }
        }
                
        if selection.active {
            selection.dragExtend(row: hit.row, col: hit.col)
        } else {
            selection.startSelection(row: hit.row, col: hit.col)
        }
        didSelectionDrag = true
        autoScrollDelta = 0
        if selection.active {
            if hit.row <= 0 {
                autoScrollDelta = calcScrollingVelocity(delta: hit.row * -1) * -1
            } else if hit.row >= terminal.rows {
                autoScrollDelta = calcScrollingVelocity(delta: hit.row - terminal.rows)
            }
        }
    }
    
    func tryUrlFont () -> NSFont
    {
        for x in ["Optima", "Helvetica", "Helvetica Neue"] {
            if let font = NSFont (name: x, size: 12) {
                return font
            }
        }
        return NSFont.systemFont(ofSize: 12)
    }
    
    // The payload contains the terminal data which is expected to be of the form
    // params;URL, so we need to extract the second component, but we also assume that
    // the input might be ill-formed, so we might return nil in that case
    func urlAndParamsFrom (payload: String) -> (String, [String:String])?
    {
        let split = payload.split(separator: ";", maxSplits: Int.max, omittingEmptySubsequences: false)
        if split.count > 1 {
            let pairs = split [0].split (separator: ":")
            var params: [String:String] = [:]
            for p in pairs {
                let kv = p.split (separator: "=")
                if kv.count == 2 {
                    params [String (kv [0])] = String (kv[1])
                }
            }
            return (String (split [1]), params)
        }
        return nil
    }
    
    var urlPreview: NSTextField?
    func previewUrl (payload: String)
    {
        if let (url, _) = urlAndParamsFrom(payload: payload) {
            if let up = urlPreview {
                up.stringValue = url
                up.sizeToFit()
            } else {
                let nup: NSTextField
                if #available(OSX 10.12, *) {
                    nup = NSTextField (string: url)
                } else {
                    nup = NSTextField ()
                }
                nup.isBezeled = false
                nup.font = tryUrlFont ()
                nup.backgroundColor = nativeForegroundColor
                nup.textColor = nativeBackgroundColor
                nup.sizeToFit()
                nup.frame = CGRect (x: 0, y: 0, width: nup.frame.width, height: nup.frame.height)
                addSubview(nup)
                urlPreview = nup
            }
        }
    }
    
    func removePreviewUrl ()
    {
        if let urlPreview = self.urlPreview {
            urlPreview.removeFromSuperview()
            self.urlPreview = nil
        }
    }
    
    public override func mouseMoved(with event: NSEvent) {
        let hit = calculateMouseHit(with: event)
        if commandActive {
            if let payload = getPayload(for: event) {
                previewUrl (payload: payload)
            }
        }
        
        if terminal.mouseMode.sendMotionEvent() {
            let flags = encodeMouseEvent(with: event)
            terminal.sendMotion(buttonFlags: flags, x: hit.col, y: hit.row)
        }
    }
    
    public override func scrollWheel(with event: NSEvent) {
        if event.deltaY == 0 {
            return
        }
        let velocity = calcScrollingVelocity(delta: Int (abs (event.deltaY)))
        if event.deltaY > 0 {
            scrollUp (lines: velocity)
        } else {
            scrollDown(lines: velocity)
        }
    }
    
    private func calcScrollingVelocity (delta: Int) -> Int
    {
        if delta > 9 {
            return max (terminal.rows, 20)
        }
        if delta > 5 {
            return 10
        }
        if delta > 1 {
            return 3
        }
        return 1
    }
    
    public override func resetCursorRects() {
        addCursorRect(bounds, cursor: .iBeam)
    }
    
    public func resetFontSize ()
    {
        fontSet = FontSet (font: FontSet.defaultFont)
    }    
}

extension TerminalView: TerminalDelegate {
    open func isProcessTrusted(source: Terminal) -> Bool {
        true
    }
    
    open func mouseModeChanged(source: Terminal) {
        if source.mouseMode == .anyEvent {
            startTracking()
        } else {
            if terminal != nil {
                deregisterTrackingInterest()
            }
        }
    }
    
    open func setTerminalTitle(source: Terminal, title: String) {
        terminalDelegate?.setTerminalTitle(source: self, title: title)
    }
    
    open func sizeChanged(source: Terminal) {
        terminalDelegate?.sizeChanged(source: self, newCols: source.cols, newRows: source.rows)
        updateScroller ()
    }
    
    open func setTerminalIconTitle(source: Terminal, title: String) {
        //
    }
    
    // Terminal.Delegate method implementation
    open func windowCommand(source: Terminal, command: Terminal.WindowManipulationCommand) -> [UInt8]? {
        return nil
    }
    
}

extension NSColor {
    func getTerminalColor () -> Color {
        guard let color = self.usingColorSpace(.deviceRGB) else {
            return Color.defaultForeground
        }
        
        var red: CGFloat = 0.0, green: CGFloat = 0.0, blue: CGFloat = 0.0, alpha: CGFloat = 1.0
        color.getRed(&red, green: &green, blue: &blue, alpha: &alpha)
        return Color(red: UInt16(red*65535), green: UInt16(green*65535), blue: UInt16(blue*65535))
    }
    func inverseColor() -> NSColor {
        guard let color = self.usingColorSpace(.deviceRGB) else {
            return self
        }
        
        var red: CGFloat = 0.0, green: CGFloat = 0.0, blue: CGFloat = 0.0, alpha: CGFloat = 1.0
        color.getRed(&red, green: &green, blue: &blue, alpha: &alpha)
        return NSColor(calibratedRed: 1.0 - red, green: 1.0 - green, blue: 1.0 - blue, alpha: alpha)
    }

    static func make (red: CGFloat, green: CGFloat, blue: CGFloat, alpha: CGFloat) -> NSColor
    {
        return NSColor (deviceRed: red, green: green, blue: blue, alpha: alpha)
    }
    
    static func make (hue: CGFloat, saturation: CGFloat, brightness: CGFloat, alpha: CGFloat) -> TTColor
    {
        return NSColor (
            calibratedHue: hue,
            saturation: saturation,
            brightness: brightness,
            alpha: alpha)
    }

    static func make (color: Color) -> NSColor
    {
        return NSColor (deviceRed: CGFloat (color.red) / 65535.0,
                        green: CGFloat (color.green) / 65535.0,
                        blue: CGFloat (color.blue) / 65535.0,
                        alpha: 1.0)
    }
    
    static func transparent () -> NSColor {
        return NSColor (calibratedWhite: 0, alpha: 0)
    }
}

// Default implementations for TerminalViewDelegate

extension TerminalViewDelegate {
    public func requestOpenLink (source: TerminalView, link: String, params: [String:String])
    {
        if let fixedup = link.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) {
            if let url = NSURLComponents(string: fixedup) {
                if let nested = url.url {
                    NSWorkspace.shared.open(nested)
                }
            }
        }
    }
    
    public func bell (source: TerminalView)
    {
        NSSound.beep()
    }
}

extension NSBezierPath {
    func addLine(to: CGPoint)
    {
        self.line (to: to)
    }
}

#endif
