//
//  MacLocalTerminalView.swift
//
//
//  Created by Miguel de Icaza on 3/6/20.
//

#if os(OSX)
import Foundation

public protocol LocalProcessTerminalViewDelegate {
    /**
     * This method is invoked to notify that the terminal has been resized to the specified number of columns and rows
     * the user interface code might try to adjut the containing scroll view, or if it is a toplevel window, the window itself
     * - Parameter source: the sending instance
     * - Parameter newCols: the new number of columns that should be shown
     * - Parameter newRow: the new number of rows that should be shown
     */
    func sizeChanged(source: LocalProcessTerminalView, newCols: Int, newRows: Int)

    /**
     * This method is invoked when the title of the terminal window should be updated to the provided title
     * - Parameter source: the sending instance
     * - Parameter title: the desired title
     */
    func setTerminalTitle(source: LocalProcessTerminalView, title: String)

    /**
     * This method will be invoked when the child process started by `startProcess` has terminated.
     * - Parameter source: the local process that terminated
     * - Parameter exitCode: the exit code returned by the process, or nil if this was an error caused during the IO reading/writing
     */
    func processTerminated (source: TerminalView, exitCode: Int32?)
}

/**
 * `LocalProcessTerminalView` is an AppKit NSView that can be used to host a local process
 * the process is launched inside a pseudo-terminal.
 *
 * Call the `startProcess` to launch the underlying process inside a pseudo terminal.
 *
 * Generally, for the `LocalProcessTerminalView` to be useful, you will want to disable the sandbox
 * for your application, otherwise the underlying shell will not have access to much - not the majority of
 * commands, not assorted places on the file systems and so on.   For this, you need to disable for your
 * target in "Signing and Capabilities" the sandbox entirely.
 *
 * Note: instances of `LocalProcessTerminalView` will set the `TerminalView`'s `delegate`
 * property and capture and consume the messages.   The messages that are most likely needed for
 * consumer applications are reposted to the `LocalProcessTerminalViewDelegate` in
 * `processDelegate`.   If you override the `delegate` directly, you might inadvertently break
 * the internal working of `LocalProcessTerminalView`.   If you must change the `delegate`
 * make sure that you proxy the values in your implementation to the values set after initializing this instance
 */
public class LocalProcessTerminalView: TerminalView, TerminalViewDelegate, LocalProcessDelegate {
    var process: LocalProcess!
    
    public override init (frame: CGRect)
    {
        super.init (frame: frame)
        setup ()
    }
    
    public required init? (coder: NSCoder)
    {
        super.init (coder: coder)
        setup ()
    }

    func setup ()
    {
        terminalDelegate = self
        process = LocalProcess (delegate: self)
    }
    
    /**
     * The `processDelegate` is used to deliver messages and information relevant t
     */
    public var processDelegate: LocalProcessTerminalViewDelegate?
    
    /**
     * This method is invoked to notify the client of the new columsn and rows that have been set by the UI
     */
    public func sizeChanged(source: TerminalView, newCols: Int, newRows: Int) {
        guard process.running else {
            return
        }
        var size = getWindowSize()
        let _ = PseudoTerminalHelpers.setWinSize(masterPtyDescriptor: process.childfd, windowSize: &size)
        
        processDelegate?.sizeChanged (source: self, newCols: newCols, newRows: newRows)
    }
    
    /**
     * Invoke this method to notify the processDelegate of the new title for the terminal window
     */
    public func setTerminalTitle(source: TerminalView, title: String) {
        processDelegate?.setTerminalTitle (source: self, title: title)
    }

    /**
     * This method is invoked when input from the user needs to be sent to the client
     */
    public func send(source: TerminalView, data: ArraySlice<UInt8>) 
    {
        process.send (data: data)
    }
    
    /**
     * Use this method to toggle the logging of data coming from the host, or pass nil to stop
     */
    public func setHostLogging (directory: String?)
    {
        process.setHostLogging (directory: directory)
    }
    
    public func scrolled(source: TerminalView, position: Double) {
        // noting
    }

    /**
     * Launches a child process inside a pseudo-terminal.
     * - Parameter executable: The executable to launch inside the pseudo terminal, defaults to /bin/bash
     * - Parameter args: an array of strings that is passed as the arguments to the underlying process
     * - Parameter environment: an array of environment variables to pass to the child process, if this is null, this picks a good set of defaults from `Terminal.getEnvironmentVariables`.
     */
    public func startProcess(executable: String = "/bin/bash", args: [String] = [], environment: [String]? = nil)
    {
        process.startProcess(executable: executable, args: args, environment: environment)
    }
    
    /**
     * Implements the LocalProcessDelegate method.
     */
    public func processTerminated(_ source: LocalProcess, exitCode: Int32?) {
        processDelegate?.processTerminated(source: self, exitCode: exitCode)
    }
    
    /**
     * Implements the LocalProcessDelegate.dataReceived method
     */
    public func dataReceived(slice: ArraySlice<UInt8>) {
        feed (byteArray: slice)
    }
    
    /**
     * Implements the LocalProcessDelegate.getWindowSize method
     */
    public func getWindowSize () -> winsize
    {
        let f: CGRect = self.frame
        return winsize(ws_row: UInt16(terminal.rows), ws_col: UInt16(terminal.cols), ws_xpixel: UInt16 (f.width), ws_ypixel: UInt16 (f.height))
    }
    
}

#endif
