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

class PseudoTerminal {
    typealias ProcessFinished = (stdout: String, stderr: String, exitCode: Int32)
    typealias CallbackHandler =  (ProcessFinished) -> Void
    typealias HandlerId = String
    
    static let shared: PseudoTerminal = {
        let pty = PseudoTerminal()
        pty.start(with: [:])
        return pty
    }()
  
    let rateLimiter = RateLimiter(minimumDelay: 0.3, queue: DispatchQueue(label: "io.fig.pseudoterminal"))

    
    fileprivate static let CRLF = "\r\n"
    

    static let recievedEnvironmentVariablesFromShellNotification = NSNotification.Name("recievedEnvironmentVariablesFromShellNotification")
    static let recievedCallbackNotification = NSNotification.Name("recievedCallbackNotification")
    
    // https://scriptingosx.com/2017/05/where-paths-come-from/
    static let defaultMacOSPath = "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:"
    
    static func log(_ message: String) {
      Logger.log(message: message, subsystem: .pty)
    }
    
  

    
    // MARK: Initialize

    fileprivate var handlers: [HandlerId: CallbackHandler] = [:]
    fileprivate let headless: HeadlessTerminal = HeadlessTerminal(onEnd: { (code) in
        PseudoTerminal.log("ending session with exit code: \(code ?? -1)")
    })
    
    init() {
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
    
    func start(with environment: [String: String]) {
        PseudoTerminal.log("Starting PTY...")
        let shell = "/bin/bash" //"/bin/bash"
        
        let rawEnv = mergeFigSpecificEnvironmentVariables(with: environment)

        
        self.headless.process.startProcess(executable: shell, args: [ "--norc", "--noprofile", "--noediting"], environment: rawEnv.count == 0 ? nil : rawEnv)
        self.headless.process.delegate = self
        
        if let shouldWriteTranscript = Settings.shared.getValue(forKey: Settings.ptyTranscript) as? Bool,
               shouldWriteTranscript {
          self.write(" script -qt0 ~/.fig/logs/pty_transcript.log")
          self.headless.process.debugIO = true
        }
        
        self.write(" set +o history" + PseudoTerminal.CRLF)
        self.write(" unset HISTFILE" + PseudoTerminal.CRLF)
      
        // Retrieve PATH from settings if it exists
        if let path = Settings.shared.getValue(forKey: Settings.ptyPathKey) as? String, path.count > 0 {
            self.set(environmentVariable: "PATH", value: path)
        } else {
            self.set(environmentVariable: "PATH", value: PseudoTerminal.defaultMacOSPath)
        }
      
        // Source default ptyrc file (if it exists)
        sourceFile(at: "~/.fig/tools/ptyrc")
      
        // Source user-specified ptyrc file (if it exists)
        let filePath = Settings.shared.getValue(forKey: Settings.ptyInitFile) as? String ?? "~/.fig/user/ptyrc"
        sourceFile(at: filePath)
        
    }
    
    func write(_ input: String) {
        self.headless.send(input + PseudoTerminal.CRLF)
    }
    
    func close() {
        if self.headless.process.running {
            kill( self.headless.process.shellPid, SIGTERM)
            kill( self.headless.process.shellPid, SIGKILL)
        }
    }
  
  func restart(with environment: [String: String], completion: ((Bool) -> Void)? = nil) {
      self.close()
    
      // todo: use self.headless.onEnd instead of a hardcoded delay
      Timer.delayWithSeconds(1) {
        guard !self.headless.process.running else {
          completion?(false)
          return
        }
        
        self.start(with: environment)
        completion?(true)
      }
    }
    
    // MARK: Utilities

    fileprivate func mergeFigSpecificEnvironmentVariables(with environment: [String : String]) -> [String] {
        // don't add shell hooks to pty
        // Add TERM variable to supress warning for ZSH
        // Set INPUTRC variable to prevent using a misconfigured inputrc file (https://linear.app/fig/issue/ENG-500)
        // Set FIG_PTY so that dotfiles can detect when they are being run in fig.pty
        let lang = NSLocale.current.languageCode ?? "en"
        let region = NSLocale.current.regionCode ?? "US"
        let LANG = lang + "_" + region
        let updatedEnv = environment.merging(["FIG_ENV_VAR" : "1",
                                              "FIG_SHELL_VAR" : "1",
                                              "TERM" : "xterm-256color",
                                              "INPUTRC" : "~/.fig/nop",
                                              "FIG_PTY" : "1",
                                              "HISTCONTROL" : "ignoreboth",
                                              "HOME" : NSHomeDirectory(),
                                              "LANG" : "\(LANG).UTF-8"]) { $1 }
        
        return updatedEnv.reduce([]) { (acc, elm) -> [String] in
            let (key, value) = elm
            return acc + ["\(key)=\(value)"]
        }
    }
    func sourceFile(at path: String) {
        let expandedFilePath = NSString(string: path).expandingTildeInPath
        
        if FileManager.default.fileExists(atPath: expandedFilePath) {
            PseudoTerminal.log("sourcing \(expandedFilePath)")
            self.write("source \(expandedFilePath)")
        }
    }
    
    func set(environmentVariable key: String, value: String) {
        self.write("export \(key)='\(value)'")
    }
    
}


extension PseudoTerminal {
    @objc func recievedEnvironmentVariablesFromShell(_ notification: Notification) {
      
      guard let env = notification.object as? [String: Any] else { return }
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
}

// MARK: Running shell commands

extension PseudoTerminal {
    struct ExecutionOptions: OptionSet {
        let rawValue: Int

        static let backgroundJob = ExecutionOptions(rawValue: 1 << 0)
        static let pipelined = ExecutionOptions(rawValue: 1 << 1)
    }
    
    static let callbackExecutable = Bundle.main.path(forAuxiliaryExecutable: "fig_callback")!
    func execute(_ command: String,
                 handlerId: HandlerId = UUID().uuidString,
                 options: ExecutionOptions = [.backgroundJob],
                 handler: @escaping CallbackHandler) {
        
        var cappedHandlerId = handlerId
        // note: magic number comes from fig_callback implementation
        if handlerId.count > 5 {
            PseudoTerminal.log("handlerId must be 5 characters or less. '\(handlerId)' is too long and will be truncated.")
            let index = handlerId.index(handlerId.startIndex, offsetBy: 5)
            cappedHandlerId = String(handlerId.prefix(upTo: index))
        }
        
        var commandToRun: String
        
        // note: pipelined commands currently do not provide stderr or exit code!
        if options.contains(.pipelined) {
            commandToRun = "\(command) | \(PseudoTerminal.callbackExecutable) \(cappedHandlerId)"
        } else {
            let tmpFilepath = "/tmp/\(cappedHandlerId)"
            commandToRun = "( ( \(command) ) 1> \(tmpFilepath).stdout 2> \(tmpFilepath).stderr; \(PseudoTerminal.callbackExecutable) \(handlerId) \(tmpFilepath) $? )"
        }
      
        if options.contains(.backgroundJob) {
            commandToRun.append(" &")
        }
      
        commandToRun.append(PseudoTerminal.CRLF)
        self.handlers[cappedHandlerId] = handler
        rateLimiter.limit {
          PseudoTerminal.log("Writing command for handler '\(cappedHandlerId)'")
          self.headless.send(commandToRun)
        }

        PseudoTerminal.log("Running '\(command)' \(options.contains(.pipelined) ? "as pipeline" : "")\(options.contains(.backgroundJob) ? " in background" : "")")
    }
    
    @objc func recievedCallbackNotification(_ notification: Notification) {
        PseudoTerminal.log("recieved callback")
        guard let info = notification.object as? [String: String?],
              let handlerId = info["handlerId"] as? String,
              let pathToFile = info["filepath"] as? String else {
        
          return
        }

        guard let handler = self.handlers[handlerId] else {
            return
        }

        PseudoTerminal.log("callback for \(handlerId) with output at \(pathToFile)")
        
        self.handlers.removeValue(forKey: handlerId)

        if let legacy = contentsOfFile(path: pathToFile, suffix: "") {
            handler((legacy, "", -2))
            return
        }
        
        let stdout = contentsOfFile(path: pathToFile, suffix: ".stdout")
        let stderr = contentsOfFile(path: pathToFile, suffix: ".stderr")
        let exitCode: Int32 = {
            guard let codeStr = info["exitCode"] as? String,
                  let code = Int32(codeStr) else {
                return -1
            }
            
            return code
        }()

        handler((stdout ?? "", stderr ?? "", exitCode))
    }
    
    fileprivate func contentsOfFile(path: String, suffix: String) -> String? {
        guard let data = FileManager.default.contents(atPath: path + suffix) else { return nil }
        return String(decoding: data, as: UTF8.self)
    }
}

extension PseudoTerminal : LocalProcessDelegate {
    func processTerminated(_ source: LocalProcess, exitCode: Int32?) {
        
    }
    
    func dataReceived(slice: ArraySlice<UInt8>) {
        
    }
    
    func getWindowSize() -> winsize {
        return winsize(ws_row: UInt16(60), ws_col: UInt16(50), ws_xpixel: UInt16 (16), ws_ypixel: UInt16 (16))
    }
    
    
}


import FigAPIBindings
extension PseudoTerminal {
    func handleWriteRequest(_ request: Fig_PseudoterminalWriteRequest) throws -> Bool {
        switch request.input {
            case .text(let text):
                self.write(text)
            case .octal(let data):
                self.headless.send(data: ArraySlice([UInt8](data)))
//                self.headless.send(data: )
            case .none:
                throw APIError.generic(message: "No input specified")
            
        }
        
        return true
    }
    
    func handleExecuteRequest(_ request: Fig_PseudoterminalExecuteRequest, with id: Int64, callback: @escaping ((Fig_PseudoterminalExecuteResponse) -> Void)) {
        
        var options: ExecutionOptions = [ .backgroundJob ]
        
        if request.hasBackgroundJob && !request.backgroundJob {
            options.remove(.backgroundJob)
        }
        
        if request.hasIsPipelined && request.isPipelined {
            options.insert(.pipelined)
        }
        
        self.execute(request.command,
                     handlerId: String(id),
                     options: options) { (stdout, stderr, exitCode) in
            callback(Fig_PseudoterminalExecuteResponse.with({ response in
                response.stdout = stdout
                response.stderr = stderr
                response.exitCode = exitCode
            }))
        }
    }
}
