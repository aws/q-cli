//
//  PseudoTerminalService.swift
//  fig
//
//  Created by Matt Schrage on 7/12/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation



enum ControlCode : String {
    typealias RawValue = String
    case EOT = "^D"
    case ETX = "^C"
    
}

protocol PseudoTerminalEventDelegate {
    func recievedDataFromPty(_ notification: Notification)
}

protocol PseudoTerminalService {
    
//    var pty: HeadlessTerminal
//    var rawOutput: String { get set }
//    var streamHandlers: Set<String> { get set }
//    var executeHandlers: Set<String> { get set }
    
    func start(with env: [String: String])
    func write(command: String, control: ControlCode?)
    func execute(command: String, handlerId:String)
    func stream(command: String, handlerId:String)
    func close()
    
    var delegate: PseudoTerminalEventDelegate? { get set }
}

class PseudoTerminalHelper {
    var executeHandlers: [ String: (String) -> Void ] = [:]
    let pty = PseudoTerminal()
    let debug = false
    fileprivate let semaphore = DispatchSemaphore(value: 1)

    // Because they ruin your punchline.
    // Why should you never tell multithreaded programming jokes?
    func execute(_ command: String, handler: @escaping (String) -> Void) {
        // Move all of this behind the semaphore!
        let id = UUID().uuidString
        executeHandlers[id] = handler
        print("pty: Executing command with PTY Service '\(command)'. Output Id = \(id).")
        // timeout prevents deadlocks
        let _ = semaphore.wait(timeout: .now())
        pty.execute(command: command, handlerId: id)
    }
    
    func start(with env: [String : String]) {
        pty.start(with: env)
        pty.delegate = self
      if debug {
        let path = "\(NSHomeDirectory())/.fig/\(UUID().uuidString).session"
        pty.write(command:"script -q -t 0 \(path)", control: nil)
        print("pty: Debug mode is on. Writting all logs to \(path). Can be read with tail -f \(path).")
      }
    }
    
    func close() {
        pty.close()
    }
}

extension PseudoTerminalHelper : PseudoTerminalEventDelegate {
    func recievedDataFromPty(_ notification: Notification) {
        if let msg = notification.object as? PtyMessage {
            print("pty: is anything happening here?")
            guard let handler = self.executeHandlers[msg.handleId] else {
                return
            }
          
            print("pty: Finished executing command for id = \(msg.handleId)")
            semaphore.signal()
            handler(msg.output)
        }
    }
    
}

class PseudoTerminal : PseudoTerminalService {
    
//    init(eventDelegate: PseudoTerminalEventDelegate) {
//        self.delegate = eventDelegate
//    }
    let pty: HeadlessTerminal = HeadlessTerminal(onEnd: { (code) in
        print("Exit")
    })
    var rawOutput = ""
    var streamHandlers: Set<String> = []
    var executeHandlers: Set<String> = []
    var delegate: PseudoTerminalEventDelegate?
    
    func start(with env: [String : String]) {
        print("Starting PTY...")
        let shell = env["SHELL"] ?? "/bin/sh"
        
        // don't add shell hooks to pty
        // Add TERM variable to supress warning for ZSH
        // Set INPUTRC variable to prevent using a misconfigured inputrc file (https://linear.app/fig/issue/ENG-500)
        // Set FIG_PTY so that dotfiles can detect when they are being run in fig.pty
        let updatedEnv = env.merging(["FIG_ENV_VAR" : "1", "FIG_SHELL_VAR" : "1", "TERM" : "xterm-256color", "INPUTRC" : "~/.fig/nop", "FIG_PTY" : "1"]) { $1 }
        let rawEnv = updatedEnv.reduce([]) { (acc, elm) -> [String] in
            let (key, value) = elm
            return acc + ["\(key)=\(value)"]
        }
        
        pty.process.startProcess(executable: shell, args: [], environment: rawEnv.count == 0 ? nil : rawEnv)
        pty.process.delegate = self

        pty.send("unset HISTFILE\r")
        
      
        // export path from userShell
        pty.send("export PATH=$(\(Defaults.userShell) -li -c \"/usr/bin/env | /usr/bin/grep '^PATH=' | /bin/cat | /usr/bin/sed 's|PATH=||g'\")\r")

        // Copy enviroment from userShell
//        pty.send("export $(env -i '\(Defaults.userShell)' -li -c env | tr '\n' ' ')\r")
        print(pty.process.delegate)
    }
    
    func write(command: String, control: ControlCode?) {
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

    let executeDelimeter = "-----------------"
    func execute(command: String, handlerId: String) {
        executeHandlers.insert(handlerId)
        let cmd = "printf \"<<<\" ; echo \"\(executeDelimeter)\(handlerId)\(executeDelimeter)\" ; \(command) ; echo \"\(executeDelimeter)\(handlerId)\(executeDelimeter)>>>\"\r"
        pty.send(cmd)
        print("Execute PTY command: \(cmd) \(pty.process.running) \(pty.process.delegate)")

    }
    
    let streamDelimeter = "================="
    func stream(command: String, handlerId: String) {
        // not sure why this is commented out?
        //        streamHandlers.insert(handlerId)
        let cmd = "printf \"<<<\" ; echo \"\(streamDelimeter)\(handlerId)\(streamDelimeter)\" ; \(command) ; echo \"\(streamDelimeter)\(handlerId)\(streamDelimeter)>>>\"\r"
        pty.send(cmd)
        print("Stream PTY command: \(command)")
    }
    
    func close() {
        print("Close PTY")
        streamHandlers = []
        executeHandlers = []
        if pty.process.running {
            pty.send(data: [0x4])
            pty.send(data: [0x4])
            pty.send(data: [0x4])
            pty.send(data: [0x4])
            pty.send(data: [0x4])
            kill(pty.process.shellPid, SIGTERM)
        }
    }
}

extension PseudoTerminal : LocalProcessDelegate {
    func processTerminated(_ source: LocalProcess, exitCode: Int32?) {
        print("Exited...\(exitCode ?? 0)")
    }
    
    func dataReceived(slice: ArraySlice<UInt8>) {
        let data = String(bytes: slice, encoding: .utf8) ?? ""
        print("data", data)
        
        
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
            if let delegate = self.delegate {
                let msg = PtyMessage(type: "stream", handleId: handle, output: ping)
                delegate.recievedDataFromPty(Notification(name: .recievedDataFromPty, object: msg))
//                let msg = PtyMessage(type: "stream", handleId: handle, output: ping)
//                NotificationCenter.default.post(name: .recievedDataFromPty, object: msg)
            }

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
                
                if let delegate = self.delegate {
                    let msg = PtyMessage(type: "execute", handleId: handle, output: output)
                    delegate.recievedDataFromPty(Notification(name: .recievedDataFromPty, object: msg))
//                    let msg = PtyMessage(type: "execute", handleId: handle, output: output)
//                    NotificationCenter.default.post(name: .recievedDataFromPty, object: msg)
                }


            }

        }
    }
    
    func getWindowSize() -> winsize {
        return winsize(ws_row: UInt16(60), ws_col: UInt16(50), ws_xpixel: UInt16 (16), ws_ypixel: UInt16 (16))
    }
    
}
