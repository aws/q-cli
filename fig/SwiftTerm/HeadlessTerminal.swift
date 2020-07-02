//
//  HeadlessTerminal.swift
//  
//
//  Created by Miguel de Icaza on 4/5/20.
//
#if !os(iOS)
import Foundation

///
/// A `HeadlessTerminal` provides a terminal emulator that runs a local process, but the output does not go
/// anywhere.   You can use this to script applications and screen scrape the output for example, by accessing the
/// `terminal` from this class.
///
public class HeadlessTerminal : TerminalDelegate, LocalProcessDelegate {
    public private(set) var terminal: Terminal!
    var process: LocalProcess!
    var onEnd: (_ exitCode: Int32?) -> ()
    var dir: String?
    
    public init (queue: DispatchQueue? = nil, options: TerminalOptions = TerminalOptions.default, onEnd: @escaping (_ exitCode: Int32?) -> ())
    {
        self.onEnd = onEnd
        terminal = Terminal(delegate: self, options: options)
        process = LocalProcess(delegate: self, dispatchQueue: queue)
    }
    
    public func processTerminated(_ source: LocalProcess, exitCode: Int32?) {
        onEnd (exitCode)
    }
    
    public func dataReceived(slice: ArraySlice<UInt8>) {
        //print (String (bytes: slice, encoding: .utf8))
        terminal.feed(buffer: slice)
    }
    
    func send(data: ArraySlice<UInt8>) {
        process.send (data: data)
    }

    func send(_ text: String) {
        send (data: ([UInt8] (text.utf8))[...])
        
    }

    public func send(source: Terminal, data: ArraySlice<UInt8>) {
        send (data: data)
    }
    

    public func getWindowSize() -> winsize {
        return winsize(ws_row: UInt16(terminal.rows), ws_col: UInt16(terminal.cols), ws_xpixel: UInt16 (16), ws_ypixel: UInt16 (16))
    }
    
    public func mouseModeChanged(source: Terminal) {
    }

    public func hostCurrentDirectoryUpdated(source: Terminal) {
        dir = source.hostCurrentDirectory
    }
    public func colorChanged(source: Terminal, idx: Int) {
    }
}

#endif
