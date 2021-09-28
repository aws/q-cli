//
//  PTY.swift
//  fig
//
//  Created by Matt Schrage on 9/27/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class PTY {
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
    
    func start(executablePath: String, environment: [String: String]) {
        PseudoTerminal.log("Starting PTY...")
        let shell = executablePath
        

        
        pty.process.startProcess(executable: shell, args: [], environment: rawEnv.count == 0 ? nil : rawEnv)
        pty.process.delegate = self
        
        
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
                                              "LANG" : "\(LANG).UTF-8"]) { $1 }
        
        return updatedEnv.reduce([]) { (acc, elm) -> [String] in
            let (key, value) = elm
            return acc + ["\(key)=\(value)"]
        }
    }
    
}


extension PTY {
    @objc func recievedEnvironmentVariablesFromShell(_ notification: Notification) {
        
    }
    
    @objc func recievedCallbackNotification(_ notification: Notification) {
        
    }
}

extension PTY : LocalProcessDelegate {
    func processTerminated(_ source: LocalProcess, exitCode: Int32?) {
        
    }
    
    func dataReceived(slice: ArraySlice<UInt8>) {
        
    }
    
    func getWindowSize() -> winsize {
        
    }
    
    
}
