//
//  File.swift
//  
//
//  Created by Miguel de Icaza on 3/22/20.
//
#if os(OSX)
import Foundation
import AppKit

public class TerminalDebugView: NSView {
    var terminalView: TerminalView
    var terminal: Terminal
    var font = NSFont(name: "Lucida Sans Typewriter", size: 8) ?? NSFont(name: "Courier", size: 8)!
    var height: CGFloat
    var dbg: NSTextField
    
    func computeCellDimensions () -> CGRect
    {
        let line = CTLineCreateWithAttributedString (NSAttributedString (string: "W", attributes: [NSAttributedString.Key.font: font]))
        
        return CTLineGetBoundsWithOptions(line, .useOpticalBounds)
    }

    public func update ()
    {
        setNeedsDisplay(frame)
        dbg.stringValue = "x: \(terminal.buffer.x) y: \(terminal.buffer.y) yDisp: \(terminal.buffer.yDisp) yBase: \(terminal.buffer.yBase) clc: \(terminal.buffer._lines.array.count) startIndex: \(terminal.buffer._lines.startIndex)"
    }
    
    public init (frame: CGRect, terminal: TerminalView)
    {
        self.terminalView = terminal
        dbg = NSTextField(frame: NSRect (x: 0, y: 8, width: frame.width, height: 14))
        
        dbg.font = font
        dbg.stringValue = "WAITING"
        self.terminal = terminal.getTerminal()
        height = 0
        super.init (frame: frame)
        terminalView.debug = self
        height = computeCellDimensions ().height
        addSubview(dbg)
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    
    func getDebugString (line _line: BufferLine?, cols: Int, prefix: String = "", hilight: Bool, col: Int) -> NSAttributedString
    {
        let res = NSMutableAttributedString ()
        var attr = Attribute.empty
        
        let nsattr: [NSAttributedString.Key:Any] = [
            .font: font,
            .foregroundColor: NSColor.black,
            .backgroundColor: NSColor.white
        ]

        let selLineAttr: [NSAttributedString.Key:Any] = [
            .font: font,
            .foregroundColor: NSColor.red,
            .backgroundColor: NSColor.black
        ]
        
        if hilight {
            print ("here")
        }
        res.append (NSAttributedString (string: prefix, attributes: (hilight ? selLineAttr : nsattr)))
        var str = ""
        if let line = _line {
            for col in 0..<cols {
                let ch: CharData = line[col]
                if col == 0 {
                    attr = ch.attribute
                } else {
                    if attr != ch.attribute {
                        res.append(NSAttributedString (string: str, attributes: nsattr))
                        str = ""
                        attr = ch.attribute
                    }
                }
                str.append(ch.code == 0 ? " " : ch.getCharacter())
            }
        } else {
            str = "<empty>"
        }
        res.append (NSAttributedString(string: str, attributes: nsattr))
        return res
    }
    
    public override func draw(_ dirtyRect: NSRect) {
        NSColor.white.set ()
        bounds.fill()
        
        //print ("Dirty rect is: \(dirtyRect)")
        NSColor.black.set ()
        guard let context = NSGraphicsContext.current?.cgContext else {
            return
        }
        context.saveGState()
        
        let baseLine = frame.height - height
        let debugBuffer = terminal.buffer
        for y in 0..<debugBuffer._lines.maxLength {
            context.textPosition = CGPoint (x: 0, y: baseLine - (height + CGFloat (y) * height))
            let flag = y == debugBuffer.yDisp ? "D" : " "
            let yb   = y == debugBuffer.yBase ? "B" : " "
            let istr = String (format: "%03d", y)
            let cstr = String (format: "%03d", debugBuffer._lines.getCyclicIndex(y))
            
            let attrLine = getDebugString(line: debugBuffer._lines.array [y], cols: terminal.cols, prefix: "[\(istr):\(cstr)]\(flag)\(yb)", hilight: false, col: debugBuffer.x)
            let ctline = CTLineCreateWithAttributedString(attrLine)
            CTLineDraw(ctline, context)
            context.drawPath(using: .fillStroke)

            let attrLine2 = getDebugString(line: debugBuffer._lines [y], cols: terminal.cols, prefix: "[\(istr)]\(flag)\(yb)", hilight: false, col: debugBuffer.x)
            let ctline2 = CTLineCreateWithAttributedString(attrLine2)
            CTLineDraw(ctline2, context)
            context.drawPath(using: .fillStroke)

        }
        context.restoreGState()
    }
}
#endif
