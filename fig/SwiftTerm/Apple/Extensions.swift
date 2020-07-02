//
//  File.swift
//  
//
//  Created by Miguel de Icaza on 4/15/20.
//
#if os(macOS) || os(iOS)
import Foundation

extension NSAttributedString.Key {
    static let fullBackgroundColor: NSAttributedString.Key = .init("SwiftTerm_fullBackgroundColor") // NSColor, default nil: no background
    static let selectionBackgroundColor: NSAttributedString.Key = .init("SwiftTerm_selectionBackgroundColor") // NSColor, default nil: no background
}

extension NSMutableAttributedString {
    func removeAttribute(_ attributeKey: NSAttributedString.Key) {
        self.removeAttribute(attributeKey, range: NSRange(location: 0, length: length))
    }
}

extension NSRange {
    var isEmpty: Bool {
        location == NSNotFound && length == 0
    }

    static var empty: NSRange {
        NSRange(location: NSNotFound, length: 0)
    }
}

#endif
