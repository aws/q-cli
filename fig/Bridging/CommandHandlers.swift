//
//  CommandHandlers.swift
//  fig
//
//  Created by Grant Gurvis on 10/21/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import Foundation

class CommandHandlers {}

extension CommandHandlers {
  static func logoutCommand() -> CommandResponse {
    let domain = Bundle.main.bundleIdentifier!
    let uuid = Defaults.uuid
    UserDefaults.standard.removePersistentDomain(forName: domain)
    UserDefaults.standard.removePersistentDomain(forName: "\(domain).shared")

    UserDefaults.standard.synchronize()

    UserDefaults.standard.set(uuid, forKey: "uuid")
    UserDefaults.standard.synchronize()

    WebView.deleteCache()

    Config.set(value: "0", forKey: "FIG_LOGGED_IN")

    return CommandResponse.with { response in
      response.success = Local_SuccessResponse.with({ success in
        success.message = "Logging out of Fig"
      })
    }
  }

  static func quitCommand() {
    NSApp.appDelegate.quit()
  }

  static func restartCommand() {
    NSApp.appDelegate.restart()
  }
  
  static func updateCommand(_ force: Bool = false) {
    DispatchQueue.main.async {
      if force {
        if UpdateService.provider.updateIsAvailable {
          UpdateService.provider.installUpdateIfAvailible()
        }
      } else {
        UpdateService.provider.checkForUpdates(nil)
      }
    }
  }
  
  static func diagnosticsCommand() -> CommandResponse {
    Logger.log(message: "Diagnostics ran")
    return CommandResponse.with { response in
      response.diagnostics.pathToBundle = Diagnostic.pathToBundle
      response.diagnostics.accessibility = String(Accessibility.enabled)
      response.diagnostics.keypath = Diagnostic.keybindingsPath ?? "<none>"
      response.diagnostics.docker = String(DockerEventStream.shared.socket.isConnected)
      response.diagnostics.symlinked = String(Diagnostic.dotfilesAreSymlinked)
      response.diagnostics.installscript = String(Diagnostic.installationScriptRan)
      response.diagnostics.securekeyboard = String(Diagnostic.secureKeyboardInput)
      response.diagnostics.securekeyboardPath = Diagnostic.blockingProcess ?? "<none>"
      response.diagnostics.currentWindowIdentifier = Diagnostic.descriptionOfTopmostWindow
      response.diagnostics.currentProcess = "\(Diagnostic.processForTopmostWindow) (\(Diagnostic.processIdForTopmostWindow)) - \(Diagnostic.ttyDescriptorForTopmostWindow)"
      response.diagnostics.onlytab = String(Defaults.onlyInsertOnTab)
      response.diagnostics.psudopath = Diagnostic.pseudoTerminalPath ?? "<generated dynamically>"
    }
  }
  
  static func displayReportWindow(message: String, path: String?, figEnvVar: String?, terminal: String?) {
    let placeholder =
    """
    \(message)
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    ---------------------------------------
    DIAGNOSTIC
    \(Diagnostic.summary)
    ---------------------------------------
    ENVIRONMENT
    Terminal: \(terminal ?? "<unknown>")
    PATH: \(path ?? "Not found")
    FIG_ENV_VAR: \(figEnvVar ?? "Not found")
    --------------------------------------
    CONFIG
    \(Diagnostic.userConfig ?? "?")
    """
    DispatchQueue.main.async {
      Feedback.getFeedback(source: "fig_report_cli", placeholder: placeholder)
    }
  }
  
  static func buildCommand(build: String?) -> CommandResponse {
    if let buildMode = Build(rawValue: build ?? "") {
      Defaults.build = buildMode
      return CommandResponse.with { response in
        response.success.message = buildMode.rawValue;
      }
    } else {
      return CommandResponse.with { response in
        response.success.message = Defaults.build.rawValue;
      }
    }
  }
  
  static func sourceCommand() {
    
  }
  
  static func restartSettingsListenerCommand() -> CommandResponse {
    DispatchQueue.main.async {
      Settings.shared.restartListener()
    }
    
    return CommandResponse.with { response in
      response.success.message = "restarting ~/.fig/settings.json file watcher"
    }
  }
  
  static func runInstallScriptCommand() -> CommandResponse {
    Onboarding.setUpEnviroment()
    
    return CommandResponse.with { response in
      response.success.message = "running installation script"
    }
  }
  
  static func buildCommand(branch: String?) -> CommandResponse? {
    if let buildMode = Build(rawValue: branch ?? "") {
        Defaults.build = buildMode
    } else {
      return CommandResponse.with { response in
        response.success.message = Defaults.build.rawValue
      }
    }
    return nil
  }
}
