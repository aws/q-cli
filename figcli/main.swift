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
        env["VIA_FIG_COMMAND"] = "1"
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


func exec(command: String, args: [String]) {
  withCStrings([command] + args) { (args) in
    guard execvp(command, args) != 0 else {
      print("Failed to exec...")
      return

    }
    fatalError("Impossible if execv succeeded")

  }

}

let arguments = CommandLine.arguments

if arguments.count > 1 {
    let command = arguments[1]
    if command == "cli:installed" {
        print("true")
        exit(0)
    } else if command == "app:running" {
      let appIsRunning = NSWorkspace.shared.runningApplications.filter { $0.bundleIdentifier == "com.mschrage.fig"}.count >= 1
      print(appIsRunning ? "1" : "0")
      exit(0)
    } else if command == "launch" {
        let appIsRunning = NSWorkspace.shared.runningApplications.filter { $0.bundleIdentifier == "com.mschrage.fig"}.count >= 1
        
        if appIsRunning {
            print("\n› It seems like the Fig app is already running.\n")
        } else {
            print("\n› Launching Fig...\n")
            NSWorkspace.shared.launchApplication(withBundleIdentifier: "com.mschrage.fig", options: .default, additionalEventParamDescriptor: nil, launchIdentifier: nil)
        }
        
        exit(0)
      
    } else if command == "settings",
              arguments.count > 2 { 
      // we handle `fig settings` in main app


      guard var settings = Settings.loadFromFile() else {
        exit(0)
      }
      
      let key = arguments[2]
      if arguments.count == 3 { // fig settings key --> Read value
        guard let value = settings[key] else {
          print("No value associated with '\(key)'.")
          exit(0)
        }
        // todo: better formatting for dicts, recursive objects
        switch value {
        case is NSArray:
          (value as! NSArray).forEach { (item) in
            print(item)
          }
        case is Bool:
          print( (value as! Bool) ? "true" : "false" )
        default:
            print(value)
        }
      } else { // fig settings key value --> Write value to key
          let value = arguments[3]
          
          guard var data = value.data(using: .utf8) else { exit(1) }
          var json = try? JSONSerialization.jsonObject(with: data, options: .allowFragments)
          let isSerializable = json != nil
          
          if !isSerializable {
              data = "\"\(value)\"".data(using: .utf8)!
              json = try? JSONSerialization.jsonObject(with: data, options: .allowFragments)
          }
          
          if json != nil {
              settings[key] = json
              Settings.serialize(settings: settings)
          } else {
              print("Could not write '\(arguments[3])' to key '\(key)'")
          }
        
      }
      
      exit(0)

    }
}

// determine if command exists as script in ~/.fig/tools/cli/SCRIPT.sh
if let pathToScriptCommand = ScriptCommand.matchesArguments(arguments) {
    exec(command: "/bin/bash",
         args: [ pathToScriptCommand ] + Array(arguments.dropFirst(2)))
    exit(0)
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
    var execOnExit = false
    var commandToExec: String?
    var argsToExec: [String] = []
    
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
        var port = UserDefaults(suiteName: "com.mschrage.fig.shared")?.integer(forKey: "port") ?? 8765
        port = port == 0 ? 8765 : port
        connection = WebSocketStarscreamConnection(url: URL(string: "ws://localhost:\(port)")!)
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
        } else if (text.starts(with: "execvp:")) {
          let payload = text.replacingOccurrences(of: "execvp:", with: "")
          let tokens = payload
            .split(separator: " ").map { String($0) }
          
          guard let command = tokens.first else {
            return
          }
          
          let args = Array(tokens.dropFirst(1))
          
          self.execOnExit = true
          self.commandToExec = command
          self.argsToExec = args
          connection.disconnect()
          return
        }
      
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

          Please email \u{001b}[1mhello@fig.io\u{001b}[0m if this problem persists.

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
  
→ If not, run \u{001b}[1mfig launch\u{001b}[0m to relaunch the app.

  Please email \u{001b}[1mhello@fig.io\u{001b}[0m if this problem persists.

"""
    print(error)
    group.leave()
}

group.wait()

if let handler = handler, handler.execOnExit, let command = handler.commandToExec {
  print("Running: \(command) \(handler.argsToExec.joined(separator: " "))")
  exec(command: command,
       args: handler.argsToExec)

}
