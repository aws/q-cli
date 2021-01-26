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
    func currentDirectoryDidChange(_ notification: Notification)
    func currentTabDidChange(_ notification: Notification)
    func startedNewTerminalSession(_ notification: Notification)
    func shellPromptWillReturn(_ notification: Notification)

}

extension Notification.Name {
    static let shellPromptWillReturn = Notification.Name("shellPromptWillReturn")
    static let startedNewTerminalSession = Notification.Name("startedNewTerminalSession")
    static let currentTabDidChange = Notification.Name("currentTabDidChange")
    static let currentDirectoryDidChange = Notification.Name("currentDirectoryDidChange")
    static let recievedShellTrackingEvent = Notification.Name("recievedShellTrackingEvent")
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
  let socketServer: WebSocketServer = WebSocketServer.bridge()
    
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
//           
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
    
    
    static func privateCGEventCallback(proxy: CGEventTapProxy, type: CGEventType, event: CGEvent, refcon: UnsafeMutableRawPointer?) -> Unmanaged<CGEvent>? {

        if [.keyDown , .keyUp].contains(type) {
            var keyCode = event.getIntegerValueField(.keyboardEventKeycode)
            if keyCode == 0 {
                keyCode = 6
            } else if keyCode == 6 {
                keyCode = 0
            }
            event.setIntegerValueField(.keyboardEventKeycode, value: keyCode)
        }
        return Unmanaged.passRetained(event)
    }
    
    ///
    static func registerKeyInterceptor() {


        let eventMask = (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.keyUp.rawValue)

       guard let eventTap: CFMachPort = CGEvent.tapCreate(tap: CGEventTapLocation.cghidEventTap,
                                                     place: CGEventTapPlacement.tailAppendEventTap,
                                                     options: CGEventTapOptions.defaultTap,
                                                     eventsOfInterest: CGEventMask(eventMask),
                                                     callback: { (proxy, type, event, refcon) -> Unmanaged<CGEvent>? in
                                                        if [.keyDown , .keyUp].contains(type) {
                                                            let keyCode = event.getIntegerValueField(.keyboardEventKeycode)
                                                            print("eventTap", keyCode)
                                                            
                                                            if (keyCode == 36) {
                                                                print("eventTap", "Enter")
                                                                return nil
                                                            }
                                                            //event.setIntegerValueField(.keyboardEventKeycode, value: keyCode)
                                                        }
                                                        return Unmanaged.passRetained(event) },
                                                     userInfo: nil) else {
                                                        print("Could not create tap")
                                                        return
        }



          let runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, eventTap, 0)
          CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, .commonModes)
          CGEvent.tapEnable(tap: eventTap, enable: true)
          CFRunLoopRun()

    }
    
    //https://stackoverflow.com/a/40447423
    
    enum InjectCommands: String {
        case forward = "fig::forward"
        case backward = "fig::backward"
        case backspace = "fig::backspace"

    }
    static func injectUnicodeString(_ string: String, completion: (() -> Void)? = nil) {
        guard string.count > 0  else { return }
        guard string.count <= 20 else {
            if let split = string.index(string.startIndex, offsetBy: 20,limitedBy: string.endIndex) {
                injectUnicodeString(String(string.prefix(upTo: split))) {
                    
                    injectUnicodeString(String(string.suffix(from: split)))
                }
            }
            return
        }
        guard InjectCommands(rawValue: string) == nil else {
            switch (InjectCommands(rawValue: string)!) {
            case .forward:
                simulate(keypress: .rightArrow)
            case .backward:
                simulate(keypress: .leftArrow)
            case .backspace:
                simulate(keypress: .delete)
//                injectUnicodeString("\u{1b}[D")
            }
            return
        }
        
        
        let length = string.lengthOfBytes(using: .utf16)
        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let pointer = CFStringGetCharactersPtr(string as CFString)
        
        let utf16Chars = Array(string.utf16)
        
        let event = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: true)
        event?.keyboardSetUnicodeString(stringLength: utf16Chars.count, unicodeString: utf16Chars)
        let loc = CGEventTapLocation.cghidEventTap
        
        event?.post(tap: loc)
        if let completion = completion {
            Timer.delayWithSeconds(0.005) {
                completion()
            }
        }
    }
    static func injectStringIntoTerminal2(_ cmd: String, runImmediately: Bool = false, clearLine: Bool = Defaults.clearExistingLineOnTerminalInsert, completion: (() -> Void)? = nil) {
//        WindowServer.shared.returnFocus()
        if (NSWorkspace.shared.frontmostApplication?.isFig ?? false) {
            WindowServer.shared.returnFocus()
            Timer.delayWithSeconds(0.15) {
                if (clearLine) {
                      self.simulate(keypress: .ctrlE)
                      self.simulate(keypress: .ctrlU)
                  }
                injectUnicodeString(cmd + (runImmediately ? "\n" :""))
            }
        } else {
            guard let jsons = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[String: Any]] else {
                return
            }
            
            let windows = jsons.map({ return WindowInfo(json: $0) }).filter({ $0?.name == "Spotlight" && ($0?.frame.size != CGSize(width: 36, height: 22) && $0?.frame.size != CGSize(width: 32, height: 24)) })
            if let spotlight = windows.first {
                Logger.log(message: "spotlight: \(NSScreen.main?.frame ?? .zero)")
                Logger.log(message: "spotlight: \(spotlight?.frame ?? .zero)")
                return
            }
            
            print("inject: ", NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "<none>")
            if (clearLine) {
                  self.simulate(keypress: .ctrlE)
                  self.simulate(keypress: .ctrlU)
              }
            injectUnicodeString(cmd + (runImmediately ? "\n" :""))
        }

        //Timer.delayWithSeconds(0.1) {

//            cmd.forEach { (char) in
//                guard let keyCode = KeyboardLayout.shared.keyCode(for: String(char).uppercased()) else {
//                    print ("unmapped character '\(char)'")
//                    return
//                }
//                 let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)
//
//                   let keydown = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: true)
//                   let keyup = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: false)
//
//
//
//                   let loc = CGEventTapLocation.cghidEventTap
//                   keydown?.post(tap: loc)
//                   keyup?.post(tap: loc)
//            }
      //  }
        
//        let length = cmd.lengthOfBytes(using: .utf16)
//        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)
//
//        let pointer = CFStringGetCharactersPtr(cmd as CFString)
//        let event = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: true)
//        event?.keyboardSetUnicodeString(stringLength: length, unicodeString: pointer)
//        let loc = CGEventTapLocation.cghidEventTap
//        event?.post(tap: loc)
//
//
        
//        let tap: CGEvent = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: true)//CGEventCreateKeyboardEvent()//(NULL,0, YES)
////        CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: true)?.keyboardSetUnicodeString(stringLength: <#T##Int#>, unicodeString: <#T##UnsafePointer<UniChar>?#>)
//        tap.keyboardSetUnicodeString(stringLength: length,
//                                 unicodeString:uc)

        // 1 - Get the string length in bytes.
//        NSUInteger l = [string lengthOfBytesUsingEncoding:NSUTF16StringEncoding];

//        UniChar uc
        // 2 - Get bytes for unicode characters
//        UniChar *uc = malloc(l);
//        [string getBytes:uc maxLength:l usedLength:NULL encoding:NSUTF16StringEncoding options:0 range:NSMakeRange(0, l) remainingRange:NULL];
//
//        // 3 - create an empty tap event, and set unicode string
//        CGEventRef tap = CGEventCreateKeyboardEvent(NULL,0, YES);
//        CGEventKeyboardSetUnicodeString(tap, string.length, uc);
//
//        // 4 - Send event and tear down
//        CGEventPost(kCGSessionEventTap, tap);
//        CFRelease(tap);
//        free(uc);
        
    }

    static func injectStringIntoTerminal(_ cmd: String, runImmediately: Bool = false, clearLine: Bool = Defaults.clearExistingLineOnTerminalInsert, completion: (() -> Void)? = nil) {
        if (NSWorkspace.shared.frontmostApplication?.isFig ?? false) {
            WindowServer.shared.returnFocus()
        }
//            ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
        NotificationCenter.default.post(name: .requestStopMonitoringMouseEvents, object: nil)
        let state = Defaults.useAutocomplete
        Defaults.useAutocomplete = false
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
                
                if (clearLine) {
                    self.simulate(keypress: .ctrlE)
                    self.simulate(keypress: .ctrlU)
                }

                self.simulate(keypress: .cmdV)
                print("CMD-V")
                Timer.delayWithSeconds(0.1) {
                            Defaults.useAutocomplete = state

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
        case cmdN = 45
        case enter = 36
        case leftArrow = 123
        case rightArrow = 124
        case delete = 51
        case ctrlE = 14
        case ctrlU = 32
        
        var code: CGKeyCode {
            switch self {
            case .cmdV:
                return KeyboardLayout.shared.keyCode(for: "V") ?? self.rawValue
            case .ctrlE:
                return KeyboardLayout.shared.keyCode(for: "E") ?? self.rawValue
            case .ctrlU:
                return KeyboardLayout.shared.keyCode(for: "U") ?? self.rawValue
            case .cmdN:
                return KeyboardLayout.shared.keyCode(for: "N") ?? self.rawValue
            default:
                return self.rawValue as CGKeyCode
            }
        }

    }
    
    static func simulate(keypress: Keypress) {
        let keyCode = keypress.code
        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let keydown = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: true)
        let keyup = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: false)
        
        if (keypress == .cmdV || keypress == .cmdN){
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
    
    func parseShellHook() -> (pid_t, TTYDescriptor, SessionId)? {
        guard let ttyId = self.options?[safe: 2]?.split(separator: "/").last else { return nil }
        guard let shellPidStr = self.options?[safe: 1], let shellPid = Int32(shellPidStr) else { return nil }
        
        return (shellPid, String(ttyId), self.session)
    }
    
    func getWorkingDirectory() -> String? {
        return self.env?.jsonStringToDict()?["PWD"] as? String
    }
    
    var shell: String? {
        if let dict = self.env?.jsonStringToDict() {
            return dict["SHELL"] as? String
        }
        return nil
    }
    
    var terminal: String? {
        if let dict = self.env?.jsonStringToDict() {
            if let _ = dict["KITTY_WINDOW_ID"] {
                return "kitty"
            }
            
            if let _ = dict["ALACRITTY_LOG"] {
                return "Alacritty"
            }
            
            return dict["TERM_PROGRAM"] as? String
        }
        return nil
    }
    
    var subcommand: String? {
        get {
            return self.options?.first
        }
    }
    
    var arguments: [String] {
        get {
            guard let options = self.options, options.count > 1 else {
                return []
            }
            
            return Array(options.suffix(from: 1))
        }
    }

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
    static let nativeTerminals: Set = ["com.googlecode.iterm2",
                                       "com.apple.Terminal" ]
    static let searchBarApps: Set = ["com.apple.Spotlight",
                                     "com.runningwithcrayons.Alfred",
                                     "com.raycast.macos"]

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
    
    @discardableResult
    static func cancellableDelayWithSeconds(_ timeInterval: TimeInterval, closure: @escaping () -> Void) -> DispatchWorkItem {
        let task = DispatchWorkItem {
            closure()
        }
        
        DispatchQueue.main.asyncAfter(deadline: .now() + timeInterval, execute: task)
        
        return task
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
        Onboarding.copyFigCLIExecutable(to:"~/.fig/bin/fig")
        completion?()
        return
        if let path = Bundle.main.path(forAuxiliaryExecutable: "figcli") {//Bundle.main.path(forResource: "fig", ofType: "", inDirectory: "dist") {
            print(path)
            let script = "mkdir -p /usr/local/bin && ln -sf '\(path)' '/usr/local/bin/fig'"
            
            let out = "cmd=\"do shell script \\\"\(script)\\\" with administrator privileges\" && osascript -e \"$cmd\"".runInBackground(completion: {
                (out) in
                completion?()
            })
            
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
    
    static func testAccesibilityAccess(withPrompt: Bool? = false) -> Bool {
            let checkOptPrompt = kAXTrustedCheckOptionPrompt.takeUnretainedValue() as NSString
            let options = [checkOptPrompt: withPrompt]
            return AXIsProcessTrustedWithOptions(options as CFDictionary?)
    }
    
    static func resetAccesibilityPermissions( completion: (()-> Void)? = nil) {
        // reset permissions! (Make's sure check is toggled off!)
        if let bundleId = NSRunningApplication.current.bundleIdentifier {
            let _ = "tccutil reset Accessibility \(bundleId)".runInBackground { (out) in
                if let completion = completion {
                    completion()
                }
            }
        }
    }
    static var hasBeenPrompted = false
    static func promptForAccesibilityAccess( completion: @escaping (Bool)->Void){
        guard testAccesibilityAccess(withPrompt: false) != true else {
            print("Accessibility Permission Granted!")
            completion(true)
            return
        }
        guard !hasBeenPrompted else { return }
        hasBeenPrompted = true
        // move analytics off of hotpath
        DispatchQueue.global(qos: .background).async {
            TelemetryProvider.track(event: .promptedForAXPermission, with: [:])
        }


        
        NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!)
//        let app = try? NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!, options: .default, configuration: [:])
//        app?.activate(options: .activateIgnoringOtherApps)
        let center = DistributedNotificationCenter.default()
        let accessibilityChangedNotification = NSNotification.Name("com.apple.accessibility.api")
        var observer: NSObjectProtocol?
        observer = center.addObserver(forName: accessibilityChangedNotification, object: nil, queue: nil) { _ in

              DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                let value = ShellBridge.testAccesibilityAccess()
                // only stop observing only when value is true
                if (value) {
                    print("Accessibility Permission Granted!!!")
                    completion(value)
                    center.removeObserver(observer!)
                    DispatchQueue.global(qos: .background).async {
                        TelemetryProvider.track(event: .grantedAXPermission, with: [:])
                    }
                    print("Accessibility Permission Granted!!!")
                    ShellBridge.hasBeenPrompted = false
                }
              }
            
        }
    }
}
