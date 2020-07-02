//
//  iOSTerminalView.swift
//
// This is the AppKit version of the TerminalView and holds the state
// variables in the `TerminalView` class, but as much of the terminal
// implementation details live in the Apple/AppleTerminalView which
// contains the shared AppKit/UIKit code
//
//  The indicator "//X" means that this code was commented out from the Mac version for the sake of
//  porting and need to be audited.
//  Created by Miguel de Icaza on 3/4/20.
//

#if os(iOS)
import Foundation
import UIKit
import CoreText
import CoreGraphics

/**
 * TerminalView provides an UIKit front-end to the `Terminal` termininal emulator.
 * It is up to a subclass to either wire the terminal emulator to a remote terminal
 * via some socket, to an application that wants to run with terminal emulation, or
 * wiring this up to a pseudo-terminal.
 *
 * Users are notified of interesting events in their implementation of the `TerminalViewDelegate`
 * methods - an instance must be provided to the constructor of `TerminalView`.
 *
 * Call the `getTerminal` method to get a reference to the underlying `Terminal` that backs this
 * view.
 *
 * Use the `configureNativeColors()` to set the defaults colors for the view to match the OS
 * defaults, otherwise, this uses its own set of defaults colors.
 */
open class TerminalView: UIScrollView, UITextInputTraits, UIKeyInput, UIScrollViewDelegate {
    struct FontSet {
        public let normal: UIFont
        let bold: UIFont
        let italic: UIFont
        let boldItalic: UIFont
        
        static var defaultFont: UIFont {
            UIFont.monospacedSystemFont (ofSize: 12, weight: .regular)
        }
        
        public init(font baseFont: UIFont) {
            self.normal = baseFont
            self.bold = UIFont (descriptor: baseFont.fontDescriptor.withSymbolicTraits ([.traitBold])!, size: 0)
            self.italic = UIFont (descriptor: baseFont.fontDescriptor.withSymbolicTraits ([.traitItalic])!, size: 0)
            self.boldItalic = UIFont (descriptor: baseFont.fontDescriptor.withSymbolicTraits ([.traitItalic, .traitBold])!, size: 0)
        }
        
        // Expected by the shared rendering code
        func underlinePosition () -> CGFloat
        {
            return -1.2
        }
        
        // Expected by the shared rendering code
        func underlineThickness () -> CGFloat
        {
            return 0.63
        }
    }
    
    /**
     * The delegate that the TerminalView uses to interact with its hosting
     */
    public weak var terminalDelegate: TerminalViewDelegate?
    
    var accessibility: AccessibilityService = AccessibilityService()
    var search: SearchService!
    var debug: UIView?
    var pendingDisplay: Bool = false
    var cellDimension: CellDimension!
    var caretView: CaretView!
    var terminal: Terminal!
    var allowMouseReporting = true
    var selection: SelectionService!
    var attrStrBuffer: CircularList<NSAttributedString>!
    
    // Attribute dictionary, maps a console attribute (color, flags) to the corresponding dictionary
    // of attributes for an NSAttributedString
    var attributes: [Attribute: [NSAttributedString.Key:Any]] = [:]
    var urlAttributes: [Attribute: [NSAttributedString.Key:Any]] = [:]
    
    // Cache for the colors in the 0..255 range
    var colors: [UIColor?] = Array(repeating: nil, count: 256)
    var trueColors: [Attribute.Color:UIColor] = [:]
    var transparent = TTColor.transparent ()

    var fontSet: FontSet
    /// The font to use to render the terminal
    public var font: UIFont {
        get {
            return fontSet.normal
        }
        set {
            fontSet = FontSet (font: newValue)
            resetFont();
        }
    }
    
    public init(frame: CGRect, font: UIFont?) {
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
          
    func setup()
    {
        setupOptions ()
        setupGestures ()
        setupAccessoryView ()
    }
    
    @objc func pasteCmd(_ sender: Any?) {
        if let s = UIPasteboard.general.string {
            send(txt: s)
            queuePendingDisplay()
        }
        
    }
    
    @objc func resetCmd(_ sender: Any?) {
        terminal.cmdReset()
        queuePendingDisplay()
    }
    
    @objc func longPress (_ gestureRecognizer: UILongPressGestureRecognizer)
    {
         if gestureRecognizer.state == .began {
            self.becomeFirstResponder()
            //self.viewForReset = gestureRecognizer.view

            var items: [UIMenuItem] = []
            
            if UIPasteboard.general.hasStrings {
                items.append(UIMenuItem(title: "Paste", action: #selector(pasteCmd)))
            }
            items.append (UIMenuItem(title: "Reset", action: #selector(resetCmd)))
            
            // Configure the shared menu controller
            let menuController = UIMenuController.shared
            menuController.menuItems = items
            
            // TODO:
            //  - If nothing is selected, offer Select, Select All
            //  - If something is selected, offer copy, look up, share, "Search on StackOverflow"

            // Set the location of the menu in the view.
            let location = gestureRecognizer.location(in: gestureRecognizer.view)
            let menuLocation = CGRect(x: location.x, y: location.y, width: 0, height: 0)
            //menuController.setTargetRect(menuLocation, in: gestureRecognizer.view!)
            menuController.showMenu(from: gestureRecognizer.view!, rect: menuLocation)
            
          }
    }
    
    /// This controls whether the backspace should send ^? or ^H, the default is ^?
    public var backspaceSendsControlH: Bool = false
    
    func calculateTapHit (gesture: UIGestureRecognizer) -> Position
    {
        let point = gesture.location(in: self)
        let col = Int (point.x / cellDimension.width)
        let row = Int (point.y / cellDimension.height)
        if row < 0 {
            return Position(col: 0, row: 0)
        }
        return Position(col: min (max (0, col), terminal.cols-1), row: min (row, terminal.rows-1))
    }

    func encodeFlags (release: Bool) -> Int
    {
        let encodedFlags = terminal.encodeButton(
            button: 1,
            release: release,
            shift: false,
            meta: false,
            control: terminalAccessory?.controlModifier ?? false)
        terminalAccessory?.controlModifier = false
        return encodedFlags
    }
    
    func sharedMouseEvent (gestureRecognizer: UIGestureRecognizer, release: Bool)
    {
        let hit = calculateTapHit(gesture: gestureRecognizer)
        terminal.sendEvent(buttonFlags: encodeFlags (release: release), x: hit.col, y: hit.row)
    }
    
    @objc func singleTap (_ gestureRecognizer: UITapGestureRecognizer)
    {
        guard gestureRecognizer.view != nil else { return }
             
        if gestureRecognizer.state != .ended {
            return
        }
     
        if allowMouseReporting && terminal.mouseMode.sendButtonPress() {
            sharedMouseEvent(gestureRecognizer: gestureRecognizer, release: false)

            if terminal.mouseMode.sendButtonRelease() {
                sharedMouseEvent(gestureRecognizer: gestureRecognizer, release: true)
            }
            return
        }
    }
    
    @objc func doubleTap (_ gestureRecognizer: UITapGestureRecognizer)
    {
        guard gestureRecognizer.view != nil else { return }
               
        if gestureRecognizer.state != .ended {
            return
        }
        
        if allowMouseReporting && terminal.mouseMode.sendButtonPress() {
            sharedMouseEvent(gestureRecognizer: gestureRecognizer, release: false)
            
            if terminal.mouseMode.sendButtonRelease() {
                sharedMouseEvent(gestureRecognizer: gestureRecognizer, release: true)
            }
            return
        } else {
            // endEditing(true)
        }
    }
    
    @objc func pan (_ gestureRecognizer: UIPanGestureRecognizer)
    {
        guard gestureRecognizer.view != nil else { return }
        if allowMouseReporting {
            switch gestureRecognizer.state {
            case .began:
                // send the initial tap
                if terminal.mouseMode.sendButtonPress() {
                    sharedMouseEvent(gestureRecognizer: gestureRecognizer, release: false)
                }
            case .ended, .cancelled:
                if terminal.mouseMode.sendButtonRelease() {
                    sharedMouseEvent(gestureRecognizer: gestureRecognizer, release: true)
                }
            case .changed:
                if terminal.mouseMode.sendButtonTracking() {
                    let hit = calculateTapHit(gesture: gestureRecognizer)
                    terminal.sendMotion(buttonFlags: encodeFlags(release: false), x: hit.col, y: hit.row)
                }
            default:
                break
            }
        }
    }
    
    func setupGestures ()
    {
        let longPress = UILongPressGestureRecognizer (target: self, action: #selector(longPress(_:)))
        longPress.minimumPressDuration = 0.7
        addGestureRecognizer(longPress)
        
        let singleTap = UITapGestureRecognizer (target: self, action: #selector(singleTap(_:)))
        addGestureRecognizer(singleTap)
        
        let doubleTap = UITapGestureRecognizer (target: self, action: #selector(doubleTap(_:)))
        doubleTap.numberOfTapsRequired = 2
        addGestureRecognizer(doubleTap)
        
        let pan = UIPanGestureRecognizer (target: self, action: #selector(pan(_:)))
        addGestureRecognizer(pan)
    }

    var _inputAccessory: UIView?
    
    ///
    /// You can set this property to a UIView to be your input accessory, by default
    /// this is an instance of `TerminalAccessory`
    ///
    public override var inputAccessoryView: UIView? {
        get { _inputAccessory }
        set {
            _inputAccessory = newValue
        }
    }
    
    /// Returns the inputaccessory in case it is a TerminalAccessory and we can use it
    var terminalAccessory: TerminalAccessory? {
        get {
            _inputAccessory as? TerminalAccessory
        }
    }

    func setupAccessoryView ()
    {
        let ta = TerminalAccessory(frame: CGRect(x: 0, y: 0, width: frame.width, height: 36),
                                              inputViewStyle: .keyboard)
        ta.terminalView = self
        inputAccessoryView = ta
    }
    
    func setupOptions ()
    {
        setupOptions(width: bounds.width, height: bounds.height)
        layer.backgroundColor = nativeBackgroundColor.cgColor
        nativeBackgroundColor = UIColor.clear
    }
    
    var _nativeFg, _nativeBg: TTColor!
    var settingFg = false, settingBg = false
    /**
     * This will set the native foreground color to the specified native color (UIColor or NSColor)
     * and will have this reflected into the underlying's terminal `foregroundColor` and
     * `backgroundColor`
     */
    public var nativeForegroundColor: UIColor {
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
    public var nativeBackgroundColor: UIColor {
        get { _nativeBg }
        set {
            if settingBg { return }
            settingBg = true
            _nativeBg = newValue
            terminal.backgroundColor = nativeBackgroundColor.getTerminalColor ()
            colorsChanged()
            settingBg = false
        }
    }

    /// Controls the color for the caret
    public var caretColor: UIColor {
        get { caretView.caretColor }
        set { caretView.caretColor = newValue }
    }
    
    var _selectedTextBackgroundColor = UIColor.green
    /// The color used to render the selection
    public var selectedTextBackgroundColor: UIColor {
        get {
            return _selectedTextBackgroundColor
        }
        set {
            _selectedTextBackgroundColor = newValue
        }
    }
    


    var lineAscent: CGFloat = 0
    var lineDescent: CGFloat = 0
    var lineLeading: CGFloat = 0
    
    open func bufferActivated(source: Terminal) {
        updateScroller ()
    }
    
    open func send(source: Terminal, data: ArraySlice<UInt8>) {
        terminalDelegate?.send (source: self, data: data)
    }
    
    /**
     * Given the current set of columns and rows returns a frame that would host this control.
     */
    open func getOptimalFrameSize () -> CGRect
    {
        return CGRect (x: 0, y: 0, width: cellDimension.width * CGFloat(terminal.cols), height: cellDimension.height * CGFloat(terminal.rows))
    }
    
    func getEffectiveWidth (rect: CGRect) -> CGFloat
    {
        return rect.width
    }
    
    func updateDebugDisplay ()
    {
    }
    
    open func scrolled(source terminal: Terminal, yDisp: Int) {
        //XselectionView.notifyScrolled(source: terminal)
        updateScroller()
        terminalDelegate?.scrolled(source: self, position: scrollPosition)
    }
    
    open func linefeed(source: Terminal) {
        selection.selectNone()
    }
    
    func updateScroller ()
    {
        contentSize = CGSize (width: CGFloat (terminal.buffer.cols) * cellDimension.width,
                              height: CGFloat (terminal.buffer.lines.count) * cellDimension.height)
        // contentOffset = CGPoint (x: 0, y: CGFloat (terminal.buffer.lines.count-terminal.rows)*cellDimension.height)
        //Xscroller.doubleValue = scrollPosition
        //Xscroller.knobProportion = scrollThumbsize
    }
    
    var userScrolling = false

    func getCurrentGraphicsContext () -> CGContext?
    {
        UIGraphicsGetCurrentContext ()
    }

    func backingScaleFactor () -> CGFloat
    {
        UIScreen.main.scale
    }
    
    override public func draw (_ dirtyRect: CGRect) {
        guard let context = getCurrentGraphicsContext() else {
            return
        }

        // Without these two lines, on font changes, some junk is being displayed
        nativeBackgroundColor.set ()
        context.clear(dirtyRect)

        // drawTerminalContents and CoreText expect the AppKit coordinate system
        context.scaleBy (x: 1, y: -1)
        context.translateBy(x: 0, y: -frame.height)
        drawTerminalContents (dirtyRect: dirtyRect, context: context)
    }
    
    open override var frame: CGRect {
        get {
            return super.frame
        }
        set(newValue) {
            super.frame = newValue
            if cellDimension == nil {
                return
            }
            let newRows = Int (newValue.height / cellDimension.height)
            let newCols = Int (getEffectiveWidth (rect: newValue) / cellDimension.width)
            
            if newCols != terminal.cols || newRows != terminal.rows {
                terminal.resize (cols: newCols, rows: newRows)
                fullBufferUpdate (terminal: terminal)
            }
            
            accessibility.invalidate ()
            search.invalidate ()
            
            terminalDelegate?.sizeChanged (source: self, newCols: newCols, newRows: newRows)
            setNeedsDisplay (frame)
        }
    }
    
    // iOS Keyboard input
    
    // UITextInputTraits
    public var keyboardType: UIKeyboardType {
        get {
            .`default`
        }
    }
    
    public var keyboardAppearance: UIKeyboardAppearance = .`default`
    public var returnKeyType: UIReturnKeyType = .`default`
    
    // This is wrong, but I can not find another good one
    public var textContentType: UITextContentType! = .familyName
    
    public var isSecureTextEntry: Bool = false
    public var enablesReturnKeyAutomatically: Bool = false
    public var autocapitalizationType: UITextAutocapitalizationType  = .none
    public var autocorrectionType: UITextAutocorrectionType = .no
    public var spellCheckingType: UITextSpellCheckingType = .no
    public var smartQuotesType: UITextSmartQuotesType = .no
    public var smartDashesType: UITextSmartDashesType = .no
    public var smartInsertDeleteType: UITextSmartInsertDeleteType = .no
    
    public override var canBecomeFirstResponder: Bool {
        true
    }
    
    public var hasText: Bool {
        return true
    }

    open func insertText(_ text: String) {
        if terminalAccessory?.controlModifier ?? false {
            self.send (applyControlToEventCharacters (text))
            terminalAccessory?.controlModifier = false
        } else {
            self.send (txt: text)
        }

        queuePendingDisplay()
    }

    open func deleteBackward() {
        self.send ([0x7f])
    }

    enum SendData {
        case text(String)
        case bytes([UInt8])
    }
    
    var sentData: SendData?
    
    func sendData (data: SendData?)
    {
        switch sentData {
        case .bytes(let b):
            self.send (b)
        case .text(let txt):
            self.send (txt: txt)
        default:
            break
        }
    }
    
    public override func resignFirstResponder() -> Bool {
        let code = super.resignFirstResponder()
        
        if code {
            keyRepeat?.invalidate()
            keyRepeat = nil
            
            terminalAccessory?.cancelTimer()
        }
        return code
    }
    var keyRepeat: Timer?
    
    public override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        guard let key = presses.first?.key else { return }

        sentData = nil

        switch key.keyCode {
        case .keyboardCapsLock:
            break // ignored
        case .keyboardLeftAlt:
            break // ignored
        case .keyboardLeftControl:
            break // ignored
        case .keyboardLeftShift:
            break // ignored
        case .keyboardLockingCapsLock:
            break // ignored
        case .keyboardLockingNumLock:
            break // ignored
        case .keyboardLockingScrollLock:
            break // ignored
        case .keyboardRightAlt:
            break // ignored
        case .keyboardRightControl:
            break // ignored
        case .keyboardRightShift:
            break // ignored
        case .keyboardScrollLock:
            break // ignored
        case .keyboardUpArrow:
            sentData = .bytes (terminal.applicationCursor ? EscapeSequences.MoveUpApp : EscapeSequences.MoveUpNormal)
        case .keyboardDownArrow:
            sentData = .bytes (terminal.applicationCursor ? EscapeSequences.MoveDownApp : EscapeSequences.MoveDownNormal)
        case .keyboardLeftArrow:
            sentData = .bytes (terminal.applicationCursor ? EscapeSequences.MoveLeftApp : EscapeSequences.MoveLeftNormal)
        case .keyboardRightArrow:
            sentData = .bytes (terminal.applicationCursor ? EscapeSequences.MoveRightApp : EscapeSequences.MoveRightNormal)
        case .keyboardPageUp:
            if terminal.applicationCursor {
                sentData = .bytes (EscapeSequences.CmdPageUp)
            } else {
                pageUp()
            }

        case .keyboardPageDown:
            if terminal.applicationCursor {
                sentData = .bytes (EscapeSequences.CmdPageDown)
            } else {
                pageDown()
            }
        case .keyboardHome:
            sentData = .bytes (terminal.applicationCursor ? EscapeSequences.MoveHomeApp : EscapeSequences.MoveHomeNormal)
            
        case .keyboardEnd:
            sentData = .bytes (terminal.applicationCursor ? EscapeSequences.MoveEndApp : EscapeSequences.MoveEndNormal)
        case .keyboardDeleteForward:
            sentData = .bytes (EscapeSequences.CmdDelKey)
            
        case .keyboardDeleteOrBackspace:
            sentData = .bytes ([backspaceSendsControlH ? 8 : 0x7f])
            
        case .keyboardEscape:
            sentData = .bytes ([0x1b])
            
        case .keyboardInsert:
            print (".keyboardInsert ignored")
            break
            
        case .keyboardReturn:
            sentData = .bytes ([10])
            
        case .keyboardTab:
            sentData = .bytes ([9])

        case .keyboardF1:
            sentData = .bytes (EscapeSequences.CmdF [1])
        case .keyboardF2:
            sentData = .bytes (EscapeSequences.CmdF [2])
        case .keyboardF3:
            sentData = .bytes (EscapeSequences.CmdF [3])
        case .keyboardF4:
            sentData = .bytes (EscapeSequences.CmdF [4])
        case .keyboardF5:
            sentData = .bytes (EscapeSequences.CmdF [5])
        case .keyboardF6:
            sentData = .bytes (EscapeSequences.CmdF [6])
        case .keyboardF7:
            sentData = .bytes (EscapeSequences.CmdF [7])
        case .keyboardF8:
            sentData = .bytes (EscapeSequences.CmdF [8])
        case .keyboardF9:
            sentData = .bytes (EscapeSequences.CmdF [9])
        case .keyboardF10:
            sentData = .bytes (EscapeSequences.CmdF [10])
        case .keyboardF11:
            sentData = .bytes (EscapeSequences.CmdF [11])
        case .keyboardF12, .keyboardF13, .keyboardF14, .keyboardF15, .keyboardF16,
             .keyboardF17, .keyboardF18, .keyboardF19, .keyboardF20, .keyboardF21,
             .keyboardF22, .keyboardF23, .keyboardF24:
            break
        case .keyboardPause, .keyboardStop, .keyboardMute, .keyboardVolumeUp, .keyboardVolumeDown:
            break
            
        default:
            if key.modifierFlags.contains (.alternate) {
                sentData = .text("\u{1b}\(key.charactersIgnoringModifiers)")
            } else {
                sentData = .text (key.characters)
            }
        }
        
        //Timer.scheduledTimer(timeInterval: <#T##TimeInterval#>, invocation: <#T##NSInvocation#>, repeats: <#T##Bool#>)
        sendData (data: sentData)
    }
    
    public override func pressesChanged(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        print ("Here\n")
    }

    public override func pressesEnded(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        // guard let key = presses.first?.key else { return }
    }
    
    /// Confromance to UITextInput
//    func pabort (_ msg: String)
//    {
//        print (msg)
//        abort ()
//    }
//
//    public func text(in range: UITextRange) -> String? {
//        pabort ("PROTO: text(in)")
//        return "test"
//    }
//
//    public func replace(_ range: UITextRange, withText text: String) {
//        pabort ("PROTO: replace")
//    }
//
//    public var selectedTextRange: UITextRange? {
//        get {
//            print ("PROTO: TODO selectedTextRange")
//            return nil
//        }
//        set {
//            pabort ("PROTO: setting selectedtextrange")
//        }
//    }
//
//    public var markedTextRange: UITextRange? {
//        get {
//            print ("Request for marked-text-range")
//            return nil
//        }
//    }
//
//    public var markedTextStyle: [NSAttributedString.Key : Any]? {
//        get {
//            pabort ("PROTO: markedTextStyle")
//            return nil
//        }
//        set {
//            pabort ("PROTO: set markedTextStyle")
//        }
//    }
//
//    public func setMarkedText(_ markedText: String?, selectedRange: NSRange) {
//        pabort ("PROTO: etMarkedText")
//    }
//
//    public func unmarkText() {
//        pabort ("PROTO: unmarktext")
//    }
//
//    // The text position is relative to the start of the buffer (buffer.yBase)
//    class TerminalTextPosition: UITextPosition {
//        var pos: Position
//        init (_ pos: Position)
//        {
//            self.pos = pos
//        }
//    }
//    public var beginningOfDocument: UITextPosition {
//        get {
//            return TerminalTextPosition(Position (col: 0, row: 0))
//        }
//    }
//
//    public var endOfDocument: UITextPosition {
//        get {
//            return TerminalTextPosition(Position (col: terminal.buffer.cols, row: //terminal.buffer.lines.count))
//        }
//    }
//
//    public func textRange(from fromPosition: UITextPosition, to toPosition: //UITextPosition) -> UITextRange? {
//        pabort ("PROTO: textRange")
//        return nil
//    }
//
//    public func position(from position: UITextPosition, offset: Int) -> UITextPosition? {
//        pabort ("PROTO: position")
//        return nil
//    }
//
//    public func position(from position: UITextPosition, in direction: //UITextLayoutDirection, offset: Int) -> UITextPosition? {
//        pabort ("PROTO: position2")
//        return nil
//    }
//
//    public func compare(_ position: UITextPosition, to other: UITextPosition) -> //ComparisonResult {
//        if let a = position as? TerminalTextPosition {
//            if let b = other as? TerminalTextPosition {
//                switch Position.compare(a.pos, b.pos){
//                case .before:
//                    return .orderedAscending
//                case .after:
//                    return .orderedDescending
//                case .equal:
//                    return .orderedSame
//                }
//            }
//        }
//        return .orderedSame
//    }
//
//    public func offset(from: UITextPosition, to toPosition: UITextPosition) -> Int {
//        pabort ("PROTO: offset")
//        return 0
//    }
//
//    public weak var inputDelegate: UITextInputDelegate?
//
//    class MyInputTokenizer: NSObject, UITextInputTokenizer {
//        func pabort (_ msg: String)
//        {
//            print (msg)
//            abort()
//        }
//        func rangeEnclosingPosition(_ position: UITextPosition, with granularity: //UITextGranularity, inDirection direction: UITextDirection) -> UITextRange? {
//            pabort ("PROTO: MIT/Range")
//
//            return nil
//        }
//
//        func isPosition(_ position: UITextPosition, atBoundary granularity: //UITextGranularity, inDirection direction: UITextDirection) -> Bool {
//            pabort ("PROTO: MIT/offset")
//            return false
//        }
//
//        func position(from position: UITextPosition, toBoundary granularity: //UITextGranularity, inDirection direction: UITextDirection) -> UITextPosition? //{
//            pabort ("PROTO: MIT/position1")
//            return nil
//        }
//
//        func isPosition(_ position: UITextPosition, withinTextUnit granularity: //UITextGranularity, inDirection direction: UITextDirection) -> Bool {
//            pabort ("PROTO: MIT/position")
//            return false
//        }
//
//
//    }
//    public var tokenizer: UITextInputTokenizer = MyInputTokenizer()
//
//    public func position(within range: UITextRange, farthestIn direction: //UITextLayoutDirection) -> UITextPosition? {
//        pabort ("PROTO: position3")
//        return nil
//    }
//
//    public func characterRange(byExtending position: UITextPosition, in direction: //UITextLayoutDirection) -> UITextRange? {
//        pabort ("PROTO: characterRnage")
//        return nil
//    }
//
//    public func baseWritingDirection(for position: UITextPosition, in direction: //UITextStorageDirection) -> NSWritingDirection {
//        pabort ("PROTO: baseWritingDirection")
//        return .leftToRight
//    }
//
//    public func setBaseWritingDirection(_ writingDirection: NSWritingDirection, for //range: UITextRange) {
//        pabort ("PROTO: setBaseWritingDirection")
//
//    }
//
//    public func firstRect(for range: UITextRange) -> CGRect {
//        pabort ("PROTO: firstRect")
//        return CGRect.zero
//    }
//
//    public func caretRect(for position: UITextPosition) -> CGRect {
//        pabort ("PROTO: caretRect")
//        return CGRect.zero
//    }
//
//    public func selectionRects(for range: UITextRange) -> [UITextSelectionRect] {
//        pabort ("PROTO: selectionRects")
//        return []
//    }
//
//    public func closestPosition(to point: CGPoint) -> UITextPosition? {
//        pabort ("PROTO: closestPosition")
//        return nil
//    }
//
//    public func closestPosition(to point: CGPoint, within range: UITextRange) -> //UITextPosition? {
//        pabort ("PROTO: closestPosition")
//        return nil
//    }
//
//    public func characterRange(at point: CGPoint) -> UITextRange? {
//        pabort ("PROTO: characterRange")
//        return nil
//    }
//
}

extension TerminalView: TerminalDelegate {
    open func isProcessTrusted(source: Terminal) -> Bool {
        true
    }
    
    open func mouseModeChanged(source: Terminal) {
        // iOS TODO
        //X
    }
    
    open func setTerminalTitle(source: Terminal, title: String) {
        terminalDelegate?.setTerminalTitle(source: self, title: title)
    }
  
    open func sizeChanged(source: Terminal) {
        terminalDelegate?.sizeChanged(source: self, newCols: source.cols, newRows: source.rows)
        updateScroller()
    }
  
    open func setTerminalIconTitle(source: Terminal, title: String) {
        //
    }
  
    // Terminal.Delegate method implementation
    open func windowCommand(source: Terminal, command: Terminal.WindowManipulationCommand) -> [UInt8]? {
        return nil
    }
}

// Default implementations for TerminalViewDelegate

extension TerminalViewDelegate {
    public func requestOpenLink (source: TerminalView, link: String, params: [String:String])
    {
        if let fixedup = link.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) {
            if let url = NSURLComponents(string: fixedup) {
                if let nested = url.url {
                    UIApplication.shared.open (nested)
                }
            }
        }
    }
    
    public func bell (source: TerminalView)
    {
        let generator = UINotificationFeedbackGenerator()
        generator.notificationOccurred(.warning)
    }
}

extension UIColor {
    func getTerminalColor () -> Color {
        var red: CGFloat = 0.0, green: CGFloat = 0.0, blue: CGFloat = 0.0, alpha: CGFloat = 1.0
        self.getRed(&red, green: &green, blue: &blue, alpha: &alpha)
        return Color(red: UInt16 (red*65535), green: UInt16(green*65535), blue: UInt16(blue*65535))
    }

    func inverseColor() -> UIColor {
        var red: CGFloat = 0.0, green: CGFloat = 0.0, blue: CGFloat = 0.0, alpha: CGFloat = 1.0
        self.getRed(&red, green: &green, blue: &blue, alpha: &alpha)
        return UIColor (red: 1.0 - red, green: 1.0 - green, blue: 1.0 - blue, alpha: alpha)
    }

    static func make (red: CGFloat, green: CGFloat, blue: CGFloat, alpha: CGFloat) -> TTColor
    {
        
        return UIColor(red: red,
                       green: green,
                       blue: blue,
                       alpha: 1.0)
    }
  
    static func make (hue: CGFloat, saturation: CGFloat, brightness: CGFloat, alpha: CGFloat) -> TTColor
    {
        return UIColor(hue: hue,
                       saturation: saturation,
                       brightness: brightness,
                       alpha: alpha)
    }
    
    static func make (color: Color) -> UIColor
    {
        UIColor (red: CGFloat (color.red) / 65535.0,
                 green: CGFloat (color.green) / 65535.0,
                 blue: CGFloat (color.blue) / 65535.0,
                 alpha: 1.0)
    }
    
    static func transparent () -> UIColor {
        return UIColor.clear
    }

}
#endif
