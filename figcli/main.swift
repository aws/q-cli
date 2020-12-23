//
//  main.swift
//  figcli
//
//  Created by Matt Schrage on 5/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa
import Starscream

func withCStrings(_ strings: [String], scoped: ([UnsafeMutablePointer<CChar>?]) throws -> Void) rethrows {
    let cStrings = strings.map { strdup($0) }
    try scoped(cStrings + [nil])
    cStrings.forEach { free($0) }
}

enum RunCommandError: Error {
    case WaitPIDError
    case POSIXSpawnError(Int32)
}

func runCommand(_ command: String, completion: ((Int32) -> Void)? = nil) throws {
    var pid: pid_t = 0
    let args = ["sh", "-c", command]
    var env = ProcessInfo().environment
        env["SHELLPID"] = String(getppid())
    let envStr = env.map { k, v in "\(k)=\(v)" }
    try withCStrings(args) { cArgs in
        try withCStrings(envStr) { cEnvs in
            var status = posix_spawn(&pid, "/bin/sh", nil, nil, cArgs, cEnvs)
            if status == 0 {
                if (waitpid(pid, &status, 0) != -1) {
                    completion?(status)
                } else {
                    throw RunCommandError.WaitPIDError
                }
            } else {
                throw RunCommandError.POSIXSpawnError(status)
            }
        }
    }
}

let arguments = CommandLine.arguments

if arguments.count > 1 {
    let command = arguments[1]
    if command == "cli:installed" {
        print("true")
        exit(0)
    }
}

// early exit if bg:* and fig is not active

if (arguments.filter { $0.starts(with: "bg:")}.count == 1  && NSWorkspace.shared.runningApplications.filter { $0.bundleIdentifier == "com.mschrage.fig"}.count == 0) {
    exit(1)
}


// get stdin
var stdin = ""
//var line: String? = nil
//repeat {
//    line = readLine(strippingNewline: false)
//    if let line = line {
//        stdin += line
//    }
//} while (line != nil)

let env = ProcessInfo.processInfo.environment
//print(env)
let envJson = try? JSONSerialization.data(withJSONObject: env, options: .prettyPrinted)

//print(stdin)
//print(arguments)

//print("Hello, World! This is my command line tool")

//}

fileprivate func delayWithSeconds(_ seconds: Double, completion: @escaping () -> ()) {
    DispatchQueue.main.asyncAfter(deadline: .now() + seconds) {
        completion()
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
    
    func jsonRepresentation() -> String? {
        guard let jsonData = try? JSONEncoder().encode(self) else { return nil }
        return String(data: jsonData, encoding: .utf8)
    }

}

class CLI : WebSocketConnectionDelegate {
    var connection: WebSocketConnection
    var busy: Bool = false {
        didSet {
            if (!busy && pendingDisconnection) {
                connection.disconnect()
//                group.leave()
            }
        }
    }
    var pendingDisconnection = false;
    let group: DispatchGroup
    
    var command: ShellMessage {
        get {
            var envString: String?
            if let envJSON = try? JSONSerialization.data(withJSONObject: env, options: .fragmentsAllowed) {
                envString = String(decoding: envJSON, as: UTF8.self)
            }
           return ShellMessage(type: "pipe",
                               source: env["TERM_PROGRAM"] ?? "",
                               session: env["TERM_SESSION_ID"] ?? "",
                               env: envString,
                               io: nil,
                               data: stdin,
                               options:  Array(CommandLine.arguments.dropFirst()))
        }
    }

    init(env: [String : String], stdin: String, arguments: [String], group: DispatchGroup) {
        self.group = group
        connection = WebSocketStarscreamConnection(url: URL(string: "ws://localhost:8765")!)
        connection.delegate = self
        connection.connect()
    }
    
    func onConnected(connection: WebSocketConnection) {
        
        if let msg = self.command.jsonRepresentation(){
            
            // The hello message allows bidirectional communication based on sessionId
            // it is unecessary for bg: commands (and can interfere with normal fig commands)
            let isBG = self.command.options?.first?.contains("bg:") ?? false
            if (!isBG) {
                var hello = self.command
                hello.type = "hello"
                connection.send(text: hello.jsonRepresentation() ?? "")
            }
            
            connection.send(text: msg)
            
        }
    }
    
    func onDisconnected(connection: WebSocketConnection, error: Error?) {
//        print("bye")
        group.leave()

    }
    
    func onError(connection: WebSocketConnection, error: Error) {
        print("error! \(error.localizedDescription)")
        group.leave()

    }
    
    func onMessage(connection: WebSocketConnection, text: String) {
//        print(text)
        // disconnect on acknowledgment...
//        connection.disconnect()
        if (text == "disconnect") {
//            guard !protected else {
//                return
//            }
            if (!busy) {
                connection.disconnect()
//                group.leave()
            } else {
                pendingDisconnection = true
            }
            return
        }

//        print("msg: '\(text)'")
        busy = true
        
        try? runCommand(text) { (status) in
            self.busy = false
        }
    }
    
    func onMessage(connection: WebSocketConnection, data: Data) {
        
    }
    
    
}


let group = DispatchGroup()

var handler: CLI?
//var socket: WebSocket!
group.enter()
//async operation 1

DispatchQueue.global(qos: .default).async {
    handler = CLI(env: env, stdin: stdin, arguments: arguments, group: group)
    
    // Network calls or some other async task

}

/// Timeout
DispatchQueue.global().asyncAfter(deadline: .now() + 1.25) {
    if (handler?.busy ?? false) { return }
    guard (arguments.filter { $0.starts(with: "bg:")}.count == 0) else {
        group.leave()
        return
    }
    
    guard let loggedIn = UserDefaults(suiteName: "com.mschrage.fig.shared")?.bool(forKey: "loggedIn"), loggedIn else {
        
        let error =
        """

        › \u{001b}[31mNot logged in to Fig\u{001b}[0m

          \u{001b}[1mQUICK FIX\u{001b}[0m
          Open Fig to set up your account.

          Please email \u{001b}[1mhello@withfig.com\u{001b}[0m if this problem persists.

        """
        print(error)
        group.leave()
        return
    }
    
    let error =
"""

› \u{001b}[31mCould not connect to fig.app.\u{001b}[0m

  \u{001b}[1mQUICK FIX\u{001b}[0m
  Check if Fig is active. (You should see the ◧ Fig icon in your menu bar).
  
→ If not, run \u{001b}[1mopen -b com.mschrage.fig\u{001b}[0m to relaunch the app.

  Please email \u{001b}[1mhello@withfig.com\u{001b}[0m if this problem persists.

"""
    print(error)
    group.leave()
}

group.wait()
