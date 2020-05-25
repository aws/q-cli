//
//  ShellBridge.swift
//  fig
//
//  Created by Matt Schrage on 4/18/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa

protocol ShellBridgeEventListener {
    func recievedDataFromPipe(_ notification: Notification)

}

extension Notification.Name {
    static let recievedDataFromPipe = Notification.Name("recievedDataFromPipe")
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

        ShellBridge.delayWithSeconds(0.25) {
            self.socket = WebSocketTaskConnection.init(url: URL(string: "ws://localhost:8765")!)
            self.socket?.delegate = self;
            self.socket?.connect()
        }
    }
    
    @objc func setPreviousApplication(notification: NSNotification!) {
        self.previousFrontmostApplication = notification!.userInfo![NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
        print(self.previousFrontmostApplication?.bundleIdentifier ?? "")
    }

    static func injectStringIntoTerminal(_ cmd: String, runImmediately: Bool = false, completion: (() -> Void)? = nil) {
            ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
            ShellBridge.delayWithSeconds(0.3) {
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
                        delayWithSeconds(0.10) {
                            self.simulate(keypress: .rightArrow)
                            self.simulate(keypress: .enter)
                            if let completion = completion {
                                completion()
                            }
                       }
    
                   // need delay so that terminal responds
                   delayWithSeconds(1) {
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

        if let path = Bundle.main.path(forResource: "websocket", ofType: "", inDirectory: "wsdist") {
            print(path)
            
            self.socketServer = path.runInBackground {
                  print("Closing the socket server")
            }

        } else {
            print("couldn't start socket server")
        }
    }
    
    func stopWebSocketServer() {
        if let server = self.socketServer {
            server.terminate()
            print("socket server process terminated")
        } else {
            print("socket server is not running")
        }
    }
}

extension ShellBridge: WebSocketConnectionDelegate {
    func onConnected(connection: WebSocketConnection) {
        print("connected")
//        connection.send(text: "hello")

        ShellBridge.delayWithSeconds(1) {
            connection.send(text: "register_as_host")

        }
    }
    
    func onDisconnected(connection: WebSocketConnection, error: Error?) {
        print("disconnected")

    }
    
    func onError(connection: WebSocketConnection, error: Error) {
        print(error)
    }
    
    func onMessage(connection: WebSocketConnection, text: String) {
//        print("msg:\(text)")
        let decoder = JSONDecoder()
        do {
            let msg = try decoder.decode(ShellMessage.self, from: text.data(using: .utf8)!)
//            print(msg)
            
            if msg.type == "pipe" {
                print(msg.data)
                NotificationCenter.default.post(name: .recievedDataFromPipe, object: msg)
            }
            
            

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
                    .union(Integrations.editors)
//                    .union(Integrations.browsers)
}

extension ShellBridge {
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
        components.host = "withfig.com"
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
            cmd = options.first!
            raw = Array(options.suffix(from: 1))
        }
        
        let argv = raw.joined(separator: " ").addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? ""
        var components = URLComponents()
        components.scheme = "https"
        components.host = "withfig.com"
        components.path = cmd
        components.queryItems = [URLQueryItem(name: "argv", value: argv)]
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
