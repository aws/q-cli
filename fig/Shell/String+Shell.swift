//
//  String+Shell.swift
//  fig
//
//  Created by Matt Schrage on 4/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import SwiftUI

extension NSAppleScript {
  static func run(path: String) {
    let task = Process()
    task.launchPath = "/usr/bin/osascript"
    task.arguments = [path]

    task.launch()
  }
}

extension Process {

  static func run(command: String,
                  args: [String],
                  workingDirectory: String? = nil,
                  environment: [String: String]? = nil)
                // swiftlint:disable large_tuple
                  -> (output: [String], error: [String], exitCode: Int32) {

      var output: [String] = []
      var error: [String] = []

      let task = Process()
      task.launchPath = command
      task.arguments = args

      let outpipe = Pipe()
      task.standardOutput = outpipe
      let errpipe = Pipe()
      task.standardError = errpipe

      if let workingDirectory = workingDirectory {
          task.currentDirectoryURL = URL(fileURLWithPath: workingDirectory)
      }

      task.environment = (environment ?? [:]).merging([
        "PROCESS_LAUNCHED_BY_FIG": "1"
      ]) { $1 }

      do {
        try task.run()
      } catch {
        return ([""], ["Failed to launch process \(command) \(args)"], 1)
      }

      let outdata = outpipe.fileHandleForReading.readDataToEndOfFile()
      if var string = String(data: outdata, encoding: .utf8) {
          string = string.trimmingCharacters(in: .newlines)
          output = string.components(separatedBy: "\n")
      }

      let errdata = errpipe.fileHandleForReading.readDataToEndOfFile()
      if var string = String(data: errdata, encoding: .utf8) {
          string = string.trimmingCharacters(in: .newlines)
          error = string.components(separatedBy: "\n")
      }

      task.waitUntilExit()
      let status = task.terminationStatus

      return (output, error, status)
  }

  static func runAsync(command: String,
                       args: [String],
                       workingDirectory: String? = nil,
                       environment: [String: String]? = nil,
                       completion:
                       (((output: [String], error: [String], exitCode: Int32)) -> Void)? = nil) {

    DispatchQueue.global(qos: .background).async {
      let response = Process.run(command: command,
                                 args: args,
                                 workingDirectory: workingDirectory,
                                 environment: environment)

      if let completion = completion {
        DispatchQueue.main.async {
          completion(response)
        }
      }
    }
  }
}

extension String {
  func runWithElevatedPrivileges() -> String? {
    let myAppleScript = "do shell script \"\(self)\" with administrator privileges"
    var error: NSDictionary?
    guard let scriptObject = NSAppleScript(source: myAppleScript) else { return nil }

    let output: NSAppleEventDescriptor = scriptObject.executeAndReturnError(&error)
    if error != nil {
      print("error: \(error ?? [:])")
      return nil
    }

    return output.stringValue

  }

  func runWithElevatedPriviledgesFromAppleScript(completion: (() -> Void)? = nil) {
    "cmd=\"do shell script \\\"\(self)\\\" with administrator privileges\" && osascript -e \"$cmd\""
      .runInBackground(completion: { (_) in
      if let completion = completion {
        completion()
      }
    })

  }

  func runAsCommand(_ isVerbose: Bool = false, cwd: String? = nil, with env: [String: String]? = nil) -> String {

    let pipe = Pipe()
    let stderr = Pipe()
    let task = Process()
    // add "-li" to get closer to terminal behavior
    task.arguments = ["-c", String(format: "%@", self)]
    task.standardOutput = pipe
    task.standardError = stderr

    //        if let cwd = cwd {
    //            task.currentDirectoryPath = cwd
    //        }
    task.currentDirectoryPath = cwd ?? NSHomeDirectory()

    if let env = env {
      task.environment = env
    } else {
      task.environment = [:]
    }

    task.environment?["HOME"] = NSHomeDirectory()

    if let env = env, let shell = env["SHELL"] {
      task.launchPath = shell
    } else {
      task.launchPath = "/bin/sh"
    }

    //        let data = pipe.fileHandleForReading.readDataToEndOfFile()
    //        let output = String(data: data, encoding: String.Encoding.utf8) ?? ""

    let outputHandler = pipe.fileHandleForReading
    outputHandler.waitForDataInBackgroundAndNotify()

    var output = ""
    var dataObserver: NSObjectProtocol!
    let notificationCenter = NotificationCenter.default
    let dataNotificationName = NSNotification.Name.NSFileHandleDataAvailable
    dataObserver = notificationCenter.addObserver(
      forName: dataNotificationName,
      object: outputHandler,
      queue: nil
    ) { [weak dataObserver]  _ in
      guard let dataObserver = dataObserver else { return }
      let data = outputHandler.availableData
      guard data.count > 0 else {
        notificationCenter.removeObserver(dataObserver)
        outputHandler.closeFile()
        return
      }
      if let line = String(data: data, encoding: .utf8) {
        if isVerbose {
          print(line)
        }
        output += line + "\n"
      }
      outputHandler.waitForDataInBackgroundAndNotify()
    }

    let errorHandler = stderr.fileHandleForReading
    errorHandler.waitForDataInBackgroundAndNotify()
    var errorObserver: NSObjectProtocol!
    errorObserver = notificationCenter.addObserver(
      forName: dataNotificationName,
      object: errorHandler,
      queue: nil
    ) { [weak errorObserver] _ in
      guard let errorObserver = errorObserver else { return }
      let data = errorHandler.availableData
      guard data.count > 0 else {
        notificationCenter.removeObserver(errorObserver)
        errorHandler.closeFile()
        return
      }
      if let line = String(data: data, encoding: .utf8) {
        if isVerbose {
          print(line)
        }
        output += line + "\n"
      }
      errorHandler.waitForDataInBackgroundAndNotify()
    }
    //        task.terminationHandler = { (process) in
    //            notificationCenter.removeObserver(dataObserver!)
    //            notificationCenter.removeObserver(errorObserver!)
    //            outputHandler.closeFile()
    //            errorHandler.closeFile()
    //        }

    task.launch()
    task.waitUntilExit()
    //
    //        print("TerminationStatus:", task.terminationStatus)
    //        print("TerminationReason:", task.terminationReason)

    return output
    //        if let result = NSString(data: file.readDataToEndOfFile(), encoding: String.Encoding.utf8.rawValue) {
    //            return result as String
    //        }
    //        else {
    //            return "--- Error running command - Unable to initialize string from file data ---"
    //        }
  }

  fileprivate func addListener(_ listener: @escaping ((String) -> Void), to pipe: Pipe) -> NSObjectProtocol {
    let handler = pipe.fileHandleForReading
    handler.waitForDataInBackgroundAndNotify()
    var observer: NSObjectProtocol!
    observer = NotificationCenter.default.addObserver(
      forName: .NSFileHandleDataAvailable,
      object: handler,
      queue: nil
    ) { [weak handler] _ in
      guard let handler = handler else { return }
      let data = handler.availableData
      guard data.count > 0 else {
        handler.closeFile()
        return
      }
      if let line = String(data: data, encoding: .utf8) {
        listener(line)
      }
      handler.waitForDataInBackgroundAndNotify()
    }
    return observer
  }

  @discardableResult func runInBackground(
    cwd: String? = nil,
    with env: [String: String]? = nil,
    updateHandler: ((String, Process) -> Void)? = nil,
    completion: ((String) -> Void)? = nil
  ) -> Process {

    let stdin = Pipe()
    let stderr = Pipe()

    let task = Process()
    task.standardOutput = stdin
    task.standardError = stderr

    task.currentDirectoryPath = cwd ?? NSHomeDirectory()

    if let env = env {
      task.environment = env
    } else {
      task.environment = [:]
    }

    task.environment?["HOME"] = NSHomeDirectory()

    if let env = env, let shell = env["SHELL"] {
      task.launchPath = shell
    } else {
      task.launchPath = "/bin/sh"
    }
    var out: String = ""
    var observers: [NSObjectProtocol] = []
    if let updateHandler = updateHandler {
      let stdinObserver = addListener({ (line) in
        updateHandler(line, task)
        out += line
      }, to: stdin)

      let stderrObserver = addListener({ (line) in
        updateHandler(line, task)
        out += line
      }, to: stderr)

      observers.append(stdinObserver)
      observers.append(stderrObserver)
    }

    // add "-li" to get closer to terminal behavior
    task.arguments = ["-c", self]
    task.launch()
    DispatchQueue.global(qos: .background).async {
      task.waitUntilExit()
      if let completion = completion {
        completion(out)
      }

      observers.forEach { (observer) in
        NotificationCenter.default.removeObserver(observer)
      }
    }

    return task
  }

}

extension String {
  /*
   Truncates the string to the specified length number of characters and appends an optional trailing string if longer.
   - Parameter length: Desired maximum lengths of a string
   - Parameter trailing: A 'String' that will be appended after the truncation.

   - Returns: 'String' object.
   */
  func truncate(length: Int, trailing: String = "…") -> String {
    return (self.count > length) ? self.prefix(length) + trailing : self
  }
}
