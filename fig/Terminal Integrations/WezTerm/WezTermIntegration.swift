//
//  WezTermIntegration.swift
//  fig
//
//  Created by Matt Schrage on 3/26/22.
//  Copyright © 2022 Matt Schrage. All rights reserved.
//

import Foundation

class WezTermIntegration: InputMethodDependentTerminalIntegrationProvider & IntegrationProvider {
  static let `default` = WezTermIntegration(bundleIdentifier: Integrations.WezTerm)

  func verifyInstallation() -> InstallationStatus {
    guard self.applicationIsInstalled else {
      return .applicationNotInstalled
    }

    let inputMethodStatus = InputMethod.default.verifyInstallation()
    guard inputMethodStatus == .installed else {
      return .pending(event: .inputMethodActivation)
    }

    // If the application is already running,
    // it must be restarted for the new input method to work
    if self.status == .pending(event: .inputMethodActivation) {
      return .pending(event: .applicationRestart)
    }

    return .installed
  }

  func uninstall() -> Bool {
    return true
  }

  func install() -> InstallationStatus {
    guard self.applicationIsInstalled else {
      return .applicationNotInstalled
    }

    if !InputMethod.default.isInstalled {
      let status = InputMethod.default.install()
      guard status == .installed else {
        return .pending(event: .inputMethodActivation)
      }

    }

    return .installed
  }

}

extension WezTermIntegration: TerminalIntegration {
  func getCursorRect(in window: ExternalWindow) -> NSRect? {
    return InputMethod.getCursorRect()
  }

  func terminalIsFocused(in window: ExternalWindow) -> Bool {
    return true
  }

}
