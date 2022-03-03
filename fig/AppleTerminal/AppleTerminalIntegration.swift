//
//  AppleTerminalIntegration.swift
//  fig
//
//  Created by Matt Schrage on 9/20/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class AppleTerminalIntegration: TerminalIntegrationProvider {
  static let `default`: AppleTerminalIntegration = {
    let provider = AppleTerminalIntegration(bundleIdentifier: Integrations.Terminal)
    provider.applicationName = " Terminal"
    return provider
  }()

  func getCursorRect(in window: ExternalWindow) -> NSRect? {
    return Accessibility.getCursorRect()
  }

  func terminalIsFocused(in window: ExternalWindow) -> Bool {
    return true
  }

  // No installation necessary!
  func verifyInstallation() -> InstallationStatus {
    return .installed
  }
  
  func uninstall() -> Bool {
    return true
  }

  func install() -> InstallationStatus {
    return .installed
  }

}
