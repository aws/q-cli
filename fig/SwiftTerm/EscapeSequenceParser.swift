//
//  EscapeSequenceParser.swift
//  SwiftTerm
//
//  Created by Miguel de Icaza on 3/28/19.
//  Copyright © 2019 Miguel de Icaza. All rights reserved.
//
// The state machien has been extended to allow ":" in the CSI Param state to initiate a new
// parameter value.   This is strictly not part of the spec, but necessary to parse the
// color scheme CSI [ 48:2:R:G:B m sequence which uses ":" instead of the more common ";"
//
// Alternative approaches:
//   * only allow ":" as a CsiParam if the first param is a 48/38.
//   * create an additiona "ignoredBuffer" that is passed to functions interested in those,
//     and this could be one of those.   Would be a little stricter, and probably better

import Foundation

enum ParserState : UInt8 {
    case ground = 0
    case escape
    case escapeIntermediate
    case csiEntry
    case csiParam
    case csiIntermediate
    case csiIgnore
    case sosPmApcString
    case oscString
    case dcsEntry
    case dcsParam
    case dcsIgnore
    case dcsIntermediate
    case dcsPassthrough
}

typealias cstring = [UInt8]

class ParsingState {
    var position: Int
    var code: UInt8
    var currentState: ParserState
    var print: Int
    var dcs: Int
    var osc: cstring
    var collect: cstring
    var parameters: [Int32]
    var abort: Bool
    
    init ()
    {
        position = 0
        code = 0
        currentState = .ground
        print = 0
        dcs = 0
        osc = []
        collect = []
        parameters = []
        abort = false
    }
}

enum ParserAction : UInt8 {
    case ignore = 0
    case error
    case print
    case execute
    case oscStart
    case oscPut
    case oscEnd
    case csiDispatch
    case param
    case collect
    case escDispatch
    case clear
    case dcsHook
    case dcsPut
    case dcsUnhook
}

class TransitionTable {
    // data is packed like this:
    // currentState << 8 | characterCode  -->  action << 4 | nextState
    var table: [UInt8]
    
    init (len: Int)
    {
        table = Array.init (repeating: 0, count: len)
    }
    
    func add (code: UInt8, state: ParserState, action: ParserAction, next: ParserState)
    {
        let v = (UInt8 (action.rawValue) << 4) | next.rawValue
        table [(Int (state.rawValue) << 8) | Int(code)] = v
    }
    
    func add (codes: [UInt8], state: ParserState, action: ParserAction, next: ParserState)
    {
        for c in codes {
            add (code: c, state: state, action: action, next: next)
        }
    }
    
    subscript (idx: Int) -> UInt8 {
        get {
            return table [idx]
        }
    }
}

protocol  DcsHandler {
    func hook (collect: cstring, parameters: [Int],  flag: UInt8)
    func put (data : ArraySlice<UInt8>)
    func unhook ()
}

class EscapeSequenceParser {
    
    static func r (low: UInt8, high: UInt8) -> [UInt8]
    {
        let c = high-low
        var ret = [UInt8]()
        for x in 0..<c {
            ret.append(low + x)
        }
        return ret;
    }
    
    static func rinclusive (low: ParserState, high: ParserState)-> [ParserState]
    {
        let c = high.rawValue-low.rawValue
        var ret = [ParserState]()
        for x in 0...c {
            ret.append(ParserState (rawValue: low.rawValue + x)!)
        }
        return ret;
    }
    
    static let NonAsciiPrintable : UInt8 = 0xa0
    
    static func buildVt500TransitionTable () -> TransitionTable
    {
        let table = TransitionTable(len: 4095)
        let states = rinclusive(low: .ground, high: .dcsPassthrough)
        
        // table with default transition
        for state in states {
            for code in 0...NonAsciiPrintable {
                table.add(code: code, state: state, action: .error, next: .ground)
            }
        }
        
        // printables
        let printables = r (low: 0x20, high: 0x7f)
        let executables = r (low: 0x00, high: 0x19) + r (low: 0x1c, high: 0x20)
        table.add (codes: printables, state: .ground, action: .print, next: .ground)
        
        // global anywhere rules
        for state in states {
            table.add (codes: [0x18, 0x1a, 0x99, 0x9a], state: state, action: .execute, next: .ground)
            table.add (codes: r (low: 0x80, high: 0x90), state: state, action: .execute, next: .ground)
            table.add (codes: r (low: 0x90, high: 0x98), state: state, action: .execute, next: .ground)
            table.add (code: 0x9c, state: state, action: .ignore, next: .ground) // ST as terminator
            table.add (code: 0x1b, state: state, action: .clear, next: .escape)  // ESC
            table.add (code: 0x9d, state: state, action: .oscStart, next: .oscString)  // OSC
            table.add (codes: [0x98, 0x9e, 0x9f], state: state, action: .ignore, next: .sosPmApcString)
            table.add (code: 0x9b, state: state, action: .clear, next: .csiEntry)  // CSI
            table.add (code: 0x90, state: state, action: .clear, next: .dcsEntry)  // DCS
        }
        // rules for executable and 0x7f
        table.add (codes: executables, state: .ground, action: .execute, next: .ground)
        table.add (codes: executables, state: .escape, action: .execute, next: .escape)
        table.add (code: 0x7f, state: .escape, action: .ignore, next: .escape)
        table.add (codes: executables, state: .oscString, action: .ignore, next: .oscString)
        table.add (codes: executables, state: .csiEntry, action: .execute, next: .csiEntry)
        table.add (code: 0x7f, state: .csiEntry, action: .ignore, next: .csiEntry)
        table.add (codes: executables, state: .csiParam, action: .execute, next: .csiParam)
        table.add (code: 0x7f, state: .csiParam, action: .ignore, next: .csiParam)
        table.add (codes: executables, state: .csiIgnore, action: .execute, next: .csiIgnore)
        table.add (codes: executables, state: .csiIntermediate, action: .execute, next: .csiIntermediate)
        table.add (code: 0x7f, state: .csiIntermediate, action: .ignore, next: .csiIntermediate)
        table.add (codes: executables, state: .escapeIntermediate, action: .execute, next: .escapeIntermediate)
        table.add (code: 0x7f, state: .escapeIntermediate, action: .ignore, next: .escapeIntermediate)
        // osc
        table.add (code: 0x5d, state: .escape, action: .oscStart, next: .oscString)
        table.add (codes: printables, state: .oscString, action: .oscPut, next: .oscString)
        table.add (code: 0x7f, state: .oscString, action: .oscPut, next: .oscString)
        table.add (codes: [0x9c, 0x1b, 0x18, 0x1a, 0x07], state: .oscString, action: .oscEnd, next: .ground)
        table.add (codes: r (low: 0x1c, high: 0x20), state: .oscString, action: .ignore, next: .oscString)
        // sos/pm/apc does nothing
        table.add (codes: [0x58, 0x5e, 0x5f], state: .escape, action: .ignore, next: .sosPmApcString)
        table.add (codes: printables, state: .sosPmApcString, action: .ignore, next: .sosPmApcString)
        table.add (codes: executables, state: .sosPmApcString, action: .ignore, next: .sosPmApcString)
        table.add (code: 0x9c, state: .sosPmApcString, action: .ignore, next: .ground)
        table.add (code: 0x7f, state: .sosPmApcString, action: .ignore, next: .sosPmApcString)
        // csi entries
        table.add (code: 0x5b, state: .escape, action: .clear, next: .csiEntry)
        table.add (codes: r (low: 0x40, high: 0x7f), state: .csiEntry, action: .csiDispatch, next: .ground)
        table.add (codes: r (low: 0x30, high: 0x3a), state: .csiEntry, action: .param, next: .csiParam)
        table.add (code: 0x3b, state: .csiEntry, action: .param, next: .csiParam)
        table.add (codes: [0x3c, 0x3d, 0x3e, 0x3f], state: .csiEntry, action: .collect, next: .csiParam)
        table.add (codes: r (low: 0x30, high: 0x3a), state: .csiParam, action: .param, next: .csiParam)
        table.add (code: 0x3b, state: .csiParam, action: .param, next: .csiParam)
        table.add (codes: r (low: 0x40, high: 0x7f), state: .csiParam, action: .csiDispatch, next: .ground)
        table.add (codes: [0x3c, 0x3d, 0x3e, 0x3f], state: .csiParam, action: .ignore, next: .csiIgnore)
        
        // csi for ":"
        table.add (code: 0x3a, state: .csiParam, action: .param, next: .csiParam)
        table.add (codes: r (low: 0x20, high: 0x40), state: .csiIgnore, action: .ignore, next: .csiIgnore)
        table.add (code: 0x7f, state: .csiIgnore, action: .ignore, next: .csiIgnore)
        table.add (codes: r (low: 0x40, high: 0x7f), state: .csiIgnore, action: .ignore, next: .ground)
        //table.Add (code: 0x3a, state: .CsiEntry, action: .Ignore, next: .CsiIgnore)
        table.add (codes: r (low: 0x20, high: 0x30), state: .csiEntry, action: .collect, next: .csiIntermediate)
        table.add (codes: r (low: 0x20, high: 0x30), state: .csiIntermediate, action: .collect, next: .csiIntermediate)
        table.add (codes: r (low: 0x30, high: 0x40), state: .csiIntermediate, action: .ignore, next: .csiIgnore)
        table.add (codes: r (low: 0x40, high: 0x7f), state: .csiIntermediate, action: .csiDispatch, next: .ground)
        table.add (codes: r (low: 0x20, high: 0x30), state: .csiParam, action: .collect, next: .csiIntermediate)
        // escIntermediate
        table.add (codes: r (low: 0x20, high: 0x30), state: .escape, action: .collect, next: .escapeIntermediate)
        table.add (codes: r (low: 0x20, high: 0x30), state: .escapeIntermediate, action: .collect, next: .escapeIntermediate)
        table.add (codes: r (low: 0x30, high: 0x7f), state: .escapeIntermediate, action: .escDispatch, next: .ground)
        table.add (codes: r (low: 0x30, high: 0x50), state: .escape, action: .escDispatch, next: .ground)
        table.add (codes: r (low: 0x51, high: 0x58), state: .escape, action: .escDispatch, next: .ground)
        table.add (codes: [0x59, 0x5a, 0x5c], state: .escape, action: .escDispatch, next: .ground)
        table.add (codes: r (low: 0x60, high: 0x7f), state: .escape, action: .escDispatch, next: .ground)
        // dcs entry
        table.add (code: 0x50, state: .escape, action: .clear, next: .dcsEntry)
        table.add (codes: executables, state: .dcsEntry, action: .ignore, next: .dcsEntry)
        table.add (code: 0x7f, state: .dcsEntry, action: .ignore, next: .dcsEntry)
        table.add (codes: r (low: 0x1c, high: 0x20), state: .dcsEntry, action: .ignore, next: .dcsEntry)
        table.add (codes: r (low: 0x20, high: 0x30), state: .dcsEntry, action: .collect, next: .dcsIntermediate)
        table.add (code: 0x3a, state: .dcsEntry, action: .ignore, next: .dcsIgnore)
        table.add (codes: r (low: 0x30, high: 0x3a), state: .dcsEntry, action: .param, next: .dcsParam)
        table.add (code: 0x3b, state: .dcsEntry, action: .param, next: .dcsParam)
        table.add (codes: [0x3c, 0x3d, 0x3e, 0x3f], state: .dcsEntry, action: .collect, next: .dcsParam)
        table.add (codes: executables, state: .dcsIgnore, action: .ignore, next: .dcsIgnore)
        table.add (codes: r (low: 0x20, high: 0x80), state: .dcsIgnore, action: .ignore, next: .dcsIgnore)
        table.add (codes: r (low: 0x1c, high: 0x20), state: .dcsIgnore, action: .ignore, next: .dcsIgnore)
        table.add (codes: executables, state: .dcsParam, action: .ignore, next: .dcsParam)
        table.add (code: 0x7f, state: .dcsParam, action: .ignore, next: .dcsParam)
        table.add (codes: r (low: 0x1c, high: 0x20), state: .dcsParam, action: .ignore, next: .dcsParam)
        table.add (codes: r (low: 0x30, high: 0x3a), state: .dcsParam, action: .param, next: .dcsParam)
        table.add (code: 0x3b, state: .dcsParam, action: .param, next: .dcsParam)
        table.add (codes: [0x3a, 0x3c, 0x3d, 0x3e, 0x3f], state: .dcsParam, action: .ignore, next: .dcsIgnore)
        table.add (codes: r (low: 0x20, high: 0x30), state: .dcsParam, action: .collect, next: .dcsIntermediate)
        table.add (codes: executables, state: .dcsIntermediate, action: .ignore, next: .dcsIntermediate)
        table.add (code: 0x7f, state: .dcsIntermediate, action: .ignore, next: .dcsIntermediate)
        table.add (codes: r (low: 0x1c, high: 0x20), state: .dcsIntermediate, action: .ignore, next: .dcsIntermediate)
        table.add (codes: r (low: 0x20, high: 0x30), state: .dcsIntermediate, action: .collect, next: .dcsIntermediate)
        table.add (codes: r (low: 0x30, high: 0x40), state: .dcsIntermediate, action: .ignore, next: .dcsIgnore)
        table.add (codes: r (low: 0x40, high: 0x7f), state: .dcsIntermediate, action: .dcsHook, next: .dcsPassthrough)
        table.add (codes: r (low: 0x40, high: 0x7f), state: .dcsParam, action: .dcsHook, next: .dcsPassthrough)
        table.add (codes: r (low: 0x40, high: 0x7f), state: .dcsEntry, action: .dcsHook, next: .dcsPassthrough)
        table.add (codes: executables, state: .dcsPassthrough, action: .dcsPut, next: .dcsPassthrough)
        table.add (codes: printables, state: .dcsPassthrough, action: .dcsPut, next: .dcsPassthrough)
        table.add (code: 0x7f, state: .dcsPassthrough, action: .ignore, next: .dcsPassthrough)
        table.add (codes: [0x1b, 0x9c], state: .dcsPassthrough, action: .dcsUnhook, next: .ground)
        table.add (code: NonAsciiPrintable, state: .oscString, action: .oscPut, next: .oscString)
        return table
    }
    
    // Array of parameters, and "collect" string
    typealias CsiHandler = ([Int],cstring) -> ()
    typealias CsiHandlerFallback = ([Int],cstring,UInt8) -> ()
    
    // String with payload
    typealias OscHandler = (ArraySlice<UInt8>) -> ()
    typealias OscHandlerFallback = (Int) -> ()
    
    // Collect + flag
    typealias EscHandler = (cstring, UInt8) -> ()
    typealias EscHandlerFallback = (cstring, UInt8) -> ()
    
    // Range of bytes to print out
    typealias PrintHandler = (ArraySlice<UInt8>) -> ()
    
    typealias ExecuteHandler = () -> ()
    
    // Handlers
    var csiHandlers: [UInt8:CsiHandler] = [:]
    var oscHandlers: [Int:OscHandler] = [:]
    var executeHandlers: [UInt8:ExecuteHandler] = [:]
    var escHandlers: [cstring:EscHandler] = [:]
    var dcsHandlers: [cstring:DcsHandler] = [:]
    var activeDcsHandler: DcsHandler? = nil
    var errorHandler: (ParsingState) -> ParsingState = { (state : ParsingState) -> ParsingState in return state; }
    
    var initialState: ParserState = .ground
    var currentState: ParserState = .ground
    
    // buffers over several calls
    var _osc: cstring
    var _pars: [Int]
    var _collect: cstring
    var printHandler: PrintHandler = { (slice : ArraySlice<UInt8>) -> () in }
    var printStateReset: () -> () = {  }
    
    var table: TransitionTable
    
    init ()
    {
        table = EscapeSequenceParser.buildVt500TransitionTable()
        _osc = []
        _pars = [0]
        _collect = []
        // "\"
        setEscHandler("\\", ParserEscHandlerFallback)
    }
    
    func ParserEscHandlerFallback (collect: cstring, flag: UInt8)
    {
    }
    
    var escHandlerFallback: EscHandlerFallback = { (collect: cstring, flag: UInt8) in
    }
    
    func setEscHandler (_ flag: String, _ callback: @escaping EscHandler)
    {
        escHandlers [Array (flag.utf8)] = callback
    }

    func setCsiHandler (_ flag: String, _ callback: @escaping CsiHandler)
    {
        csiHandlers [flag.first!.asciiValue!] = callback
    }
    
    func setDcsHandler (_ flag: String, _ callback: DcsHandler)
    {
        dcsHandlers [Array (flag.utf8)] = callback
    }

    var executeHandlerFallback : ExecuteHandler = { () -> () in
    }
    
    var csiHandlerFallback : CsiHandlerFallback = { (pars: [Int], collect: cstring, code: UInt8) -> () in
        print ("Cannot handle ESC-\(code)")
    }
    
    var oscHandlerFallback: OscHandlerFallback = { (code: Int) -> () in
        
    }
    
    func reset ()
    {
        currentState = initialState
        _osc = []
        _pars = [0]
        _collect = []
        activeDcsHandler = nil
        printStateReset()
    }

    var logFileCounter = 1
    func dump (_ data: ArraySlice<UInt8>)
    {
        let dir = "/tmp"
        let path = dir + "/log-\(logFileCounter)"
        do {
            let dataCopy = Data (data)
            try dataCopy.write(to: URL.init(fileURLWithPath: path))
            logFileCounter += 1
        } catch {
            // Ignore write error
            //print ("Got error while logging data dump to \(path)")
        }
    }
    
    func parse (data: ArraySlice<UInt8>)
    {
        var code : UInt8 = 0
        var transition : UInt8 = 0
        var error = false
        var currentState = self.currentState
        var print = -1
        var dcs = -1
        var osc = self._osc
        var collect = self._collect
        var pars = self._pars
        var dcsHandler = activeDcsHandler
        
        //dump (data)
            
        // process input string
        var i = data.startIndex
        let len = data.count
        while i < len {
            code = data [i]
            
            // 1f..80 are printable ascii characters
            // c2..f3 are valid utf8 beginning of sequence elements, and most importantly,
            // does not cover 0x90 which is the DCS initiator in 8 bit mode.
            
            // The nice code is commented out, because this ends up consuming valid utf8 code when
            // we are in the middle of things (force a small reading buffer to see more easily)
            if currentState == .ground && code > 0x1f  { // }(code > 0x1f && code < 0x80 || (code > 0xc2 && code < 0xf3)) {
                print = (~print != 0) ? print : i
                repeat {
                    i += 1
                } while i < len && data [i] > 0x1f
                continue;
            }
            
            // shortcut for CSI params
            if currentState == .csiParam && (code > 0x2f && code < 0x39) {
                let newV = pars [pars.count - 1] * 10 + Int(code) - 48
                
                // Prevent attempts at overflowing - crash 
                let willOverflow =  newV > ((Int.max/10)-10)
                pars [pars.count - 1] = willOverflow ? 0 : newV
                
                i += 1
                continue
            }
            
            // Normal transition and action loop
            transition = table [(Int(currentState.rawValue) << 8) | Int (UInt8 ((code < 0xa0 ? code : EscapeSequenceParser.NonAsciiPrintable)))]
            let action = ParserAction (rawValue: transition >> 4)!
            switch action {
            case .print:
                print = (~print != 0) ? print : i
            case .execute:
                if ~print != 0 {
                    printHandler (data [print..<i])
                    print = -1
                }
                if let callback = executeHandlers [code] {
                    callback ()
                } else {
                    // executeHandlerFallback (code)
                }
            case .ignore:
                // handle leftover print or dcs chars
                if ~print != 0 {
                    printHandler (data [print..<i])
                    print = -1
                } else if ~dcs != 0 {
                    dcsHandler?.put (data: data [dcs..<i])
                    dcs = -1
                }
            case .error:
                // chars higher than 0x9f are handled by this action
                // to keep the transition table small
                if code > 0x9f {
                    switch (currentState) {
                    case .ground:
                        print = (~print != 0) ? print : i;
                    case .csiIgnore:
                        transition |= ParserState.csiIgnore.rawValue;
                    case .dcsIgnore:
                        transition |= ParserState.dcsIgnore.rawValue;
                    case .dcsPassthrough:
                        dcs = (~dcs != 0) ? dcs : i;
                        transition |= ParserState.dcsPassthrough.rawValue;
                        break;
                    default:
                        error = true;
                        break;
                    }
                } else {
                    error = true;
                }
                // if we end up here a real error happened
                if error {
                    let state = ParsingState ()
                    state.position = i
                    state.code = code
                    state.currentState = currentState
                    state.print = print
                    state.dcs = dcs
                    state.osc = osc
                    state.collect = collect
                    let inject = errorHandler (state)
                    if inject.abort {
                        return;
                    }
                    error = false;
                }
            case .csiDispatch:
                // Trigger CSI handler
                if let handler = csiHandlers [code] {
                    handler (pars, collect)
                } else {
                    csiHandlerFallback (pars, collect, code)
                }
            case .param:
                if code == 0x3b || code == 0x3a {
                    pars.append (0)
                } else {
                    let newV = pars [pars.count - 1] * 10 + Int(code) - 48
                    
                    // Prevent attempts at overflowing - crash
                    let willOverflow =  newV > ((Int.max/10)-10)
                    pars [pars.count - 1] = willOverflow ? 0 : newV
                }
            case .escDispatch:
                if let handler = escHandlers [collect + [code]] {
                    handler (collect, code)
                } else {
                    escHandlerFallback(collect, code)
                }
            case .collect:
                collect.append (code)
            case .clear:
                if ~print != 0 {
                    printHandler (data [print..<i])
                    print = -1
                }
                osc = []
                pars = [0]
                collect = []
                dcs = -1
                printStateReset()
            case .dcsHook:
                if let dcs = dcsHandlers [collect + [code]] {
                    dcsHandler = dcs
                    dcs.hook (collect: collect, parameters: pars, flag: code)
                }
                // FIXME: perhaps have a fallback?
                break
            case .dcsPut:
                dcs = (~dcs != 0) ? dcs : i
            case .dcsUnhook:
                if let d = dcsHandler {
                    if ~dcs != 0 {
                        d.put (data: data[dcs..<i])
                        d.unhook ()
                        dcsHandler = nil
                    }
                }
                if code == 0x1b {
                    transition |= ParserState.escape.rawValue
                }
                osc = []
                pars = [0]
                collect = []
                dcs = -1
                printStateReset()
            case .oscStart:
                if ~print != 0 {
                    printHandler (data[print..<i])
                    print = -1
                }
                osc = []
            case .oscPut:
                var j = i
                while j < len {
                    let c = data [j]
                    if c == ControlCodes.BEL || c == ControlCodes.CAN || c == ControlCodes.ESC {
                        break
                    } else if c >= 0x20 {
                        osc.append (c)
                    }
                    j += 1
                }
                i = j - 1
            case .oscEnd:
                if osc.count != 0 && code != ControlCodes.CAN && code != ControlCodes.SUB {
                    // NOTE: OSC subparsing is not part of the original parser
                    // we do basic identifier parsing here to offer a jump table for OSC as well
                    var oscCode : Int
                    var content : ArraySlice<UInt8>
                    let semiColonAscii = 59 // ';'
                    
                    if let idx = osc.firstIndex (of: UInt8(semiColonAscii)){
                        oscCode = EscapeSequenceParser.parseInt (osc [0..<idx])
                        content = osc [(idx+1)...]
                    } else {
                        oscCode = EscapeSequenceParser.parseInt (osc[0...])
                        content = []
                    }
                    if let handler = oscHandlers [oscCode] {
                        handler (content)
                    } else {
                        oscHandlerFallback (oscCode)
                    }
                }
                if code == 0x1b {
                    transition |= ParserState.escape.rawValue
                }
                osc = []
                pars = [0]
                collect = []
                dcs = -1
                printStateReset()
            }
            currentState = ParserState (rawValue: transition & 15)!
            i += 1
        }
        // push leftover pushable buffers to terminal
        if currentState == .ground && (~print != 0) {
            printHandler (data [print..<len])
        } else if currentState == .dcsPassthrough && (~dcs != 0) && dcsHandler != nil {
            dcsHandler!.put (data: data [dcs..<len])
        }
        
        // save non pushable buffers
        _osc = osc
        _collect = collect
        _pars = pars
        
        // save active dcs handler reference
        activeDcsHandler = dcsHandler
        
        // save state
        
        self.currentState = currentState
    }
    
    static func parseInt (_ str: ArraySlice<UInt8>) -> Int
    {
        var result = 0
        for x in str {
            if x < 48 || x > 57 {
                return result
            }
            
            let newV = result * 10 + Int ((x - 48))
            let willOverflow =  newV > ((Int.max/10)-10)
            if willOverflow {
                return 0
            }
            result = newV
        }
        return result
    }
}
