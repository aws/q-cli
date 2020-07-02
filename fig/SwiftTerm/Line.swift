//
//  Line.swift
//  
//
//  Created by Miguel de Icaza on 3/13/20.
//

import Foundation

struct LineFragment {
    var text: String?
    var line: Int
    var location: Int
    var length: Int
    
    static func newLine (line: Int) -> LineFragment
    {
        LineFragment(text: "\n", line: line, location: -1, length: 1)
    }
}

class Line : CustomDebugStringConvertible {
    var fragments: [LineFragment] = []
    
    public init ()
    {
    }
    
    // gets the line number of the first fragment
    var startLine: Int? {
        get {
            return fragments.first?.line ?? nil
        }
    }
    
    var startLocation: Int? {
        get {
            fragments.first?.location ?? nil
        }
    }
    
    private(set) var length: Int = 0
    
    public var debugDescription: String {
        get {
            if fragments.count == 0 {
                return "[]"
            }
            var result = "\(fragments.count)/\(length): ["
            for fragment in fragments {
                if fragment.text == "\n" {
                    result += "\\n"
                } else {
                    result += fragment.text ?? ""
                }
                result += "]["
            }
            return result
        }
    }
    
    func add (fragment: LineFragment)
    {
        fragments.append(fragment)
        length += fragment.length
    }
    
    func toString () -> String
    {
        var result = ""
        for x in fragments {
            result += x.text ?? ""
        }
        return result
    }
    
    func getFragmentIndex (forPosition: Int) -> Int
    {
        var count = 0
        for i in 0..<fragments.count {
            count += fragments[i].length
            if count > forPosition {
                return i
            }
        }
        return fragments.count - 1
    }
    
    subscript (idx: Int) -> LineFragment {
        return fragments [idx]
    }
    
}
