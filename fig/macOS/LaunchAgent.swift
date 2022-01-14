//
//  LaunchAgentHelper.swift
//  fig
//
//  Created by Matt Schrage on 1/11/22.
//  Copyright © 2022 Matt Schrage. All rights reserved.
//

import Foundation

class LaunchAgent {
    static var launchAgentDirectory: URL? {
        let libDir = try? FileManager.default.url(for: .libraryDirectory,
                                                  in: .userDomainMask,
                                                  appropriateFor: nil,
                                                  create: false)

        return libDir?.appendingPathComponent("LaunchAgents")
    }

  static let launchOnStartup = LaunchAgent(fileName: "io.fig.launcher.plist",
                                           plist: [
                                              "Label": "io.fig.launcher",
                                              "Program": Bundle.main.executablePath!,
                                              "RunAtLoad": true
                                           ])

  // https://stackoverflow.com/questions/4377015/uninstaller-for-a-cocoa-application
  static let uninstallWatcher = LaunchAgent(fileName: "io.fig.uninstall.plist",
                                            plist: [
                                              "Label": "io.fig.uninstall",
                                              "WatchPaths": [ "~/.Trash" ],
                                              "ProgramArguments": [ "osascript",
                                                                    "~/.fig/tools/uninstaller.scpt",
                                                                    Bundle.main.bundlePath
                                                                  ],
                                              "KeepAlive": false
                                            ])
  let launchAgentFile: URL?
  let plist: NSDictionary
  init(fileName: String, plist: NSDictionary) {
    self.launchAgentFile = LaunchAgent.launchAgentDirectory?.appendingPathComponent(fileName)
    self.plist = plist
  }

    static var launchAgentFile: URL? {
        launchAgentDirectory?.appendingPathComponent("io.fig.launcher.plist")
    }

    func add() {
        guard let launchAgentDirectory = LaunchAgent.launchAgentDirectory else {
          LaunchAgent.log("Error: could not access launch agent directory")
            return
        }

        guard let launchAgentFile = launchAgentFile else {
            LaunchAgent.log("Error: could not access launch agent file")
            return
        }

        if (launchAgentDirectory as NSURL).checkResourceIsReachableAndReturnError(nil) == false {
            _ = try? FileManager.default.createDirectory(at: launchAgentDirectory,
                                                         withIntermediateDirectories: false,
                                                         attributes: nil)
        }

        self.plist.write(to: launchAgentFile, atomically: true)
    }

    func addIfNotPresent() {
      if !self.enabled() {
        self.add()
      }
    }

    func remove() {
      _ = try? FileManager.default.removeItem(at: self.launchAgentFile!)
    }

    func enabled() -> Bool {
      let reachable = (self.launchAgentFile as NSURL?)?.checkResourceIsReachableAndReturnError(nil)
        return reachable ?? false
    }
}

extension LaunchAgent: Logging {
  static func log(_ message: String) {
    Logger.log(message: message, subsystem: .launchAgent)
  }
}
