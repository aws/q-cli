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
    
    func start(with env: [String: String])
    func write(command: String, control: ControlCode?)
    func execute(command: String, handlerId:String, asBackgroundJob: Bool, asPipeline: Bool)
    func stream(command: String, handlerId:String)
    func close()
    
    var delegate: PseudoTerminalEventDelegate? { get set }
}

class PseudoTerminalHelper {
    var executeHandlers: [ String: (String) -> Void ] = [:]
    let queue = DispatchQueue(label: "com.withfig.ptyhelper", attributes: .concurrent)

    let pty = PseudoTerminal()
    let debug = false
    fileprivate let semaphore = DispatchSemaphore(value: 1)

    // Because they ruin your punchline.
    // Why should you never tell multithreaded programming jokes?
    func execute(_ command: String, handler: @escaping (String) -> Void) {
        // timeout prevents deadlocks
        let _ = semaphore.wait(timeout: .now())
        // Move all of this behind the semaphore!
        let id = UUID().uuidString
        queue.async(flags: .barrier) {
          self.executeHandlers[id] = handler
        }
        print("pty: Executing command with PTY Service '\(command)'. Output Id = \(id).")
        pty.execute(command: command, handlerId: id, asBackgroundJob: true, asPipeline: false)
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
            var handlerOption: ((String) -> Void)?
            queue.sync {
              handlerOption = self.executeHandlers[msg.handleId]
            }
          
            guard let handler = handlerOption else {
                return
            }
          
            print("pty: Finished executing command for id = \(msg.handleId)")
            semaphore.signal()
            handler(msg.output)
          
            queue.async(flags: .barrier) {
              self.executeHandlers.removeValue(forKey: msg.handleId)
            }
        }
    }
    
}

class PseudoTerminal : PseudoTerminalService {
    static let recievedEnvironmentVariablesFromShellNotification = NSNotification.Name("recievedEnvironmentVariablesFromShellNotification")
    static let recievedCallbackNotification = NSNotification.Name("recievedCallbackNotification")
    static func log(_ message: String) {
      Logger.log(message: message, subsystem: .pty)
    }
  
    init() {
//        self.delegate = eventDelegate
      NotificationCenter.default.addObserver(self,
                                             selector: #selector(recievedEnvironmentVariablesFromShell(_:)),
                                             name: PseudoTerminal.recievedEnvironmentVariablesFromShellNotification,
                                             object: nil)
      NotificationCenter.default.addObserver(self,
                                             selector: #selector(recievedCallbackNotification(_:)),
                                             name: PseudoTerminal.recievedCallbackNotification,
                                             object: nil)
    }
  
    deinit {
      NotificationCenter.default.removeObserver(self)
    }
  
    @objc func recievedEnvironmentVariablesFromShell(_ notification: Notification) {
      
      guard let env = notification.object as? [String: Any], self.mirrorsEnvironment else { return }
      // Update environment variables in autocomplete PTY
      let patterns = Settings.shared.getValue(forKey: Settings.ptyEnvKey) as? [String]
      let environmentVariablesToMirror: Set<String> = Set(patterns ?? [ "AWS_" ]).union(["PATH"])
      let variablesToUpdate = env.filter({ (element) -> Bool in
        guard element.value as? String != nil else {
          return false
        }
        
        return environmentVariablesToMirror.reduce(false) { (result, prefix) -> Bool in
          return result || element.key.starts(with: prefix)
        }
      })
      
      let command = variablesToUpdate.keys.map { "export \($0)='\(variablesToUpdate[$0] ?? "")'" }.joined(separator: "\n")
      
      let tmpFile = NSTemporaryDirectory().appending("fig_source_env")
      Logger.log(message: "Writing new ENV vars to '\(tmpFile)'", subsystem: .pty)

      do {
        
        try command.write(toFile: tmpFile,
                      atomically: true,
                      encoding: .utf8)
        sourceFile(at: tmpFile)
      } catch {
        Logger.log(message: "could not source ENV vars from '\(tmpFile)'", subsystem: .pty)
      }
    }
  
    @objc func recievedCallbackNotification(_ notification: Notification) {
      PseudoTerminal.log("recieved callback")
      guard let info = notification.object as? [String: String],
            let handlerId = info["handlerId"],
            let pathToFile = info["filepath"] else {
        return
      }
      guard executeHandlers.contains(handlerId) else {
        return
      }

      PseudoTerminal.log("callback for \(handlerId) with output at \(pathToFile)")
      executeHandlers.remove(handlerId)

      if let delegate = self.delegate,
         let data = FileManager.default.contents(atPath: pathToFile) {
          let output = String(decoding: data, as: UTF8.self)
          let msg = PtyMessage(type: "execute", handleId: handlerId, output: output)
          delegate.recievedDataFromPty(Notification(name: .recievedDataFromPty, object: msg))
      }
      

    }

  
    let pty: HeadlessTerminal = HeadlessTerminal(onEnd: { (code) in
      PseudoTerminal.log("ending session with exit code: \(code ?? -1)")
    })
  
    var rawOutput = ""
    var streamHandlers: Set<String> = []
    var executeHandlers: Set<String> = []
    var delegate: PseudoTerminalEventDelegate?
    var mirrorsEnvironment = false
    func start(with env: [String : String]) {
        PseudoTerminal.log("Starting PTY...")
        let shell = env["SHELL"] ?? "/bin/sh"
        
        // don't add shell hooks to pty
        // Add TERM variable to supress warning for ZSH
        // Set INPUTRC variable to prevent using a misconfigured inputrc file (https://linear.app/fig/issue/ENG-500)
        // Set FIG_PTY so that dotfiles can detect when they are being run in fig.pty
        let lang = NSLocale.current.languageCode ?? "en"
        let region = NSLocale.current.regionCode ?? "US"
        let LANG = lang + "_" + region
        let updatedEnv = env.merging(["FIG_ENV_VAR" : "1",
                                      "FIG_SHELL_VAR" : "1",
                                      "TERM" : "xterm-256color",
                                      "INPUTRC" : "~/.fig/nop",
                                      "FIG_PTY" : "1",
                                      "HISTCONTROL" : "ignoreboth",
                                      "LANG" : "\(LANG).UTF-8"]) { $1 }
        let rawEnv = updatedEnv.reduce([]) { (acc, elm) -> [String] in
            let (key, value) = elm
            return acc + ["\(key)=\(value)"]
        }
        
        pty.process.startProcess(executable: shell, args: [], environment: rawEnv.count == 0 ? nil : rawEnv)
        pty.process.delegate = self

        pty.send(" set +o history\r")
        pty.send(" unset HISTFILE\r")
      
        // Retrieve PATH from settings if it exists
        if let path = Settings.shared.getValue(forKey: Settings.ptyPathKey) as? String {
          pty.send("export PATH='\(path)'\r")
        } else { // export PATH from userShell
          pty.send("export PATH=$(\(Defaults.userShell) -li -c \"/usr/bin/env | /usr/bin/grep '^PATH=' | /bin/cat | /usr/bin/sed 's|PATH=||g'\")\r")
        }
      
        // Source default ptyrc file (if it exists)
        sourceFile(at: "~/.fig/tools/ptyrc")
      
      // Source user-specified ptyrc file (if it exists)
        let filePath = Settings.shared.getValue(forKey: Settings.ptyInitFile) as? String ?? "~/.fig/user/ptyrc"
        sourceFile(at: filePath)
    }
    
    func sourceFile(at path: String) {
      let expandedFilePath = NSString(string: path).expandingTildeInPath

      if FileManager.default.fileExists(atPath: expandedFilePath) {
        PseudoTerminal.log("sourcing \(expandedFilePath)")
        pty.send("source \(expandedFilePath)\r")
      }
    }
  
    func write(command: String, control: ControlCode?) {
        if let code = control {
            PseudoTerminal.log("Write PTY controlCode: \(code.rawValue)")
            switch code {
            case .EOT:
                pty.send(data: [0x4])
            case .ETX:
                pty.send(data: [0x3])
            }
        } else {
            PseudoTerminal.log("Write PTY command: \(command)")
            pty.send("\(command)\r")
        }
    }
  
    func shell(command: String, handlerId: String) {
      pty.send("\(command) | fig_callback \(handlerId) &\r")
      PseudoTerminal.log("Execute PTY command: \(command) \(pty.process.running) \(pty.process.delegate)")
    }
  
    static let callbackExecutable = "\(NSHomeDirectory())/.fig/bin/fig_callback"
    func execute(command: String,
                 handlerId: String,
                 asBackgroundJob: Bool,
                 asPipeline: Bool) {
      
      executeHandlers.insert(handlerId)

      var commandToRun: String
      
      if asPipeline {
        commandToRun = "\(command) | \(PseudoTerminal.callbackExecutable) \(handlerId)"
      } else {
        let tmpFilepath = "/tmp/\(handlerId)"
        commandToRun = "( \(command) )> \(tmpFilepath) && \(PseudoTerminal.callbackExecutable) \(handlerId) \(tmpFilepath)"

      }
      
      if asBackgroundJob {
        commandToRun.append(" &")
      }
      
      commandToRun.append("\r")
      
      pty.send(commandToRun)

      PseudoTerminal.log("Execute PTY command: \(command) \(pty.process.running) \(pty.process.delegate)")
        PseudoTerminal.log(commandToRun)
    }
    
    let streamDelimeter = "================="
    func stream(command: String, handlerId: String) {
        // not sure why this is commented out?
        //        streamHandlers.insert(handlerId)
        let cmd = "printf \"<<<\" ; echo \"\(streamDelimeter)\(handlerId)\(streamDelimeter)\" ; \(command) ; echo \"\(streamDelimeter)\(handlerId)\(streamDelimeter)>>>\"\r"
        pty.send(cmd)
        PseudoTerminal.log("Stream PTY command: \(command)")
    }
    
    func close() {
        PseudoTerminal.log("Close PTY")
        streamHandlers = []
        executeHandlers = []
        if pty.process.running {
            pty.send(data: [0x4])
            kill(pty.process.shellPid, SIGTERM)
        }
    }
}

extension PseudoTerminal : LocalProcessDelegate {
    func processTerminated(_ source: LocalProcess, exitCode: Int32?) {
        PseudoTerminal.log("Exited...\(exitCode ?? 0)")
    }
    
    func dataReceived(slice: ArraySlice<UInt8>) {
        let data = String(bytes: slice, encoding: .utf8) ?? ""
        PseudoTerminal.log(data)
        
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
            
            if let delegate = self.delegate {
                let msg = PtyMessage(type: "stream", handleId: handle, output: ping)
                delegate.recievedDataFromPty(Notification(name: .recievedDataFromPty, object: msg))
            }

        }
        
        if let streamCandidate = data.groups(for:"<<<\(streamDelimeter)(.*?)\(streamDelimeter)")[safe: 0] {
            streamHandlers.insert(streamCandidate[1])
        }
        
        rawOutput += data

    }
    
    func getWindowSize() -> winsize {
        return winsize(ws_row: UInt16(60), ws_col: UInt16(50), ws_xpixel: UInt16 (16), ws_ypixel: UInt16 (16))
    }
    
}
