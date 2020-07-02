//
//  BufferLine.swift
//  SwiftTerm
//
//  Created by Miguel de Icaza on 3/26/19.
//  Copyright © 2019 Miguel de Icaza. All rights reserved.
//

import Foundation

class BufferLine: CustomDebugStringConvertible{
    var isWrapped: Bool
    var data: [CharData]
    
    public init (cols: Int, fillData: CharData? = nil, isWrapped: Bool = false)
    {
        let fill = (fillData == nil) ? CharData.Null : fillData!
        data = Array.init(repeating: fill, count: cols)
        self.isWrapped = isWrapped
    }
    
    public init (from other: BufferLine)
    {
        data = other.data
        isWrapped = other.isWrapped
    }
    
    public var count: Int {
        get {
            return data.count
        }
    }
    
    public subscript (index : Int /*, callingMethod: String = #function */) -> CharData {
        get {
            // The x value in a buffer can point beyond the column, due to the way that we allow
            // buffer.x to grow (this is to support some wrapmodes and write on the edge)
            if index >= data.count {
                /* print ("Warning: the method \(callingMethod) has not been audited to clamp buffer.x to cols-1; fixing") */
                return data [data.count-1]
            }
            return data [index]
        }
        set(value) {
            if index >= data.count {
                /* print ("Warning: the method \(callingMethod) has not been audited to clamp buffer.x to cols-1; fixing") */
                data [data.count-1] = value
            }
            data [index] = value
        }
    }
    
    public func getWidth (index: Int) -> Int {
        return Int (data [index].width)
    }
    
    /// Test whether contains any chars.
    public func hasContent (index: Int) -> Bool {
        data [index].code != 0 || data [index].attribute != CharData.defaultAttr;
    }
    
    public func hasAnyContent () -> Bool {
        for i in 0..<data.count {
            if hasContent(index: i) {
                return true
            }
        }
        return false
    }
    
    public func insertCells (pos: Int, n: Int, rightMargin: Int, fillData: CharData)
    {
        let len = rightMargin + 1
        //let len = data.count
        let pos = pos % len
        if n < len - pos {
            for i in (0..<len-pos-n).reversed() {
                data [pos+n+i] = data [pos+i]
            }
            for i in 0..<n {
                data [pos+i] = fillData
            }
        } else {
            for i in pos..<len {
                data [i] = fillData
            }
        }
    }
    
    public func deleteCells (pos: Int, n: Int, rightMargin: Int, fillData: CharData)
    {
        // let len = data.count
        let len = rightMargin + 1 
        let p = pos % len
        if n < len - p {
            for i in 0..<len-pos-n {
                data [pos+i] = self [pos+n+i]
            }
            for i in len-n..<len {
                data [i] = fillData
            }
        } else {
            for i in pos..<len {
                data [i] = fillData
            }
        }
    }
    
    public func replaceCells (start : Int, end : Int, fillData : CharData)
    {
        let length = data.count
        var idx = start
        while idx < end && idx < length {
            data [idx] = fillData
            idx += 1
        }
    }
    
    public func resize (cols : Int, fillData : CharData)
    {
        let len = data.count
        if len == cols {
            return
        }
        
        if cols > len {
            var newData = Array.init(repeating: fillData, count: cols)
            if len > 0 {
                for i in 0..<len {
                    newData [i] = data [i]
                }
            }
            data = newData
        } else {
            if cols > 0 {
                data = Array.init (data [0..<cols])
            } else {
                data = [CharData]()
            }
        }
    }
    
    public func fill (with: CharData)
    {
        for i in 0..<data.count {
            data [i] = with
        }
    }
    
    public func fill (with: CharData, atCol: Int, len: Int)
    {
        for i in 0..<len {
            data [i+atCol] = with
        }
    }
    
    public func copyFrom (line: BufferLine)
    {
        if data.count != line.count {
            data = Array.init (repeating: CharData.Null, count: line.count)
        }
        for i in 0..<line.count {
            data [i] = line [i]
        }
        isWrapped = line.isWrapped
    }
    
    public func getTrimmedLength () -> Int
    {
        for i in (0..<data.count).reversed() {
            if data [i].code != 0 {
                return i + Int (data [i].width - 1)
//                for _ in 0...i {
//                    width += Int (data [i].width)
//                }
//                return width
            }
        }
        return 0
    }
    
    public func copyFrom (_ src: BufferLine, srcCol: Int, dstCol: Int, len: Int)
    {
        data.replaceSubrange(dstCol..<(dstCol+len), with: src.data [srcCol..<(srcCol+len)])
    }
    
    public func translateToString (trimRight: Bool = false, startCol: Int = 0, endCol: Int = -1) -> String
    {
        var ec = endCol == -1 ? data.count : endCol
        if trimRight {
            ec = min (ec, getTrimmedLength())
        }
        var result = ""
        for i in startCol..<ec {
            result.append (data [i].getCharacter ())
        }
        return result
    }
    
    public var debugDescription: String {
        get {
            translateToString()
        }
    }

}

