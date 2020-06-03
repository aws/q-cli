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
}

extension Notification.Name {
    static let recievedDataFromPipe = Notification.Name("recievedDataFromPipe")
    static let recievedUserInputFromTerminal = Notification.Name("recievedUserInputFromTerminal")
    static let recievedStdoutFromTerminal = Notification.Name("recievedStdoutFromTerminal")

}

class ShellBridge {
    static let shared = ShellBridge()
    
    var previousFrontmostApplication: NSRunningApplication?
    var socket: WebSocketConnection?
    var socketServer: Process?
    init() {
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(setPreviousApplication(notification:)), name: NSWorkspace.didDeactivateApplicationNotification, object: nil)
        // start Python server script 
//        "python3 /path/to/executable/fig.py utils:ws-start".runAsCommand()
        self.startWebSocketServer()


    }
    
    @objc func setPreviousApplication(notification: NSNotification!) {
        self.previousFrontmostApplication = notification!.userInfo![NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
        print("Deactivated:", self.previousFrontmostApplication?.bundleIdentifier ?? "")
    }

    static func injectStringIntoTerminal(_ cmd: String, runImmediately: Bool = false, completion: (() -> Void)? = nil) {
            ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
            Timer.delayWithSeconds(0.3) {
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
                       self.simulate(keypress: .cmdV)
                Timer.delayWithSeconds(0.10) {
//                            self.simulate(keypress: .rightArrow)
                            self.simulate(keypress: .enter)
                            if let completion = completion {
                                completion()
                            }
                       }
    
                   // need delay so that terminal responds
                Timer.delayWithSeconds(1) {
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
    }
    
    static func simulate(keypress: Keypress) {
        let keyCode = keypress.rawValue as CGKeyCode
        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let keydown = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: true)
        let keyup = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: false)
        
        if (keypress == .cmdV){
            keydown?.flags = CGEventFlags.maskCommand;
        }
        
        let loc = CGEventTapLocation.cghidEventTap
        keydown?.post(tap: loc)
        keyup?.post(tap: loc)
    }
    
    func startWebSocketServer() {
        
        // check if websocket server is running -- this might happen if the app crashes
        
        let pid = "pgrep -f websocket".runAsCommand().trimmingCharacters(in: .whitespacesAndNewlines)
        if (pid.count > 0) {
            print("The websocket server is already running on \(pid) process.\n Shut it off with `pkill -f websocket`")
            
            let out = "pkill -f websocket".runAsCommand()
            print("Killed websocket:\(out)")

        }
        
        if let path = Bundle.main.path(forResource: "websocket", ofType: "", inDirectory: "wsdist") {
            print(path)
            print("Starting socket server")
            os_log("Starting socket server", log: OSLog.socketServer, type: .info)

            self.socketServer = path.runInBackground {
                  print("Closing the socket server")
            }
            os_log("Socket Server succesfully started...", log: OSLog.socketServer, type: .info)

            Timer.delayWithSeconds(0.5) {
                self.attemptToConnectToSocketServer()
            }
        } else {
            os_log("Couldn't start socket server", log: OSLog.socketServer, type: .error)
            print("couldn't start socket server")
        }
    }
    
    func stopWebSocketServer( completion:(() -> Void)? = nil) {
        if let server = self.socketServer {
            server.terminationHandler = { (proc) -> Void in
                print("socket server process terminated")
                os_log("socket server process terminated", log: OSLog.socketServer, type: .info)
                if let completion = completion {
                    completion()
                }
            }
            print("Terminating socket server...")

            server.terminate()

        } else {
            print("socket server is not running")
            os_log("socket server is not running", log: OSLog.socketServer, type: .error)

        }
    }
    
    func attemptToConnectToSocketServer() {
        self.socket = WebSocketTaskConnection.init(url: URL(string: "ws://localhost:8765")!)
        self.socket?.delegate = self;
        self.socket?.connect()
    }
}

extension ShellBridge: WebSocketConnectionDelegate {
    func onConnected(connection: WebSocketConnection) {
        print("connected")

        Timer.delayWithSeconds(1) {
            connection.send(text: "register_as_host")

        }
        
    }
    
    func onDisconnected(connection: WebSocketConnection, error: Error?) {
        print("disconnected")
        Timer.delayWithSeconds(5) {
            self.attemptToConnectToSocketServer()
        }

    }
    
    func onError(connection: WebSocketConnection, error: Error) {
        print(error)
        self.socket?.disconnect()
    }
    
    
    func onMessage(connection: WebSocketConnection, text: String) {
        let decoder = JSONDecoder()
        do {
            let msg = try decoder.decode(ShellMessage.self, from: text.data(using: .utf8)!)
//            print(msg)
            
            switch msg.type {
            case "pipe":
                NotificationCenter.default.post(name: .recievedDataFromPipe, object: msg)
            case "pty":
                if let io = msg.io {
                    if io == "i" {
                        NotificationCenter.default.post(name: .recievedUserInputFromTerminal, object: msg)
                    } else if io == "o" {
                        NotificationCenter.default.post(name: .recievedStdoutFromTerminal, object: msg)
                    }
                }
                
            default:
                print("Unhandled match from Websocket Message")
            }
//            if msg.type == "pipe" {
//                print(msg.data)
//                NotificationCenter.default.post(name: .recievedDataFromPipe, object: msg)
//            }
            
            

        } catch {
            print("oops: couldn't parse '\(text)'")
        }
        

    }
    
    func onMessage(connection: WebSocketConnection, data: Data) {
        print("msg:\(data)")
    }
    
    
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

class Integrations {
    static let terminals: Set = ["com.googlecode.iterm2",
                                 "com.apple.Terminal",
                                 "io.alacritty",
                                 "co.zeit.hyper"]
    static let browsers:  Set = ["com.google.Chrome"]
    static let editors:   Set = ["com.apple.dt.Xcode",
                                 "com.sublimetext.3"]
    
    static let whitelist = Integrations.terminals
//                    .union(Integrations.editors)
//                    .union(Integrations.browsers)
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
        components.scheme = "https"
        components.host = "app.withfig.com"
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
        components.scheme = "https"
        components.host = "app.withfig.com"
        components.path = cmd
        components.queryItems = [URLQueryItem(name: "input", value: argv)]
        return components.url!//URL(string:"\(components.string!)?input=\(argv)")!
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
    static func symlinkCLI(){
//        cmd="tell application \"Terminal\" to do script \"uptime\""
//          osascript -e "$cmd"
        
        if let path = Bundle.main.path(forResource: "fig", ofType: "", inDirectory: "dist") {
            print(path)
            let script = "mkdir -p /usr/local/bin && ln -sf '\(path)' '/usr/local/bin/fig'"
            
            let out = "cmd=\"do shell script \\\"\(script)\\\" with administrator privileges\" && osascript -e \"$cmd\"".runAsCommand()
            
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
