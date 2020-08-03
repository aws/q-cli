//
//  main.swift
//  figcli
//
//  Created by Matt Schrage on 5/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Starscream
let arguments = CommandLine.arguments
    
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
                group.leave()
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
            
            var hello = self.command
            hello.type = "hello"
            connection.send(text: hello.jsonRepresentation() ?? "")
            
            connection.send(text: msg)
            
        }
    }
    
    func onDisconnected(connection: WebSocketConnection, error: Error?) {
        print("bye")
        group.leave()

    }
    
    func onError(connection: WebSocketConnection, error: Error) {
        print("error! \(error.localizedDescription)")
        group.leave()

    }
    
    func onMessage(connection: WebSocketConnection, text: String) {
        // disconnect on acknowledgment...
//        connection.disconnect()
        if (text == "disconnect") {
            if (!busy) {
                group.leave()
            } else {
                pendingDisconnection = true
                return
            }
        }

//        print("msg: '\(text)'")
        busy = true
        let out = text.runAsCommand(false, cwd: ProcessInfo.processInfo.environment["PWD"], with: ProcessInfo.processInfo.environment)
        print(out)
        busy = false

//        print(out)
//        text.runInBackground(cwd: ProcessInfo.processInfo.environment["PWD"], with: ProcessInfo.processInfo.environment, updateHandler: { (out, proc) in
//            print(out)
//        }) {
//            print("done!")
//        }
        // stdout
        
        // run command without displaying
        
        // run / insert?
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

//delayWithSeconds(1) {
//    group.leave()
//}

group.wait()



