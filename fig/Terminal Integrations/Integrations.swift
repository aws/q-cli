//
//  Integrations.swift
//  fig
//
//  Created by Matt Schrage on 3/1/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import FigAPIBindings

class Integrations {
  static let iTerm = "com.googlecode.iterm2"
  static let Terminal = "com.apple.Terminal"
  static let Hyper = "co.zeit.hyper"
  static let VSCode = "com.microsoft.VSCode"
  static let VSCodeInsiders = "com.microsoft.VSCodeInsiders"
  static let VSCodiumLegacy = "com.visualstudio.code.oss"
  static let VSCodium = "com.vscodium"
  static let Kitty = "net.kovidgoyal.kitty"
  static let Alacritty = "io.alacritty"
  static let Tabby = "org.tabby"
  static let IntellijCE = "com.jetbrains.intellij.ce"
  static let Intellij = "com.jetbrains.intellij"
  static let WebStorm = "com.jetbrains.WebStorm"
  static let GoLand = "com.jetbrains.goland"
  static let PhpStorm = "com.jetbrains.PhpStorm"
  static let PyCharm = "com.jetbrains.pycharm"
  static let PyCharmCE = "com.jetbrains.pycharm.ce"
  static let AppCode = "com.jetbrains.AppCode"
  static let Rider = "com.jetbrains.rider"
  static let CLion = "com.jetbrains.CLion"
  static let RubyMine = "com.jetbrains.rubymine"
  static let DataSpell = "com.jetbrains.dataspell"
  static let Nova = "com.panic.Nova"
  static let AndroidStudio = "com.google.android.studio"
  static let WezTerm = "com.github.wez.wezterm"

  static let terminals: Set = [
    "com.googlecode.iterm2",
    "com.apple.Terminal",
    "io.alacritty",
    "co.zeit.hyper",
    "net.kovidgoyal.kitty"
  ]
  static let browsers: Set = ["com.google.Chrome"]
  static let editors: Set = [
    "com.apple.dt.Xcode",
    "com.sublimetext.3",
    "com.microsoft.VSCode"
  ]
  static let nativeTerminals: Set = [
    "com.googlecode.iterm2",
    "com.apple.Terminal"
  ]
  static let searchBarApps: Set = [
    "com.apple.Spotlight",
    "com.runningwithcrayons.Alfred",
    "com.raycast.macos"
  ]

  static let jetbrainsIDEs: Set = [
    IntellijCE,
    Intellij,
    WebStorm,
    GoLand,
    PhpStorm,
    PyCharm,
    PyCharmCE,
    AppCode,
    Rider,
    CLion,
    AndroidStudio,
    RubyMine
  ]
  static let inputMethodDependentTerminals: Set = jetbrainsIDEs.union([Alacritty, Kitty, Nova, WezTerm])

  static let electronIDEs: Set = [VSCode, VSCodeInsiders, VSCodium, VSCodiumLegacy]
  static var electronTerminals: Set<String> {
    let additions = Set(
      Settings.shared.getValue(forKey: Settings.additionalElectronTerminalsKey) as? [String] ?? [])

    return
      additions
      .union(Integrations.electronIDEs)
      .union([Integrations.Hyper, Integrations.Tabby])
  }

  static var terminalsWhereAutocompleteShouldAppear: Set<String> {
    let additions = Set(
      Settings.shared.getValue(forKey: Settings.additionalTerminalsKey) as? [String] ?? [])
    return Set(Integrations.providers.keys)
      .union(additions)
      .subtracting(Integrations.autocompleteBlocklist)
  }

  static func bundleIsValidTerminal(_ bundle: String?) -> Bool {
    return Integrations.terminalsWhereAutocompleteShouldAppear.contains(bundle ?? "")
  }

  static func frontmostApplicationIsValidTerminal() -> Bool {
    return bundleIsValidTerminal(NSWorkspace.shared.frontmostApplication?.bundleIdentifier)
  }

  static var autocompleteBlocklist: Set<String> {
    var blocklist: Set<String> = []
    if let hyperDisabled = Settings.shared.getValue(forKey: Settings.hyperDisabledKey) as? Bool,
       hyperDisabled {
      blocklist.insert(Integrations.Hyper)
    }

    if let vscodeDisabled = Settings.shared.getValue(forKey: Settings.vscodeDisabledKey) as? Bool,
       vscodeDisabled {
      blocklist.insert(Integrations.VSCode)
      blocklist.insert(Integrations.VSCodeInsiders)
    }

    if let itermDisabled = Settings.shared.getValue(forKey: Settings.iTermDisabledKey) as? Bool,
       itermDisabled {
      blocklist.insert(Integrations.iTerm)
    }

    if let terminalDisabled = Settings.shared.getValue(forKey: Settings.terminalDisabledKey)
        as? Bool, terminalDisabled {
      blocklist.insert(Integrations.Terminal)
    }
    return blocklist
  }

  static var allowed: Set<String> {
    if let allowed = UserDefaults.standard.string(forKey: "allowedApps") {
      return Set(allowed.split(separator: ",").map({ String($0) }))
    } else {
      return []
    }
  }

  static var blocked: Set<String> {
    if let allowed = UserDefaults.standard.string(forKey: "blockedApps") {
      return Set(allowed.split(separator: ",").map({ String($0) }))
    } else {
      return []
    }
  }

  static var allowlist: Set<String> {
    return Integrations.terminals
      .union(Integrations.allowed)
      .subtracting(Integrations.blocked)
  }

  static let accessibilityDependentIntegrations: [String: TerminalIntegrationProvider] =
  [
    Integrations.iTerm: iTermIntegration.default,
    Integrations.Hyper: HyperIntegration.default,
    Integrations.VSCode: VSCodeIntegration.default,
    Integrations.VSCodeInsiders: VSCodeIntegration.insiders,
    Integrations.VSCodium: VSCodeIntegration.vscodium,
    Integrations.VSCodiumLegacy: VSCodeIntegration.vscodiumLegacy,
    Integrations.Terminal: AppleTerminalIntegration.default,
    Integrations.Tabby: TabbyIntegration.default
  ]

  static let inputMethodDependentIntegrations: [String: TerminalIntegrationProvider] =
  [
    Integrations.Alacritty: AlacrittyIntegration.default,
    Integrations.Kitty: KittyIntegration.default,
    Integrations.WezTerm: WezTermIntegration.default,
    Integrations.Nova: NovaIntegration.default,
    // Jetbrains IDEs
    Integrations.Intellij: JetBrainsIntegration.idea,
    Integrations.IntellijCE: JetBrainsIntegration.ideaCE,
    Integrations.WebStorm: JetBrainsIntegration.WebStorm,
    Integrations.PhpStorm: JetBrainsIntegration.PhpStorm,
    Integrations.GoLand: JetBrainsIntegration.GoLand,
    Integrations.PyCharm: JetBrainsIntegration.PyCharm,
    Integrations.PyCharmCE: JetBrainsIntegration.PyCharmCE,
    Integrations.AppCode: JetBrainsIntegration.AppCode,
    Integrations.CLion: JetBrainsIntegration.CLion,
    Integrations.Rider: JetBrainsIntegration.Rider,
    Integrations.AndroidStudio: JetBrainsIntegration.AndroidStudio,
    Integrations.RubyMine: JetBrainsIntegration.RubyMine,
    Integrations.DataSpell: JetBrainsIntegration.DataSpell
  ]

  static let providers: [String: TerminalIntegrationProvider] = accessibilityDependentIntegrations.merging(inputMethodDependentIntegrations) { $1 }

  static func handleListIntegrationsRequest() -> CommandResponse {
    CommandResponse.with { response in
      response.integrationList = Local_TerminalIntegrationsListResponse.with({ list in
        list.integrations = Integrations.providers.map({ (key: String, value: TerminalIntegrationProvider) in
          Local_TerminalIntegration.with { integration in
            integration.bundleIdentifier = key
            integration.name = value.applicationName
            integration.status = value.status.description
          }
        })
      })
    }
  }
}

protocol IntegrationProvider {
  // access the stored value (no calculation)
  var status: InstallationStatus { get }

  // idempotent (but potentially expensive) function to determine whether the integration is installed
  func verifyInstallation() -> InstallationStatus

  // update the user's environment to install the integration
  func install() -> InstallationStatus

  // update the user's environment to uninstall the integration
  func uninstall() -> Bool

  var id: String { get }
}

struct Plugin {
  let name: String
  let version: String
  let resourceInBundle: URL

  init(name: String, version: String, fileExtension: String) {
    self.name = name
    self.version = version

    self.resourceInBundle = Bundle.main.url(forResource: Plugin.slug(name: name, version: version),
                                            withExtension: fileExtension)!
  }

  var slug: String {
    return Plugin.slug(name: self.name, version: self.version)
  }

  fileprivate static func slug(name: String, version: String) -> String {
    return name + "-" + version
  }
}
