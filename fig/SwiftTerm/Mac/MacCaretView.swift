//
//  MacCaretView.swift
//  
// Implements the caret in the Mac caret view
//
//  Created by Miguel de Icaza on 3/20/20.
//

#if os(OSX)
import Foundation
import AppKit
import CoreText
import CoreGraphics

// The CaretView is used to show the cursor
class CaretView: NSView {
    public override init (frame: CGRect)
    {
        super.init(frame: frame)
        wantsLayer = true
        setupView()
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    
    public var caretColor: NSColor = NSColor.selectedControlColor {
        didSet {
            setupView()
        }
    }
    
    public var focused: Bool = false {
        didSet {
            setupView()
        }
    }

    func setupView() {
        guard let layer = layer else { return }
        layer.borderWidth = focused ? 0 : 2
        layer.borderColor = caretColor.cgColor
        layer.backgroundColor = focused ? caretColor.cgColor : NSColor.clear.cgColor
    }
    
    override func hitTest(_ point: NSPoint) -> NSView? {
        // we do not want to steal hits, let the terminal view take them
        return nil
    }
}
#endif
