//
//  TmuxIntegration.swift
//  fig
//
//  Created by Matt Schrage on 3/2/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class TmuxIntegration {

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
