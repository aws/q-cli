//
//  TmuxIntegration.swift
//  fig
//
//  Created by Matt Schrage on 3/2/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class TmuxIntegration {

  //  func update(tty: TTY, for process: proc) {
  //
  //    tty.pty!.execute("tmux list-panes -a -F \"#{pane_pid} #{pane_id}\"") { panes in
  //      print("tmux: \(panes)")
  //      let mapping: [String: pid_t] = panes.split(separator: "\r\n")
  //                                          .map { $0.split(separator: " ") }
  //                                          .reduce([:]) { (out, tokens) -> [String: pid_t] in
  //        guard let pidString = tokens.first, let pid = pid_t(pidString), let paneId = tokens.last else { return out }
  //        print("tmux: \(paneId), \(pid)")
  //        var new = out
  //        new[String(paneId)] = pid
  //        return new
  //      }
  //
  //      guard let (_, _, p) = AXWindowServer.shared.allowlistedWindow?.hash.components(), let pane = p else {
  //        print("tmux: could not get windowHash components")
  //        return
  //      }
  //
  //      guard let pane = self.activePane else {
  //        print("tmux: no active pane")
  //        return
  //      }
  //
  //      print("tmux: activePane? \(pane)")
  //      guard let tmux_pid = mapping["%\(pane)"] else {
  //        print("tmux: could not find pid for pane %\(pane)")
  //        return
  //      }
  //
  //      guard let activeTmuxProcess = (ProcessStatus.getProcesses(for: "").filter {  $0.pid == tmux_pid }).first else
  //      { print("tmux: could not find process for pid")
  //        return
  //      }
  //
  //      tty.cwd = activeTmuxProcess.cwd
  //      tty.cmd = activeTmuxProcess.cmd // don't update command
  //      tty.pid = activeTmuxProcess.pid
  //      tty.isShell = activeTmuxProcess.isShell
  //      tty.runUsingPrefix = nil
  //    }
  //  }
  //
  //  func runUsingPrefix() -> String? {
  //    return nil
  //  }
  //
  //  var activePane: String?
  //  func initialize(tty: TTY) {
  //
  //  }
  //
  //  func setActivePane(_ id: String, in tty: TTY) {
  //    self.activePane = id.replacingOccurrences(of: "%", with: "")
  //    tty.update()
  //  }
  //
  //  static var command: String = "tmux"
  //
  //  func shouldHandleProcess(_ process: proc) -> Bool {
  //        return process.name == TmuxIntegration.command
  //  }
  //

  static var settingsPath: String {
    let defaultPath = "\(NSHomeDirectory())/.tmux.conf"
    return (try? FileManager.default.destinationOfSymbolicLink(atPath: defaultPath)) ?? defaultPath
  }

  // Make sure to update in uninstall script if the payload is changed
  static let payload =
    """
  # Fig Tmux Integration: Enabled
  source-file ~/.fig/tmux
  # End of Fig Tmux Integration
  """

  static var isInstalled: Bool {

    guard let configuration = try? String(contentsOfFile: settingsPath), configuration.contains(payload) else {
      return false
    }

    return true
  }

}
