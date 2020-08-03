//
//  ShellBridge.swift
//  fig
//
//  Created by Matt Schrage on 4/18/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa
import OSLog

protocol ShellBridgeEventListener {
    func recievedDataFromPipe(_ notification: Notification)
    func recievedUserInputFromTerminal(_ notification: Notification)
    func recievedStdoutFromTerminal(_ notification: Notification)
    
    func recievedDataFromPty(_ notification: Notification)

}

extension Notification.Name {
    static let recievedDataFromPipe = Notification.Name("recievedDataFromPipe")
    static let recievedUserInputFromTerminal = Notification.Name("recievedUserInputFromTerminal")
    static let recievedStdoutFromTerminal = Notification.Name("recievedStdoutFromTerminal")
    static let recievedDataFromPty = Notification.Name("recievedDataFromPty")

}

protocol MouseMonitoring {
    func requestStopMonitoringMouseEvents(_ notification: Notification)
    func requestStartMonitoringMouseEvents(_ notification: Notification)
}

extension Notification.Name {
    static let requestStopMonitoringMouseEvents = Notification.Name("requestStopMonitoringMouseEvents")
    static let requestStartMonitoringMouseEvents = Notification.Name("requestStartMonitoringMouseEvents")
}

class ShellBridge {
    static let shared = ShellBridge()
    let socketServer: WebSocketServer = WebSocketServer.bridge
    
    var pty: HeadlessTerminal = HeadlessTerminal(onEnd: { (code) in
        print("Exit")
    })
    var rawOutput = ""
    var streamHandlers: Set<String> = []
    var executeHandlers: Set<String> = []
    
    var previousFrontmostApplication: NSRunningApplication?
//    var socket: WebSocketConnection?
//    var socketServer: Process?
    init() {
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(setPreviousApplication(notification:)), name: NSWorkspace.didDeactivateApplicationNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(spaceChanged), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)
        
//        startPty(env: [:])
//        streamInPty(command: "sftp mschrage_mschrage-static@ssh.phx.nearlyfreespeech.net", handlerId: "abcde")
//        executeInPty(command: "ls -1aF", handlerId: "abcdef")
//        let term = HeadlessTerminal { (exit) in
//            print("Exit:\(exit)")
//        }
//        term.process.debugIO = true
//
//        term.process.startProcess(executable: "/bin/zsh", args: [], environment: nil)
//
//        term.send("ls -1aF\r\n")
//        term.send("\n\n")
//        term.send("ms\r")
//        Timer.delayWithSeconds(1) {
//            term.send("M@tthew1!\r")
//            Timer.delayWithSeconds(10) {
//                term.send("ls\r")
//
//            }
//        }

//        kill(term.process.shellPid, SIGTERM)

//        term.send("pwd\r")
//        term.send("cd ~\r")
//        term.send("pwd\r")
//        term.send(data: [0x3])
//        term.send(data: [0x4])

        

//        term.process.processTerminated()
//        signal(SIGTERM) { (<#Int32#>) in
//            <#code#>
//        }


//        term.process.processTerminated()



        // start Python server script 
//        "python3 /path/to/executable/fig.py utils:ws-start".runAsCommand()
        self.startWebSocketServer()


    }
    
    func startPty(env: [String: String]) {
//        if (pty.process.running) {
//            print("Closing old PTY...")
//            streamHandlers = []
//            executeHandlers = []
//            pty.send(data: [0x4])
//            kill(pty.process.shellPid, SIGTERM)
//            return
//        }
        print("Start PTY")

        let shell = env["SHELL"] ?? "/bin/sh"
        let rawEnv = env.reduce([]) { (acc, elm) -> [String] in
            let (key, value) = elm
            return acc + ["\(key)=\(value)"]
        }
        
        pty.process.startProcess(executable: shell, args: [], environment: rawEnv.count == 0 ? nil : rawEnv)
        pty.process.delegate = self
        pty.send("unset HISTFILE\r")
    }
    
    let executeDelimeter = "-----------------"
    func executeInPty(command: String, handlerId:String) {
        executeHandlers.insert(handlerId)
        let cmd = "printf \"<<<\" ; echo \"\(executeDelimeter)\(handlerId)\(executeDelimeter)\" ; \(command) ; echo \"\(executeDelimeter)\(handlerId)\(executeDelimeter)>>>\"\r"
        pty.send(cmd)
        print("Execute PTY command: \(command)")

    }

    let streamDelimeter = "================="
    func streamInPty(command: String, handlerId:String) {
//        streamHandlers.insert(handlerId)
        let cmd = "printf \"<<<\" ; echo \"\(streamDelimeter)\(handlerId)\(streamDelimeter)\" ; \(command) ; echo \"\(streamDelimeter)\(handlerId)\(streamDelimeter)>>>\"\r"
        pty.send(cmd)
        print("Stream PTY command: \(command)")

    }
    
    //http://www.physics.udel.edu/~watson/scen103/ascii.html
    enum ControlCode : String {
        typealias RawValue = String
        case EOT = "^D"
        case ETX = "^C"
        
    }
    func writeInPty(command: String, control: ControlCode? = nil) {
        
        if let code = control {
            print("Write PTY controlCode: \(code.rawValue)")
            switch code {
            case .EOT:
                pty.send(data: [0x4])
            case .ETX:
                pty.send(data: [0x3])
            }
        } else {
            print("Write PTY command: \(command)")
            pty.send("\(command)\r")
        }

    }
    
    func closePty() {
        streamHandlers = []
        executeHandlers = []
        if (pty.process.running) {
            pty.send(data: [0x4])
            pty.send(data: [0x4])
            pty.send(data: [0x4])
            pty.send(data: [0x4])
            pty.send(data: [0x4])
            pty.send(data: [0x4])
            pty.send(data: [0x4])
            pty.send(data: [0x4])

            kill(pty.process.shellPid, SIGTERM)
        }
    }
    
    func startWebSocketServer() {
//        self.socketServer = WebSocketServer.bridge
    }
    
    func stopWebSocketServer( completion:(() -> Void)? = nil) {
        if let completion = completion {
         completion()
        }
    }
    
    // This fixes an issue where focus would bounce to an application in the previous workspace. Essentially this resets previous application anytime the workspace is changed.
    
    @objc func spaceChanged() {
        self.previousFrontmostApplication = NSWorkspace.shared.frontmostApplication
//        let windowNumbers = NSWindow.windowNumbersWithOptions( NSWindowNumberListAllSpaces | NSWindowNumberListAllApplications as NSWindowNumberListOptions )
        
        let windows = NSWindow.windowNumbers(options: [.allApplications, .allSpaces])
        print(windows)
    }
    
    @objc func setPreviousApplication(notification: NSNotification!) {
        self.previousFrontmostApplication = notification!.userInfo![NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
        print("Deactivated:", self.previousFrontmostApplication?.bundleIdentifier ?? "")
    }

    static func injectStringIntoTerminal(_ cmd: String, runImmediately: Bool = false, completion: (() -> Void)? = nil) {
        if (NSWorkspace.shared.frontmostApplication?.isFig ?? false) {
            WindowServer.shared.returnFocus()
        }
//            ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
        NotificationCenter.default.post(name: .requestStopMonitoringMouseEvents, object: nil)
        print("Stop monitoring mouse")
//        NSApp.deactivate()
            Timer.delayWithSeconds(0.2) {
            if let currentApp = NSWorkspace.shared.frontmostApplication {
//        if let currentApp = ShellBridge.shared.previousFrontmostApplication {
               if (currentApp.isTerminal) {
                   // save current pasteboard
//                   currentApp.activate(options: .activateIgnoringOtherApps)
                   let pasteboard = NSPasteboard.general
                   let copiedString = pasteboard.string(forType: .string) ?? ""
                   
                   // add our script to pasteboard
                   NSPasteboard.general.clearContents()
                   NSPasteboard.general.setString(cmd, forType: .string)
                   print(pasteboard.string(forType: .string) ?? "")
                       // Be careful: in some apps, CMD-Enter toggles fullscreen
                self.simulate(keypress: .ctrlE)
                self.simulate(keypress: .ctrlU)

                       self.simulate(keypress: .cmdV)
                print("CMD-V")
                Timer.delayWithSeconds(0.1) {
                            if (runImmediately) {
                                print("ENTER")
                                self.simulate(keypress: .enter)
                            } else {
                                self.simulate(keypress: .rightArrow)
                            }

                            Timer.delayWithSeconds(0.10) {
                                NotificationCenter.default.post(name: .requestStartMonitoringMouseEvents, object: nil)
                                print("Start monitoring mouse")

                            }

                            if let completion = completion {
                                completion()
                            }
                       }
    
                   // need delay so that terminal responds
                Timer.delayWithSeconds(0.5) {
                       // restore pasteboard
                       NSPasteboard.general.clearContents()
                       pasteboard.setString(copiedString, forType: .string)
                   }
               }
           }
        }
    }
    
    //https://gist.github.com/eegrok/949034
    enum Keypress: UInt16 {
        case cmdV = 9
        case enter = 36
        case rightArrow = 124
        case ctrlE = 14
        case ctrlU = 32

    }
    
    static func simulate(keypress: Keypress) {
        let keyCode = keypress.rawValue as CGKeyCode
        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let keydown = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: true)
        let keyup = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: false)
        
        if (keypress == .cmdV){
            keydown?.flags = CGEventFlags.maskCommand;
        }
        
        if (keypress == .ctrlE || keypress == .ctrlU) {
            keydown?.flags = CGEventFlags.maskControl;
        }
        
        let loc = CGEventTapLocation.cghidEventTap
        keydown?.post(tap: loc)
        keyup?.post(tap: loc)
    }
}

struct PtyMessage: Codable {
    var type: String
    var handleId: String
    var output: String
}

struct ShellMessage: Codable {
    var type: String
    var source: String
    var session: String
    var env: String?
    var io: String?
    var data: String
    var options: [String]?

}

extension ShellBridge : LocalProcessDelegate {
    func processTerminated(_ source: LocalProcess, exitCode: Int32?) {
        print("Exited...\(exitCode ?? 0)")
    }
    
    func dataReceived(slice: ArraySlice<UInt8>) {
        
        let data = String(bytes: slice, encoding: .utf8) ?? ""
        print(data)
        
        
        for handle in streamHandlers {
            var ping = ""
            let header = data.components(separatedBy: "<<<\(streamDelimeter)\(handle)\(streamDelimeter)")
            if header.count == 2 {
                ping += header[1]
            } else {
                ping = data
            }
            
            let tail = ping.components(separatedBy: "\(streamDelimeter)\(handle)\(streamDelimeter)>>>")
            
            if tail.count == 2 {
                ping = tail[0]
                streamHandlers.remove(handle)
                rawOutput = ""
            }
            
            print(handle, ping)
            let msg = PtyMessage(type: "stream", handleId: handle, output: ping)
            NotificationCenter.default.post(name: .recievedDataFromPty, object: msg)
            
            
        }
        
        if let streamCandidate = data.groups(for:"<<<\(streamDelimeter)(.*?)\(streamDelimeter)")[safe: 0] {
            streamHandlers.insert(streamCandidate[1])
        }
        
        rawOutput += data

        for handle in executeHandlers {
            let groups = rawOutput.groups(for: "(?s)<<<\(executeDelimeter)\(handle)\(executeDelimeter)(.*?)\(executeDelimeter)\(handle)\(executeDelimeter)>>>")
            
            if let group = groups[safe: 0], let output = group.last {
                executeHandlers.remove(handle)
                rawOutput = ""
                print(handle, output)
                let msg = PtyMessage(type: "execute", handleId: handle, output: output)
                NotificationCenter.default.post(name: .recievedDataFromPty, object: msg)

            }

        }
        


        
//        print(data)
    }
        
    func getWindowSize() -> winsize {
        return winsize(ws_row: UInt16(60), ws_col: UInt16(50), ws_xpixel: UInt16 (16), ws_ypixel: UInt16 (16))
    }
    
    
}

class Integrations {
    static let terminals: Set = ["com.googlecode.iterm2",
                                 "com.apple.Terminal",
                                 "io.alacritty",
                                 "co.zeit.hyper",
                                "net.kovidgoyal.kitty"]
    static let browsers:  Set = ["com.google.Chrome"]
    static let editors:   Set = ["com.apple.dt.Xcode",
                                 "com.sublimetext.3",
                                 "com.microsoft.VSCode"]
    static var allowed: Set<String> {
        get {
            if let allowed = UserDefaults.standard.string(forKey: "allowedApps") {
                return Set(allowed.split(separator: ",").map({ String($0)}))
            } else {
                return []
            }
        }
    }
    
    static var blocked: Set<String> {
        get {
           if let allowed = UserDefaults.standard.string(forKey: "blockedApps") {
               return Set(allowed.split(separator: ",").map({ String($0)}))
           } else {
               return []
           }
       }
    }
    static var whitelist: Set<String> {
        get {
            return Integrations.terminals
            .union(Integrations.allowed)
      .subtracting(Integrations.blocked)
        }
    }
}

extension Timer {
    class func delayWithSeconds(_ seconds: Double, completion: @escaping () -> ()) {
        DispatchQueue.main.asyncAfter(deadline: .now() + seconds) {
            completion()
        }
    }
}

extension NSRunningApplication {
    var isTerminal: Bool {
        get {
            return  Integrations.terminals.contains(self.bundleIdentifier ?? "")
        }
    }
    
    var isBrowser: Bool {
        get {
            return  Integrations.browsers.contains(self.bundleIdentifier ?? "")
        }
    }
    
    var isEditor: Bool {
        get {
            return  Integrations.editors.contains(self.bundleIdentifier ?? "")
        }
    }
    var isFig: Bool {
        get {
            return  self.bundleIdentifier ?? "" == "com.mschrage.fig"
        }
    }
}

extension ShellBridge {
    // fig search hello there -url  -> https://withfig.com/web/hello/there?
    static func commandLineOptionsToURL(_ options: [String]) -> URL {
        var root = ""
        
        var endOfPathIndex = 0;
        for value in options {
            let isFlag = value.starts(with: "-")
            if isFlag {
                break
            }
            root += "/\(value)"
            endOfPathIndex += 1;
        }
        
        let flags: [String] = Array(options.suffix(from: endOfPathIndex))
        let pairs = flags.chunked(into: 2)
        let keys   = pairs.map { $0.first!.trimmingCharacters(in: CharacterSet.init(charactersIn: "-")) }
        let values = pairs.map { $0.last! }
        
        var query: [String: String] = [:]

        for (index, key) in keys.enumerated() {
            query[key] = values[index]
        }
        
        var components = URLComponents()
        components.scheme = Remote.baseURL.scheme ?? "https"
        components.host = Remote.baseURL.host ?? "app.withfig.com"
        components.port = Remote.baseURL.port
        components.path = root
        components.queryItems = query.map {
             URLQueryItem(name: $0, value: $1)
        }
        return components.url!
    }
    
    static func commandLineOptionsToRawURL(_ options: [String]) -> URL {
        var cmd = ""
        var raw:[String] = []
        if (options.count > 0) {
            cmd = "/\(options.first!)"
            raw = Array(options.suffix(from: 1))
        }
        
        let argv = raw.joined(separator: " ").addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? ""
        var components = URLComponents()
        components.scheme = Remote.baseURL.scheme ?? "https"
        components.host = Remote.baseURL.host ?? "app.withfig.com"
        components.port = Remote.baseURL.port
        components.path = cmd
        components.queryItems = [URLQueryItem(name: "input", value: argv)]
        return components.url!//URL(string:"\(components.string!)?input=\(argv)")!
    }
    
    // https://app.withfig.com/alias?fmt=echo%20whoami&input=values,hello
    static func aliasToRawURL(_ format: String, options: [String]) -> URL {
        
        let argv = options.joined(separator: " ").addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? ""
        let fmt = format.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? ""
        var components = URLComponents()
        components.scheme = Remote.baseURL.scheme ?? "https"
        components.host = Remote.baseURL.host ?? "app.withfig.com"
        components.port = Remote.baseURL.port
        components.path = "/fig_template"
        components.queryItems = [
            URLQueryItem(name: "fmt", value: fmt),
            URLQueryItem(name: "input", value: argv)
        ]
        return components.url!
    }
}

extension Array {
    func chunked(into size: Int) -> [[Element]] {
        return stride(from: 0, to: count, by: size).map {
            Array(self[$0 ..< Swift.min($0 + size, count)])
        }
    }
}

extension ShellBridge {
    static func symlinkCLI(completion: (()-> Void)? = nil){
//        cmd="tell application \"Terminal\" to do script \"uptime\""
//          osascript -e "$cmd"
        
        if let path = Bundle.main.path(forAuxiliaryExecutable: "figcli") {
            print(path)
            let _ = "mkdir -p /usr/local/bin && ln -sf '\(path)' '/usr/local/bin/fig'".runWithElevatedPriviledgesFromAppleScript()
            return
        }
        if let path = Bundle.main.path(forAuxiliaryExecutable: "figcli") {//Bundle.main.path(forResource: "fig", ofType: "", inDirectory: "dist") {
            print(path)
            let script = "mkdir -p /usr/local/bin && ln -sf '\(path)' '/usr/local/bin/fig'"
            
            let out = "cmd=\"do shell script \\\"\(script)\\\" with administrator privileges\" && osascript -e \"$cmd\"".runInBackground(completion: completion)
            
            print(out)
            //let _ = "test -f ~/.bash_profile && echo \"fig init #start fig pty\" >> ~/.bash_profile".runAsCommand()
            //let _ = "test -f ~/.zprofile && echo \"fig init #start fig pty\" >> ~/.zprofile".runAsCommand()
            //let _ = "test -f ~/.profile && echo \"fig init #start fig pty\" >> ~/.profile".runAsCommand()
   

        } else {
            print("couldn't find 'fig' cli executable")
            os_log("couldn't find 'fig' cli executable", log: OSLog.socketServer, type: .error)

        }

    }
    
    static func promptForAccesibilityAccess() {
            //get the value for accesibility
            let checkOptPrompt = kAXTrustedCheckOptionPrompt.takeUnretainedValue() as NSString
            //set the options: false means it wont ask
            //true means it will popup and ask
            let options = [checkOptPrompt: true]
            //translate into boolean value
            let accessEnabled = AXIsProcessTrustedWithOptions(options as CFDictionary?)
            print(accessEnabled)
    }
}
