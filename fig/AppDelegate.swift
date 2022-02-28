//
//  AppDelegate.swift
//  fig
//
//  Created by Matt Schrage on 4/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa
import Sparkle
import WebKit
import Sentry

@NSApplicationMain
// swiftlint:disable:next type_body_length
class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate {

  var window: NSWindow!
  var onboardingWindow: WebViewWindow!
  var statusBarItem: NSStatusItem!
  var frontmost: NSMenuItem?
  var integrationPrompt: NSMenuItem?

  var clicks: Int = 6
  let updater = UpdateService.provider

  let iTermObserver = WindowObserver(with: "com.googlecode.iterm2")
  // swiftlint:disable:next identifier_name
  let TerminalObserver = WindowObserver(with: "com.apple.Terminal")
  // swiftlint:disable:next identifier_name
  let HyperObserver = WindowObserver(with: Integrations.Hyper)
  let VSCodeObserver = WindowObserver(with: Integrations.VSCode)

  func applicationDidFinishLaunching(_ aNotification: Notification) {
    Logger.resetLogs()
    SentrySDK.start { options in
      options.dsn = "https://4544a50058a645f5a779ea0a78c9e7ec@o436453.ingest.sentry.io/5397687"
      options.debug = false // Enabled debug when first installing is always helpful
      options.enableAutoSessionTracking = true
      options.attachStacktrace = true
      options.sessionTrackingIntervalMillis = 5_000
      options.enabled = !Defaults.shared.telemetryDisabled
    }
    warnToMoveToApplicationIfNecessary()

    // Set timeout to avoid hanging misbehaving 3rd party apps
    Accessibility.setGlobalTimeout(seconds: 2)

    if let hideMenuBar = Settings.shared.getValue(forKey: Settings.hideMenubarIcon) as? Bool, hideMenuBar {
      print("Not showing menubarIcon because of \(Settings.hideMenubarIcon)")
    } else {
      let statusBar = NSStatusBar.system
      statusBarItem = statusBar.statusItem(
        withLength: NSStatusItem.squareLength)
      statusBarItem.button?.title = "🍐"
      statusBarItem.button?.image = NSImage(imageLiteralResourceName: "statusbar@2x.png")// .overlayBadge()
      statusBarItem.button?.image?.isTemplate = true
      statusBarItem.button?.wantsLayer = true
    }

    //        NSApp.setActivationPolicy(NSApplication.ActivationPolicy.accessory)
    // prevent multiple sessions
    let bundleID = Bundle.main.bundleIdentifier!
    if NSRunningApplication.runningApplications(withBundleIdentifier: bundleID).count > 1 {
      SentrySDK.capture(message: "Multiple Fig instances running!")
      Logger.log(message: "Multiple Fig instances running! Terminating now!")
      NSRunningApplication.runningApplications(withBundleIdentifier: bundleID)
        .filter { $0.processIdentifier != NSRunningApplication.current.processIdentifier }
        .forEach { (app) in
          Logger.log(message: "Existing Process Id = \(app.processIdentifier)")
          app.forceTerminate()
        }
    }

    TelemetryProvider.shared.track(
      event: .launchedApp,
      with: ["crashed": Defaults.shared.launchedFollowingCrash ? "true" : "false"]
    )
    Defaults.shared.launchedFollowingCrash = true
    Config.shared.set(value: nil, forKey: Config.userExplictlyQuitApp)
    Accessibility.checkIfPermissionRevoked()

    //        AppMover.moveIfNecessary()
    _ = LocalState.shared
    _ = Settings.shared
    _ = ShellBridge.shared
    _ = WindowManager.shared
    _ = ShellHookManager.shared
    _ = KeypressProvider.shared

    _ = AXWindowServer.shared
    _ = TerminalSessionLinker.shared

    _ = IPC.shared

    _ = DockerEventStream.shared
    _ = iTermIntegration.default
    _ = InputMethod.default

    TelemetryProvider.shared.register()
    Accessibility.listen()

    handleUpdateIfNeeded()
    Defaults.shared.useAutocomplete = true
    Defaults.shared.autocompleteVersion = "v9"

    Defaults.shared.autocompleteWidth = 250
    Defaults.shared.ignoreProcessList = [
      "figcli",
      "gitstatusd-darwin-x86_64",
      "gitstatusd-darwin-arm64",
      "nc",
      "fig_pty",
      "starship",
      "figterm"
    ]

    let hasLaunched = UserDefaults.standard.bool(forKey: "hasLaunched")
    let email = UserDefaults.standard.string(forKey: "userEmail")

    if !hasLaunched || email == nil {
      Defaults.shared.loggedIn = false
      Defaults.shared.build = .production
      Defaults.shared.clearExistingLineOnTerminalInsert = true
      Defaults.shared.showSidebar = false

      Config.shared.set(value: "0", forKey: Config.userLoggedIn)
      //            Defaults.shared.defaultActivePosition = .outsideRight

      let onboardingViewController = WebViewController()
      onboardingViewController.webView?.defaultURL = nil
      onboardingViewController.webView?.loadBundleApp("landing")
      onboardingViewController.webView?.dragShouldRepositionWindow = true

      onboardingWindow = WebViewWindow(viewController: onboardingViewController)
      onboardingWindow.makeKeyAndOrderFront(nil)
      onboardingWindow.setFrame(NSRect(x: 0, y: 0, width: 590, height: 480), display: true, animate: false)
      onboardingWindow.center()
      onboardingWindow.appearance = NSAppearance(named: NSAppearance.Name.vibrantLight)

      onboardingWindow.makeKeyAndOrderFront(self)

      UserDefaults.standard.set(true, forKey: "hasLaunched")
      UserDefaults.standard.synchronize()
    } else {
      // identify user for Sentry!
      let user = User()
      user.email = email
      SentrySDK.setUser(user)
      ShellBridge.symlinkCLI()
      Config.shared.set(value: "1", forKey: Config.userLoggedIn)
      UpdateService.provider.resetShellConfig()

      if !Accessibility.enabled {
        Accessibility.showPromptUI()
      }

      self.setupCompanionWindow()
    }

    configureStatusBarItem()
    setUpAccesibilityObserver()
    NotificationCenter.default.addObserver(self,
                                           selector: #selector(windowDidChange(_:)),
                                           name: AXWindowServer.windowDidChangeNotification,
                                           object: nil)

    NotificationCenter.default.addObserver(self,
                                           selector: #selector(settingsUpdated),
                                           name: Settings.settingsUpdatedNotification,
                                           object: nil)

    NotificationCenter.default.addObserver(self,
                                           selector: #selector(integrationsUpdated),
                                           name: Integrations.statusDidChange,
                                           object: nil)

    if let shouldLaunchOnStartup = Settings.shared.getValue(forKey: Settings.launchOnStartupKey) as? Bool,
       !shouldLaunchOnStartup {
      LaunchAgent.launchOnStartup.remove()
    } else {
      LaunchAgent.launchOnStartup.addIfNotPresent()
    }

    //        iTermTabIntegration.listenForHotKey()
    SecureKeyboardInput.listen()
//    InputMethod.default.uninstall()
    iTermObserver?.windowDidAppear {
      SecureKeyboardInput.notifyIfEnabled()
    }

    TerminalObserver?.windowDidAppear {
      SecureKeyboardInput.notifyIfEnabled()
    }

    VSCodeObserver?.windowDidAppear {
      SecureKeyboardInput.notifyIfEnabled()
      Accessibility.triggerScreenReaderModeInFrontmostApplication()
      //          if !VSCodeIntegration.isInstalled {
      //            VSCodeIntegration.promptToInstall()
      //          }
    }

    HyperObserver?.windowDidAppear {
      SecureKeyboardInput.notifyIfEnabled()
      Accessibility.triggerScreenReaderModeInFrontmostApplication()
      //          if !HyperIntegration.isInstalled {
      //            HyperIntegration.promptToInstall()
      //          }
    }

    Integrations.providers.values.forEach { provider in
      if !provider.isInstalled {
        provider.install(withRestart: false, inBackground: true)
      }
    }

  }

  func warnToMoveToApplicationIfNecessary() {
    if Diagnostic.isRunningOnReadOnlyVolume && !Defaults.shared.loggedIn {
      Alert.show(title: "Move to Applications folder",
                 message: "Fig needs to be launched from your Applications folder in order to work properly.",
                 okText: "Quit",
                 icon: NSApp.applicationIconImage)

      NSApp.terminate(self)

    }

    // Get rid of reference to DMG if it exists in LoginItems
    // let dmgAppURL = NSURL.fileURL(withPath: "/Volumes/Fig/Fig.app")
    // LoginItems.shared.removeURLIfExists(dmgAppURL)
  }

  func remindToSourceFigInExistingTerminals() {

    // filter for native terminal windows (with hueristic to avoid menubar items + other window types)
    let nativeTerminals = NSWorkspace.shared.runningApplications.filter {
      Integrations.nativeTerminals.contains($0.bundleIdentifier ?? "")
    }

    let count = nativeTerminals.count
    guard count > 0 else { return }
    let iTermOpen = nativeTerminals.contains { $0.bundleIdentifier == "com.googlecode.iterm2" }
    let terminalAppOpen = nativeTerminals.contains { $0.bundleIdentifier == "com.apple.Terminal" }

    var emulators: [String] = []

    if iTermOpen {
      emulators.append("iTerm")
    }

    if terminalAppOpen {
      emulators.append("Terminal")
    }

    let restart = self.dialogOKCancel(
      question: "Restart existing terminal sessions?",
      text: "Any terminal sessions started before Fig are not tracked.\n\n" +
        "Run `fig source` in each session to connect or restart your terminals.\n",
      prompt: "Restart \(emulators.joined(separator: " and "))",
      noAction: false,
      icon: NSImage.init(imageLiteralResourceName: NSImage.applicationIconName)
    )

    if restart {
      print("restart")

      if iTermOpen {
        let iTerm = Restarter(with: "com.googlecode.iterm2")
        iTerm.restart()
      }

      if terminalAppOpen {
        let terminalApp = Restarter(with: "com.apple.Terminal")
        terminalApp.restart()
      }
    }
  }

  func openMenu() {
    if let hidden = Settings.shared.getValue(forKey: Settings.hideMenubarIcon) as? Bool,
       hidden {
      return
    }

    if let menu = self.statusBarItem.menu {
      self.statusBarItem.popUpMenu(menu)
    }
  }

  @objc func statusBarButtonClicked(sender: NSStatusBarButton) {
    let event = NSApp.currentEvent!

    if event.type == NSEvent.EventType.leftMouseUp {
      sender.menu = self.defaultStatusBarMenu()
      if let menu = sender.menu {
        menu.popUp(positioning: nil, at: NSPoint(x: 0, y: statusBarItem.statusBar!.thickness), in: sender)
      }

      // This is critical, otherwise clicks won't be processed again
      sender.menu = nil
    } else {
      // control + click!
      print("statusbar: button left clicked. Could be used for debugging menu!")
    }
  }

  func alertStatusBarMenu() -> NSMenu {
    let statusBarMenu = NSMenu(title: "fig")
    statusBarItem.menu = statusBarMenu
    statusBarMenu.addItem(
      withTitle: "Fig is disabled...",
      action: nil,
      keyEquivalent: "")

    statusBarMenu.addItem(
      withTitle: "Turn on Accessibility",
      action: #selector(AppDelegate.promptForAccesibilityAccess),
      keyEquivalent: "")

    statusBarMenu.addItem(NSMenuItem.separator())
    let enable = statusBarMenu.addItem(
      withTitle: "You may need to toggle the",
      action: nil,
      keyEquivalent: "")
    enable.image = NSImage(imageLiteralResourceName: NSImage.smartBadgeTemplateName)

    let inset = statusBarMenu.addItem(
      withTitle: "  checkbox off and on",
      action: nil,
      keyEquivalent: "")
    inset.indentationLevel = 1

    statusBarMenu.addItem(NSMenuItem.separator())
    statusBarMenu.addItem(
      withTitle: "Quit Fig",
      action: #selector(AppDelegate.quit),
      keyEquivalent: "")

    return statusBarMenu
  }

  func onboardingStatusBarMenu() -> NSMenu {
    let statusBarMenu = NSMenu(title: "fig")
    statusBarItem.menu = statusBarMenu
    statusBarMenu.addItem(
      withTitle: "Fig hasn't been set up yet...",
      action: nil,
      keyEquivalent: "")

    statusBarMenu.addItem(NSMenuItem.separator())

    statusBarMenu.addItem(NSMenuItem.separator())
    let issue = statusBarMenu.addItem(
      withTitle: "Report a bug...",
      action: #selector(AppDelegate.sendFeedback),
      keyEquivalent: "")
    issue.image = NSImage(imageLiteralResourceName: "github")

    let forum = statusBarMenu.addItem(
      withTitle: "Support Guide",
      action: #selector(AppDelegate.viewSupportForum),
      keyEquivalent: "")
    forum.image = NSImage(named: NSImage.Name("commandkey"))

    statusBarMenu.addItem(NSMenuItem.separator())

    statusBarMenu.addItem(
      withTitle: "Quit Fig",
      action: #selector(AppDelegate.quit),
      keyEquivalent: "")

    statusBarMenu.addItem(NSMenuItem.separator())
    statusBarMenu.addItem(
      withTitle: "Uninstall Fig",
      action: #selector(AppDelegate.uninstall),
      keyEquivalent: "")

    return statusBarMenu
  }

  func integrationsMenu() -> NSMenu {
    let integrationsMenu = NSMenu(title: "fig")

    integrationsMenu.addItem(NSMenuItem.separator())
    integrationsMenu.addItem(
      withTitle: "Terminal Integrations",
      action: nil,
      keyEquivalent: "")

    Integrations.providers.values.sorted(by: { lhs, rhs in
      return lhs.applicationName.lowercased() >= rhs.applicationName.lowercased()
    }).forEach { provider in
      guard provider.applicationIsInstalled else { return }

      let name = provider.applicationName
      let item = integrationsMenu.addItem(
        withTitle: name, // + " Integration",
        action: #selector(provider.promptToInstall),
        keyEquivalent: "")

      item.target = provider

      switch provider.status {
      case .applicationNotInstalled:
        break
      case .unattempted:
        item.image = Icon.fileIcon(for: "fig://template?color=808080&badge=?&w=16&h=16")
      case .installed:
        item.action = nil // disable selection
        item.image = Icon.fileIcon(for: "fig://template?color=2ecc71&badge=✓&w=16&h=16")
      case .pending(let dependency):
        let actionsMenu = NSMenu(title: "actions")

        item.action = nil // disable selection

        switch dependency {
        case .applicationRestart:
          item.image = Icon.fileIcon(for: "fig://template?color=FFA500&badge=⟳&w=16&h=16")

          let restart = actionsMenu.addItem(
            withTitle: "Restart \(provider.applicationName)",
            action: #selector(provider.restart),
            keyEquivalent: "")
          restart.target = provider
        case .inputMethodActivation:
          item.image = Icon.fileIcon(for: "fig://template?color=FFA500&badge=⌨&w=16&h=16")
          actionsMenu.addItem(
            withTitle: "Requires Input Method",
            action: nil,
            keyEquivalent: "")

          switch InputMethod.default.status {
          case .failed(let error, _):
            actionsMenu.addItem(NSMenuItem.separator())
            actionsMenu.addItem(
              withTitle: error,
              action: nil,
              keyEquivalent: "")
            actionsMenu.addItem(NSMenuItem.separator())
            let installer = actionsMenu.addItem(
              withTitle: "Attempt to Install",
              action: #selector(provider.promptToInstall),
              keyEquivalent: "")
            installer.target = provider
          default:
            break
          }

        }

        item.submenu = actionsMenu
      case .failed(let error, let supportURL):
        item.image = Icon.fileIcon(for: "fig://template?color=e74c3c&badge=╳&w=16&h=16")
        let actionsMenu = NSMenu(title: "actions")

        actionsMenu.addItem(
          withTitle: error,
          action: nil,
          keyEquivalent: "")

        actionsMenu.addItem(NSMenuItem.separator())
        let install = actionsMenu.addItem(withTitle: "Attempt to install",
                                          action: #selector(provider.promptToInstall),
                                          keyEquivalent: "")
        install.target = provider

        if supportURL != nil {
          actionsMenu.addItem(NSMenuItem.separator())

          let button = actionsMenu.addItem(
            withTitle: "Learn more",
            action: #selector(provider.openSupportPage),
            keyEquivalent: "")

          button.target = provider
        }

        item.submenu = actionsMenu
      }
    }

    integrationsMenu.addItem(NSMenuItem.separator())

    let developer = integrationsMenu.addItem(
      withTitle: "Developer",
      action: nil,
      keyEquivalent: "")
    developer.submenu = developerMenu()

    integrationsMenu.addItem(NSMenuItem.separator())

    integrationsMenu.addItem(
      withTitle: "Uninstall Fig",
      action: #selector(AppDelegate.uninstall),
      keyEquivalent: "")

    return integrationsMenu
  }

  func developerMenu() -> NSMenu {
    let developerMenu = NSMenu(title: "Developer")

    developerMenu.addItem(
      withTitle: "Install CLI Tool",
      action: #selector(AppDelegate.addCLI),
      keyEquivalent: "")
    developerMenu.addItem(
      withTitle: "Request Accessibility Permission",
      action: #selector(AppDelegate.promptForAccesibilityAccess),
      keyEquivalent: "")
    developerMenu.addItem(NSMenuItem.separator())

    developerMenu.addItem(
      withTitle: "Run Install/Update Script",
      action: #selector(AppDelegate.setupScript),
      keyEquivalent: "")

    if !Defaults.shared.isProduction {
      developerMenu.addItem(
        withTitle: "Internal (not for prod)",
        action: nil,
        keyEquivalent: "")
      developerMenu.addItem(
        withTitle: "Flush logs",
        action: #selector(AppDelegate.flushLogs),
        keyEquivalent: "")
      developerMenu.addItem(
        withTitle: "Windows",
        action: #selector(AppDelegate.allWindows),
        keyEquivalent: "")
      developerMenu.addItem(
        withTitle: "Keyboard",
        action: #selector(AppDelegate.getKeyboardLayout),
        keyEquivalent: "")
      developerMenu.addItem(
        withTitle: "AXObserver",
        action: #selector(AppDelegate.addAccessibilityObserver),
        keyEquivalent: "")
      developerMenu.addItem(
        withTitle: "Get Selected Text",
        action: #selector(AppDelegate.getSelectedText),
        keyEquivalent: "")
      developerMenu.addItem(
        withTitle: "Processes",
        action: #selector(AppDelegate.processes),
        keyEquivalent: "")
    }

    return developerMenu
  }

  func defaultStatusBarMenu() -> NSMenu {

    let statusBarMenu = NSMenu(title: "fig")
    statusBarMenu.addItem(NSMenuItem.separator())

    let autocomplete = statusBarMenu.addItem(
      withTitle: "Autocomplete", // (βeta)
      action: #selector(AppDelegate.toggleAutocomplete(_:)),
      keyEquivalent: "")
    autocomplete.state = Defaults.shared.useAutocomplete ? .on : .off
    autocomplete.indentationLevel = 1
    statusBarMenu.addItem(NSMenuItem.separator())

    //        statusBarMenu.addItem(NSMenuItem.separator())

    let forum = statusBarMenu.addItem(
      withTitle: "Support Guide",
      action: #selector(AppDelegate.viewSupportForum),
      keyEquivalent: "")
    forum.image = NSImage(named: NSImage.Name("commandkey"))

    let slack = statusBarMenu.addItem(
      withTitle: "Join Fig Community",
      action: #selector(AppDelegate.inviteToSlack),
      keyEquivalent: "")
    slack.image = NSImage(named: NSImage.Name("discord"))// .resized(to: NSSize(width: 16, height: 16))
    statusBarMenu.addItem(NSMenuItem.separator())
    let settings = statusBarMenu.addItem(
      withTitle: "Settings",
      action: #selector(Settings.openUI),
      keyEquivalent: "")
    settings.image = NSImage(imageLiteralResourceName: "gear")
    settings.target = Settings.self

    statusBarMenu.addItem(NSMenuItem.separator())

    if let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String,
       let build = Bundle.main.infoDictionary?["CFBundleVersion"] as? String {
      statusBarMenu.addItem(withTitle: "Version \(version) (B\(build))", action: nil, keyEquivalent: "")
    }
    statusBarMenu.addItem(
      withTitle: "Check for Updates...",
      action: #selector(AppDelegate.checkForUpdates),
      keyEquivalent: "")
    let integrations = statusBarMenu.addItem(
      withTitle: "Integrations",
      action: nil,
      keyEquivalent: "")
    integrations.submenu = integrationsMenu()

    statusBarMenu.addItem(NSMenuItem.separator())
    let issue = statusBarMenu.addItem(
      withTitle: "Report a bug...", // ✉️
      action: #selector(AppDelegate.sendFeedback),
      keyEquivalent: "")
    issue.image = NSImage(imageLiteralResourceName: "github")
    statusBarMenu.addItem(NSMenuItem.separator())

    let invite = statusBarMenu.addItem(
      withTitle: "Invite a friend...",
      action: #selector(AppDelegate.inviteAFriend),
      keyEquivalent: "")
    invite.image = NSImage(named: NSImage.Name("invite"))

    statusBarMenu.addItem(NSMenuItem.separator())
    statusBarMenu.addItem(
      withTitle: "Restart",
      action: #selector(AppDelegate.restart),
      keyEquivalent: "")
    statusBarMenu.addItem(
      withTitle: "Quit Fig",
      action: #selector(AppDelegate.quit),
      keyEquivalent: "")

    if !Defaults.shared.isProduction || Defaults.shared.beta {
      statusBarMenu.addItem(NSMenuItem.separator())
      statusBarMenu.addItem(
        withTitle: "\(Defaults.shared.beta ? "[Beta] ":"")\(Defaults.shared.build.rawValue)",
        action: nil,
        keyEquivalent: "")
    }

    if let devMode = Settings.shared.getValue(forKey: Settings.developerModeKey) as? Bool, devMode {
      statusBarMenu.addItem(NSMenuItem.separator())
      let devMode = statusBarMenu.addItem(
        withTitle: "Developer mode",
        action: #selector(toggleDeveloperMode),
        keyEquivalent: ""
      )
      devMode.state = .on
      devMode.indentationLevel = 1

    } else if let devModeCLI = Settings.shared.getValue(forKey: Settings.developerModeNPMKey) as? Bool, devModeCLI {
      statusBarMenu.addItem(NSMenuItem.separator())
      let devMode = statusBarMenu.addItem(
        withTitle: "Developer mode",
        action: #selector(toggleDeveloperMode),
        keyEquivalent: ""
      )
      devMode.state = .on
      devMode.indentationLevel = 1

    }

    return statusBarMenu
  }

  func configureStatusBarItem() {
    guard self.statusBarItem != nil else {
      return
    }

    guard Defaults.shared.loggedIn else {
      self.statusBarItem.menu = self.onboardingStatusBarMenu()
      self.statusBarItem.menu?.delegate = self
      return
    }

    if Accessibility.enabled {
      DispatchQueue.main.async {
        self.statusBarItem.button?.layer?.removeAnimation(forKey: "spring")
      }

      self.statusBarItem.menu = self.defaultStatusBarMenu()

    } else {
      DispatchQueue.main.async {
        let spring = CASpringAnimation(keyPath: "position.y")
        spring.initialVelocity = -100
        spring.damping = 5
        spring.mass = 0.5
        spring.fromValue = 1
        spring.toValue = 0
        spring.repeatCount = .greatestFiniteMagnitude
        spring.duration = spring.settlingDuration + 1.5

        self.statusBarItem.button?.layer?.add(spring, forKey: "spring")
      }

      self.statusBarItem.menu = self.alertStatusBarMenu()

    }

    self.statusBarItem.menu?.delegate = self

  }

  @objc func settingsUpdated() {
    print("Settings updated!!!!")
    if let hideMenuBar = Settings.shared.getValue(forKey: Settings.hideMenubarIcon) as? Bool {
      if hideMenuBar {
        self.statusBarItem = nil
      } else {
        let statusBar = NSStatusBar.system
        statusBarItem = statusBar.statusItem(
          withLength: NSStatusItem.squareLength)
        statusBarItem.button?.title = "🍐"
        statusBarItem.button?.image = NSImage(imageLiteralResourceName: "statusbar@2x.png")// .overlayBadge()
        statusBarItem.button?.image?.isTemplate = true
        statusBarItem.button?.wantsLayer = true

      }
    }

    configureStatusBarItem()
  }

  @objc func integrationsUpdated() {
    configureStatusBarItem()
  }

  func setUpAccesibilityObserver() {

    let center = DistributedNotificationCenter.default()
    let accessibilityChangedNotification = NSNotification.Name("com.apple.accessibility.api")
    center.addObserver(forName: accessibilityChangedNotification, object: nil, queue: nil) { _ in
      print("Accessibility status changed!")
      DispatchQueue.global(qos: .userInitiated).asyncAfter(deadline: .now() + 0.1) {
        print("Start configuring startbar item")
        self.configureStatusBarItem()
        print("Done configuring startbar item")

      }
    }
  }

  func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
    return .terminateNow
  }

  @objc func flushLogs() {
    TelemetryProvider.shared.flushAll(includingCurrentDay: true)
  }

  @objc func newTerminalWindow() {
    WindowManager.shared.newNativeTerminalSession()
  }

  @objc func openSettingsDocs() {
    NSWorkspace.shared.open(URL(string: "https://fig.io/docs/support/settings")!)
  }

  @objc func uninstall() {

    let confirmed = self.dialogOKCancel(
      question: "Uninstall Fig?",
      text: "You will need to restart any currently running terminal sessions.",
      icon: NSImage(imageLiteralResourceName: NSImage.applicationIconName)
    )

    if confirmed {
      TelemetryProvider.shared.track(event: .uninstallApp, with: [:])

      ShellHookManager.shared.ttys().forEach { (pair) in
        let (_, tty) = pair
        tty.setTitle("Restart this terminal to finish uninstalling Fig...")
      }

      var uninstallScriptFile: String? = "\(NSHomeDirectory())/.fig/tools/uninstall-script.sh"
      if !FileManager.default.fileExists(atPath: uninstallScriptFile!) {
        uninstallScriptFile = Bundle.main.path(forResource: "uninstall", ofType: "sh")
      }

      if let general = uninstallScriptFile {
        NSWorkspace.shared.open(
          URL(string: "https://fig.io/uninstall?email=\(Defaults.shared.email ?? "")&" +
            "version=\(Diagnostic.distribution.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? "")")!)
        LoginItems.shared.removeAllItemsMatchingBundleURL()

        let domain = Bundle.main.bundleIdentifier!
        let uuid = Defaults.shared.uuid
        UserDefaults.standard.removePersistentDomain(forName: domain)
        UserDefaults.standard.removePersistentDomain(forName: "\(domain).shared")

        UserDefaults.standard.synchronize()

        UserDefaults.standard.set(uuid, forKey: "uuid")
        UserDefaults.standard.synchronize()

        WebView.deleteCache()
        InputMethod.default.uninstall()

        let out = "bash \(general)".runAsCommand()
        Logger.log(message: out)
        self.quit()
      }
    }
  }

  @objc func sendFeedback() {
    //        NSWorkspace.shared.open(URL(string:"mailto:hello@withfig.com")!)

    Github.openIssue()
    TelemetryProvider.shared.track(event: .sendFeedback, with: [:])
  }

  @objc func setupScript() {
    TelemetryProvider.shared.track(event: .runInstallationScript, with: [:])
    Onboarding.setUpEnviroment()
  }

  @objc func toggleiTermIntegration(_ sender: NSMenuItem) {
    iTermIntegration.default.promptToInstall { _ in
      sender.state = iTermIntegration.default.isInstalled ? .on : .off
    }
  }

  func dialogOKCancel(
    question: String,
    text: String,
    prompt: String = "OK",
    noAction: Bool = false,
    icon: NSImage? = nil,
    noActionTitle: String? = nil
  ) -> Bool {
    let alert = NSAlert() // NSImage.cautionName
    alert.icon = icon ?? NSImage(imageLiteralResourceName: "NSSecurity").overlayAppIcon()
    alert.icon.size = NSSize(width: 32, height: 32)
    alert.messageText = question
    alert.informativeText = text
    alert.alertStyle = .warning
    let button = alert.addButton(withTitle: prompt)
    button.highlight(true)
    if !noAction {
      alert.addButton(withTitle: noActionTitle ?? "Not now")
    }
    return alert.runModal() == .alertFirstButtonReturn
  }

  func handleUpdateIfNeeded() {
    Logger.log(message: "Checking if app has updated...")
    guard let previous = Defaults.shared.versionAtPreviousLaunch else {
      let updatedVersionString = Bundle.main.infoDictionary?["CFBundleShortVersionString"]
      Defaults.shared.versionAtPreviousLaunch = updatedVersionString as? String
      print("Update: First launch!")
      Logger.log(message: "First launch!")
      TelemetryProvider.shared.track(event: .firstTimeUser, with: [:])
      Onboarding.setUpEnviroment()
      return
    }

    guard let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String,
          let build = Bundle.main.infoDictionary?["CFBundleVersion"] as? String else {
      print("Update: No version detected.")
      return
    }

    let current = version + "," + build

    // upgrade path!
    if previous != current {

      Onboarding.setUpEnviroment()

      TelemetryProvider.shared.track(event: .updatedApp, with: [:])

      // resolves a bug where Fig was added to login items multiple times
      // if the appropriate setting is enabled, a single entry will be readded
      LoginItems.shared.removeAllItemsMatchingBundleURL()
    }

    Defaults.shared.versionAtPreviousLaunch = current

  }

  @objc func restart() {
    Defaults.shared.launchedFollowingCrash = false
    Logger.log(message: "Restarting Fig...")
    let url = URL(fileURLWithPath: Bundle.main.resourcePath!)
    let path = url.deletingLastPathComponent().deletingLastPathComponent().absoluteString
    let task = Process()
    task.launchPath = "/usr/bin/open"
    task.arguments = [path]
    task.launch()
    exit(0)
  }

  func setupCompanionWindow() {
    Logger.log(message: "Setting up companion windows")
    Defaults.shared.loggedIn = true

    Logger.log(message: "Configuring status bar")
    self.configureStatusBarItem()

    Logger.log(message: "Creating windows...")
    WindowManager.shared.createAutocomplete()

  }

  @objc func inviteToSlack() {
    NSWorkspace.shared.open(URL(string: "https://fig-core-backend.herokuapp.com/community")!)
    TelemetryProvider.shared.track(event: .joinSlack, with: [:])

  }

  @objc func inviteAFriend() {
    guard let email = Defaults.shared.email else {
      Alert.show(
        title: "You are not logged in!",
        message: "Run `fig util:logout` and try again.",
        icon: Alert.appIcon
      )
      return
    }

    TelemetryProvider.shared.track(event: .inviteAFriend, with: [:])

    let request = URLRequest(
      url: Remote.API.appendingPathComponent("/waitlist/get-referral-link-from-email/\(email)")
    )
    let task = URLSession.shared.dataTask(with: request) { (data, _, _) in
      DispatchQueue.main.async {

        guard let data = data else {
          Alert.show(title: "Could not retrieve referral link!",
                     message: "Please contact hello@fig.io and we will get you a working referral link.",
                     icon: Alert.appIcon)
          return
        }

        let link = String(decoding: data, as: UTF8.self)

        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(link, forType: .string)

        let openInBrowser = Alert.show(title: "Thank you for sharing Fig!",
                                       message: "Your invite link has been copied to your clipboard!\n\n\(link)",
                                       okText: "Open in browser...",
                                       icon: Alert.appIcon,
                                       hasSecondaryOption: true)

        if openInBrowser, let url = URL(string: link) {
          NSWorkspace.shared.open(url)
        }
      }
    }

    task.resume()
  }

  @objc func viewDocs() {
    NSWorkspace.shared.open(URL(string: "https://fig.io/docs")!)
    TelemetryProvider.shared.track(event: .viewDocs, with: [:])
  }

  @objc func viewSupportForum() {
    NSWorkspace.shared.open(URL(string: "https://fig.io/support")!)
    TelemetryProvider.shared.track(event: .viewSupportForum, with: [:])
  }

  @objc func getKeyboardLayout() {
    guard let vKey = KeyboardLayout.shared.keyCode(for: "V"),
          let eKey = KeyboardLayout.shared.keyCode(for: "E"),
          let uKey = KeyboardLayout.shared.keyCode(for: "U") else {
      return
    }
    print("v=\(vKey); e=\(eKey); u=\(uKey)")
  }

  @objc func toggleAutocomplete(_ sender: NSMenuItem) {
    Defaults.shared.useAutocomplete = !Defaults.shared.useAutocomplete
    sender.state = Defaults.shared.useAutocomplete ? .on : .off
    TelemetryProvider.shared.track(
      event: .toggledAutocomplete,
      with: ["status": Defaults.shared.useAutocomplete ? "on" : "off"]
    )

    if Defaults.shared.useAutocomplete {
      WindowManager.shared.createAutocomplete()

      KeypressProvider.shared.registerKeystrokeHandler()
      AXWindowServer.shared.registerWindowTracking()
    }

    Logger.log(message: "Toggle autocomplete \(Defaults.shared.useAutocomplete ? "on" : "off")")
  }

  @objc func getSelectedText() {
    NSEvent.addGlobalMonitorForEvents(matching: .keyUp) { (event) in
      print("keylogger:", event.characters ?? "", event.keyCode)
      let systemWideElement = AXUIElementCreateSystemWide()
      var focusedElement: AnyObject?

      let error = AXUIElementCopyAttributeValue(
        systemWideElement,
        kAXFocusedUIElementAttribute as CFString,
        &focusedElement
      )
      if error != .success {
        print("Couldn't get the focused element. Probably a webkit application")
      } else {
        var names: CFArray?
        // swiftlint:disable:next force_cast
        _ = AXUIElementCopyAttributeNames(focusedElement as! AXUIElement, &names)
        print(names as Any)

        var parametrizedNames: CFArray?
        // swiftlint:disable:next force_cast
        _ = AXUIElementCopyParameterizedAttributeNames(focusedElement as! AXUIElement, &parametrizedNames)
        print(parametrizedNames as Any)
      }
    }
  }
  @objc func newAccesibilityAPI() {}
  var observer: AXObserver?

  @objc func addAccessibilityObserver() {
    let first = WindowServer.shared.topmostWindow(for: NSWorkspace.shared.frontmostApplication!)!
    print(first.bundleId ?? "?")
    let axErr = AXObserverCreate(
      first.app.processIdentifier,
      // swiftlint:disable:next line_length
      { (observer: AXObserver, element: AXUIElement, notificationName: CFString, refcon: UnsafeMutableRawPointer?) -> Void in
        print("axobserver:", notificationName)
        print("axobserver:", element)
        print("axobserver:", observer)
        print("axobserver:", refcon as Any)
      },
      &observer
    )

    // kAXWindowMovedNotification
    let focusedWindowChangedObserver = AXObserverAddNotification(
      observer!,
      AXUIElementCreateApplication(first.app.processIdentifier),
      kAXFocusedWindowChangedNotification as CFString,
      nil
    )
    print("axobserver:", focusedWindowChangedObserver)
    let mainWindowChangedObserver = AXObserverAddNotification(
      observer!,
      AXUIElementCreateApplication(first.app.processIdentifier),
      kAXMainWindowChangedNotification as CFString,
      nil
    )
    print("axobserver:", mainWindowChangedObserver)
    func addObserver(notification: CFString) {
      AXObserverAddNotification(
        observer!,
        AXUIElementCreateApplication(first.app.processIdentifier),
        notification,
        nil
      )
    }

    let notificationTypes = [
      kAXWindowMiniaturizedNotification,
      kAXWindowDeminiaturizedNotification,
      kAXWindowCreatedNotification,
      kAXWindowCreatedNotification,
      kAXApplicationShownNotification,
      kAXApplicationHiddenNotification,
      kAXApplicationActivatedNotification,
      kAXApplicationDeactivatedNotification
    ]

    for notificationType in notificationTypes {
      AXObserverAddNotification(
        observer!,
        AXUIElementCreateApplication(first.app.processIdentifier),
        notificationType as CFString,
        nil
      )
    }

    AXObserverAddNotification(observer!, first.accesibilityElement!, kAXWindowMovedNotification as CFString, nil)
    AXObserverAddNotification(observer!, first.accesibilityElement!, kAXWindowResizedNotification as CFString, nil)

    print(axErr)
    print(observer as Any)
    CFRunLoopAddSource(CFRunLoopGetCurrent(), AXObserverGetRunLoopSource(observer!), CFRunLoopMode.defaultMode)
  }

  @objc func toggleOnlyTab(_ sender: NSMenuItem) {
    Defaults.shared.onlyInsertOnTab = !Defaults.shared.onlyInsertOnTab
    sender.state = Defaults.shared.onlyInsertOnTab ? .on : .off
  }

  @objc func toggleSidebar(_ sender: NSMenuItem) {
    //         if let companion = self.window as? CompanionWindow,
    //            let vc = companion.contentViewController as? WebViewController,
    //            let webView = vc.webView {
    //            companion.positioning = .icon
    //            webView.loadRemoteApp(at: Remote.baseURL.appendingPathComponent("hide"))
    //
    //        }

    Defaults.shared.showSidebar = !Defaults.shared.showSidebar
    sender.state = Defaults.shared.showSidebar ? .on : .off
    WindowManager.shared.requestWindowUpdate()

    TelemetryProvider.shared.track(
      event: .toggledSidebar,
      with: ["status": Defaults.shared.useAutocomplete ? "on" : "off"]
    )
  }

  @objc func toggleLogging(_ sender: NSMenuItem) {
    Defaults.shared.broadcastLogs = !Defaults.shared.broadcastLogs
    sender.state = Defaults.shared.broadcastLogs ? .on : .off
  }

  @objc func toggleFigIndicator(_ sender: NSMenuItem) {

  }

  @objc func toggleVSCodeIntegration(_ sender: NSMenuItem) {
    VSCodeIntegration.default.promptToInstall { _ in
      sender.state = VSCodeIntegration.default.isInstalled ? .on : .off
    }
  }

  @objc func toggleHyperIntegration(_ sender: NSMenuItem) {
    HyperIntegration.default.promptToInstall { _ in
      sender.state = HyperIntegration.default.isInstalled ? .on : .off
    }
  }

  @objc func toggleDebugAutocomplete(_ sender: NSMenuItem) {
    Defaults.shared.debugAutocomplete = !Defaults.shared.debugAutocomplete
    sender.state = Defaults.shared.debugAutocomplete ? .on : .off

    if !Defaults.shared.debugAutocomplete {
      WindowManager.shared.autocomplete?.maxHeight = 0
    }
  }

  @objc func pid() {
    if let window = WindowServer.shared.topmostAllowlistedWindow() {
      print("\(window.bundleId ?? "") -  pid:\(window.app.processIdentifier) - \(window.windowId)")
    }
  }

  @objc func checkForUpdates() {
    self.updater.checkForUpdates(self)
  }

  @objc func toggleVisibility() {
    if let window = self.window {
      // swiftlint:disable:next force_cast
      let companion = window as! CompanionWindow
      let position = companion.positioning

      if NSWorkspace.shared.frontmostApplication?.isFig ?? false {
        ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
      }

      if position == CompanionWindow.defaultPassivePosition {
        companion.positioning = CompanionWindow.defaultActivePosition
        NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
      } else {
        companion.positioning = CompanionWindow.defaultPassivePosition
      }
    }
  }

  @available(macOS, deprecated: 10.11)
  @objc func applicationIsInStartUpItems() -> Bool {
    return LaunchAgent.launchOnStartup.enabled()
  }

  @objc func quit() {

    if let statusbar = self.statusBarItem {
      NSStatusBar.system.removeStatusItem(statusbar)
    }

    Config.shared.set(value: "1", forKey: Config.userExplictlyQuitApp)

    TelemetryProvider.shared.track(event: .quitApp, with: [:]) { (_, _, _) in
      DispatchQueue.main.async {
        NSApp.terminate(self)
      }
    }

  }

  @objc func toggleDeveloperMode() {
    Defaults.shared.toggleDeveloperMode()
  }

  @objc func promptForAccesibilityAccess() {
    Accessibility.promptForPermission()
  }

  @objc func addCLI() {
    ShellBridge.symlinkCLI()
  }

  @objc func killSocketServer() {
    ShellBridge.shared.stopWebSocketServer()
  }

  @objc func startSocketServer() {
    ShellBridge.shared.startWebSocketServer()
  }

  @objc func spaceChanged() {
    print("spaceChanged!")
  }

  @objc func newActiveApp() {
    print("newActiveApp!")
  }

  func applicationWillTerminate(_ aNotification: Notification) {
    ShellBridge.shared.stopWebSocketServer()
    Defaults.shared.launchedFollowingCrash = false
    PseudoTerminal.shared.dispose()
    InputMethod.default.terminate()

    // Ensure that fig.socket is deleted, so that if user switches acounts it can be recreated
    try? FileManager.default.removeItem(atPath: "/tmp/fig.socket")

    Logger.log(message: "app will terminate...")
  }

  // > fig search
  @objc func getTopTerminalWindow() {
    guard let app = NSWorkspace.shared.frontmostApplication else {
      return
    }

    if app.bundleIdentifier == "com.googlecode.iterm2" {
      let appRef = AXUIElementCreateApplication(app.processIdentifier)

      var window: AnyObject?
      let result = AXUIElementCopyAttributeValue(appRef, kAXFocusedWindowAttribute as CFString, &window)
      // add error handling

      if result == .apiDisabled {
        print("Accesibility needs to be enabled.")
        return
      }

      print(window ?? "<none>" )

      var position: AnyObject?
      var size: AnyObject?

      let result2 = AXUIElementCopyAttributeValue(
        // swiftlint:disable:next force_cast
        window as! AXUIElement,
        kAXPositionAttribute as CFString,
        &position
      )

      // swiftlint:disable:next force_cast
      AXUIElementCopyAttributeValue(window as! AXUIElement, kAXSizeAttribute as CFString, &size)

      switch result2 {
      case .parameterizedAttributeUnsupported:
        print("parameterizedAttributeUnsupported")
      case .success:
        print("success")

      case .failure:
        print("error")

      case .illegalArgument:
        print("error")

      case .invalidUIElement:
        print("error")

      case .invalidUIElementObserver:
        print("error")

      case .cannotComplete:
        print("error")

      case .attributeUnsupported:
        print("error")

      case .actionUnsupported:
        print("error")

      case .notificationUnsupported:
        print("error")
      case .notImplemented:
        print("error")

      case .notificationAlreadyRegistered:
        print("error")

      case .notificationNotRegistered:
        print("error")

      case .apiDisabled:
        print("error")

      case .noValue:
        print("error")

      case .notEnoughPrecision:
        print("error")

      @unknown default:
        print("error")

      }

      if let position = position, let size = size {
        // swiftlint:disable:next force_cast
        let point = AXValueGetters.asCGPoint(value: position as! AXValue)
        // swiftlint:disable:next force_cast
        let bounds = AXValueGetters.asCGSize(value: size as! AXValue)
        print(point, bounds)

        let titleBarHeight: CGFloat = 23.0

        let includeTitleBarHeight = false

        let terminalWindowFrame = NSRect.init(
          x: point.x,
          y: (NSScreen.main?.visibleFrame.height)! - point.y + ((includeTitleBarHeight) ? titleBarHeight : 0),
          width: bounds.width,
          height: bounds.height - ((includeTitleBarHeight) ? 0 : titleBarHeight)
        )
        print(terminalWindowFrame)
        self.window.windowController?.shouldCascadeWindows = false
        self.clicks += 1

        print(self.window.frame)
      }
    }
  }

  @objc func processes() {
    printProcesses("")
    var size: Int32 = 0
    if let ptr = getProcessInfo("", &size) {
      let buffer = UnsafeMutableBufferPointer<fig_proc_info>(start: ptr, count: Int(size))

      buffer.forEach { (process) in
        var proc = process

        let cwd = withUnsafeBytes(of: &proc.cwd) { (rawPtr) -> String in
          let ptr = rawPtr.baseAddress!.assumingMemoryBound(to: CChar.self)
          return String(cString: ptr)
        }

        let cmd = withUnsafeBytes(of: &proc.cmd) { (rawPtr) -> String in
          let ptr = rawPtr.baseAddress!.assumingMemoryBound(to: CChar.self)
          return String(cString: ptr)
        }

        let tty = withUnsafeBytes(of: &proc.tty) { (rawPtr) -> String in
          let ptr = rawPtr.baseAddress!.assumingMemoryBound(to: CChar.self)
          return String(cString: ptr)
        }

        print("proc: ", proc.pid, cwd, cmd, tty)
      }
      free(ptr)
    }
  }

  @objc func allWindows() {
    Timer.delayWithSeconds(3) {
      guard let json = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[String: Any]] else {
        return
      }

      let infos = json.compactMap({ WindowInfo(json: $0) })
      print(infos)

      print(infos.filter({
        return NSRunningApplication(processIdentifier: pid_t($0.pid))?.bundleIdentifier == "com.apple.Spotlight"
      }))
    }
  }

  @objc func pasteStringToTerminal() {
    let terminals = NSRunningApplication.runningApplications(withBundleIdentifier: "com.googlecode.iterm2")
    if let activeTerminal = terminals.first {
      activeTerminal.activate(options: NSApplication.ActivationOptions.init())
      print("Simulate paste for process: \(activeTerminal.processIdentifier)")
      ShellBridge.simulate(keypress: .v, pid: activeTerminal.processIdentifier, maskCommand: true)
    }
  }

  func injectStringIntoTerminal(_ cmd: String) {
    if let currentApp = NSWorkspace.shared.frontmostApplication {
      if currentApp.bundleIdentifier == "com.googlecode.iterm2" {
        // save current pasteboard
        let pasteboard = NSPasteboard.general
        let copiedString = pasteboard.string(forType: .string) ?? ""

        // add our script to pasteboard
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(cmd, forType: .string)
        print(pasteboard.string(forType: .string) ?? "")
        ShellBridge.simulate(keypress: .v, maskCommand: true)
        ShellBridge.simulate(keypress: .rightArrow)
        ShellBridge.simulate(keypress: .enter)

        // need delay so that terminal responds
        Timer.delayWithSeconds(1) {
          // restore pasteboard
          NSPasteboard.general.clearContents()
          pasteboard.setString(copiedString, forType: .string)
        }
      }
    }
  }

  @objc func sendStringIfTerminalActive() {
    injectStringIntoTerminal("echo \"hello world\"")
  }
}

private func delayWithSeconds(
  _ seconds: Double,
  completion: @escaping () -> Void
) {
  DispatchQueue.main.asyncAfter(deadline: .now() + seconds) {
    completion()
  }
}

func getDocumentsDirectory() -> URL {
  return URL(fileURLWithPath: NSSearchPathForDirectoriesInDomains(.documentDirectory, .userDomainMask, true)[0])
}

struct WindowInfo {
  let frame: CGRect
  let name: String
  let pid: Int
  let number: Int
  let visible: Bool

  init?(json: [String: Any]) {
    guard let pid = json["kCGWindowOwnerPID"] as? Int else {
      return nil
    }

    guard let name = json["kCGWindowOwnerName"] as? String else {
      return nil
    }

    guard let onScreen = json["kCGWindowIsOnscreen"] as? Bool else {
      return nil
    }

    guard let rect = json["kCGWindowBounds"] as? [String: Any] else {
      return nil
    }

    // swiftlint:disable:next identifier_name
    guard let x = rect["X"] as? CGFloat else {
      return nil
    }

    // swiftlint:disable:next identifier_name
    guard let y = rect["Y"] as? CGFloat else {
      return nil
    }

    guard let height = rect["Height"] as? CGFloat else {
      return nil
    }

    guard let width = rect["Width"] as? CGFloat else {
      return nil
    }

    guard let number = json["kCGWindowNumber"] as? Int else {
      return nil
    }

    self.pid = pid
    self.name = name
    self.number = number
    self.frame = CGRect(x: x, y: y, width: width, height: height)
    self.visible = onScreen
  }
}

class AXValueGetters {

  class func asCGRect(value: AXValue) -> CGRect {
    var val = CGRect.zero
    AXValueGetValue(value, AXValueType.cgRect, &val)
    return val
  }

  class func asCGPoint(value: AXValue) -> CGPoint {
    var val = CGPoint.zero
    AXValueGetValue(value, AXValueType.cgPoint, &val)
    return val
  }

  class func asCFRange(value: AXValue) -> CFRange {
    var val = CFRange(location: 0, length: 0)
    AXValueGetValue(value, AXValueType.cfRange, &val)
    return val
  }

  class func asCGSize(value: AXValue) -> CGSize {
    var val = CGSize.zero
    AXValueGetValue(value, AXValueType.cgSize, &val)
    return val
  }

}

extension AppDelegate: SUUpdaterDelegate {
  func updater(_ updater: SUUpdater, didAbortWithError error: Error) {

  }

  func updaterDidNotFindUpdate(_ updater: SUUpdater) {

  }

  func updater(_ updater: SUUpdater, didFindValidUpdate item: SUAppcastItem) {
    print("Found valid update")
  }

  func updater(_ updater: SUUpdater, didFinishLoading appcast: SUAppcast) {

  }
}

extension AppDelegate: NSMenuDelegate {
  func menuDidClose(_ menu: NSMenu) {
    print("menuDidClose")
  }

  @objc func windowDidChange(_ notification: Notification) {
  }

  @objc func resetWindowTracking() {
    if let app = NSWorkspace.shared.frontmostApplication {
      AXWindowServer.shared.register(app, fromActivation: false)
    }
  }

  func stringArrayToMenu(items: [String]) -> [NSMenuItem] {
    var legendItems: [NSMenuItem] = []
    items.forEach { (str) in
      if str == "---" {
        legendItems.append(NSMenuItem.separator())
      } else {
        legendItems.append(NSMenuItem(title: str, action: nil, keyEquivalent: ""))
      }
    }
    return legendItems
  }

  func getSubmenu(window: ExternalWindow?, app: NSRunningApplication) -> (NSColor, [NSMenuItem]) {
    let companionWindow = WindowManager.shared.autocomplete
    if let (message, hexString, shouldDisplay) = companionWindow?.status, shouldDisplay {
      let color: NSColor = hexString != nil ? (NSColor(hex: hexString!) ?? .red) : .red
      return (color, stringArrayToMenu(items: message.split(separator: "\n").map { String($0) }))
    }

    if !Integrations.bundleIsValidTerminal(window?.bundleId) {
      let items = stringArrayToMenu(items: [
        "Not tracking window...",
        "---",
        "Switch to a different application",
        "and then return to current window"
      ])
      return (NSColor.clear, items)
    }

    if let isLoading = companionWindow?.webView?.isLoading, isLoading {
      return (NSColor.yellow, [
        NSMenuItem(title: "Autocomplete is loading", action: nil, keyEquivalent: ""),
        NSMenuItem.separator(),
        NSMenuItem(title: "Make sure you're connected to", action: nil, keyEquivalent: ""),
        NSMenuItem(title: "the internet and try again.", action: nil, keyEquivalent: ""),
        NSMenuItem.separator(),
        NSMenuItem(title: "Reload Autocomplete", action: #selector(restart), keyEquivalent: "")
      ])
    }

    guard let windowSafe = window else {
      return (NSColor.red, [
        NSMenuItem(title: "Window is not being tracked.", action: nil, keyEquivalent: ""),
        NSMenuItem.separator(),
        NSMenuItem(title: "Reset Window Tracking", action: #selector(resetWindowTracking), keyEquivalent: ""),
        NSMenuItem(title: "Restart Fig", action: #selector(restart), keyEquivalent: "")
      ])
    }

    if !Diagnostic.installationScriptRan {
      return (NSColor.red, [
        NSMenuItem(title: "~/.fig directory is misconfigured", action: nil, keyEquivalent: ""),
        NSMenuItem.separator(),
        NSMenuItem(title: "Re-run Install Script", action: #selector(setupScript), keyEquivalent: "")
      ])
    }

    if (SecureKeyboardInput.wasEnabled && windowSafe.bundleId == Integrations.Terminal) ||
        SecureKeyboardInput.enabled {
      // Also check previous value (wasEnabled) because clicking on menubar icon will disable secure keyboard
      // input in Terminal.app
      let color = NSColor.systemPink
      var legendItems = [
        NSMenuItem(title: "'Secure Keyboard Input' Enabled", action: nil, keyEquivalent: ""),
        NSMenuItem.separator(),
        NSMenuItem(title: "This prevents Fig from", action: nil, keyEquivalent: ""),
        NSMenuItem(title: "processing keypress events. ", action: nil, keyEquivalent: ""),
        NSMenuItem.separator()
      ]

      let app = SecureKeyboardInput.responsibleApplication ?? app
      let pid = SecureKeyboardInput.responsibleProcessId ?? app.processIdentifier
      if SecureKeyboardInput.enabled(by: windowSafe.bundleId),
         let name = app.localizedName {
        let open = NSMenuItem(
          title: "Disable in '\(name)' (\(pid)).",
          action: #selector(SecureKeyboardInput.openRelevantMenu),
          keyEquivalent: ""
        )
        open.target = SecureKeyboardInput.self
        legendItems.append(open)

      } else {
        // Run `ioreg -l -w 0 | grep SecureInput` to determine which app is responsible.
        let lock = NSMenuItem(
          title: "Lock screen and log back in",
          action: #selector(SecureKeyboardInput.lockscreen),
          keyEquivalent: ""
        )
        lock.target = SecureKeyboardInput.self
        legendItems.append(lock)
      }

      legendItems.append(NSMenuItem.separator())
      let support = NSMenuItem(
        title: "Learn more",
        action: #selector(SecureKeyboardInput.openSupportPage),
        keyEquivalent: ""
      )
      support.target = SecureKeyboardInput.self
      legendItems.append(support)
      return (color, legendItems)
    }

    guard Diagnostic.unixSocketServerExists else {

      return (NSColor.red, [
        NSMenuItem(title: "Unix socket does not exist", action: nil, keyEquivalent: ""),
        NSMenuItem.separator(),
        NSMenuItem(title: "This prevents Fig from", action: nil, keyEquivalent: ""),
        NSMenuItem(title: "recieving data from the shell.", action: nil, keyEquivalent: ""),
        NSMenuItem.separator(),
        NSMenuItem(title: "Restart Fig", action: #selector(restart), keyEquivalent: "")
      ])
    }

    guard let shellContext = windowSafe.associatedShellContext else {

      let items = stringArrayToMenu(items: [
        "Not linked to terminal session.",
        "---",
        "window: \(windowSafe.hash)",
        "---",
        "Run `fig doctor` for more details."

      ])
      return (NSColor.yellow, items)
    }
    guard shellContext.isShell() else {
      let items = stringArrayToMenu(items: [
        "Running proccess (\(shellContext.executablePath)) is not a shell.",
        "---",
        "Fix: exit current process",
        "---",
        "window: \(windowSafe.hash)"
      ])
      return (NSColor.cyan, items)
    }

    var figtermSocketExists: Bool = false

    if let session = window?.session {
      let path = FigTerm.path(for: session)
      figtermSocketExists = FileManager.default.fileExists(atPath: path)

      guard figtermSocketExists else {
        let items = stringArrayToMenu(items: [
          "Inserting text into the terminal will fail.",
          "---",
          "figterm socket does not exist for session:",
          "\(session)"
        ])
        return (NSColor.yellow, items)
      }

    }

    if let (color, layout) = Diagnostic.debuggerStatusFromWeb {
      let items = stringArrayToMenu(items: layout)
      return (color, items)
    }

    let path = Diagnostic.pseudoTerminalPathAppearsValid

    let items = stringArrayToMenu(items: [
      "Everything seems to be working.",
      "---",
      "window: \(windowSafe.hash.truncate(length: 15, trailing: "..."))",
      "tty: \(shellContext.ttyDescriptor)",
      "cwd: \(shellContext.workingDirectory)",
      "pid: \(shellContext.processId)",
      "keybuffer: \(windowSafe.associatedEditBuffer?.representation ?? "???")",
      "path: \( path != nil ? (path! ? "☑" : "☒ ") : "<generated dynamically>")",
      "---",
      "Run `fig doctor` to perform",
      "additional debugging checks."
    ])
    return (NSColor.green, items)
  }

  func menuWillOpen(_ menu: NSMenu) {
    print("menuWillOpen")
    DispatchQueue.global(qos: .background).async {
      TelemetryProvider.shared.track(event: .openedFigMenuIcon, with: [:])
    }
    guard Defaults.shared.loggedIn, Accessibility.enabled else {
      return
    }

    if let frontmost = self.frontmost {
      if menu.items.contains(frontmost) {
        menu.removeItem(frontmost)
      }

      self.frontmost = nil
    }

    if let app = NSWorkspace.shared.frontmostApplication, !app.isFig {
      let window = AXWindowServer.shared.allowlistedWindow
      if Integrations.bundleIsValidTerminal(window?.bundleId) ||
          Integrations.frontmostApplicationIsValidTerminal() {

        let (color, menuItems) = getSubmenu(window: window, app: app)

        let legend = NSMenu(title: "legend")
        menuItems.forEach { (item) in
          legend.addItem(item)
        }

        var image: NSImage?
        if let pid = window?.app.processIdentifier,
           let windowApp = NSRunningApplication(processIdentifier: pid) {
          image = windowApp.icon
        } else {
          image = app.icon
        }

        let cmd = window?.associatedShellContext?.executablePath
        let app = menu.insertItem(
          withTitle: "Debugger (\(cmd ?? "???"))",
          action: nil,
          keyEquivalent: "",
          at: 0
        )

        app.image = image?.resized(to: NSSize(width: 16, height: 16))?.overlayBadge(color: color, text: "")
        app.submenu = legend

        self.frontmost = app
      } else {
        let icon = app.icon?.resized(to: NSSize(width: 16, height: 16))

        let text = Integrations.autocompleteBlocklist.contains(app.bundleIdentifier ?? "")
          ? "has been disabled."
          : "is not supported."

        let item = NSMenuItem(title: text, action: nil, keyEquivalent: "")
        item.image = icon

        menu.insertItem(item, at: 0)
        self.frontmost = item
      }
    }

    if let integrations = self.integrationPrompt {
      if menu.items.contains(integrations) {
        menu.removeItem(integrations)
      }

      self.integrationPrompt = nil
    }

    if !Diagnostic.installationScriptRan {
      let item = NSMenuItem(
        title: "Rerun Install Script",
        action: #selector(AppDelegate.setupScript),
        keyEquivalent: ""
      )
      item.image = NSImage(named: NSImage.Name("alert"))
      menu.insertItem(item, at: 1)
      self.integrationPrompt = item
      return
    }

    if let app = NSWorkspace.shared.frontmostApplication,
       !app.isFig,
       let provider = Integrations.providers[app.bundleIdentifier ?? ""],
       !provider.isInstalled {
      let name: String = provider.applicationName

      let item = NSMenuItem(
        title: "Install \(name) Integration",
        action: #selector(AppDelegate.installIntegrationForFrontmostApp),
        keyEquivalent: ""
      )
      item.image = NSImage(named: NSImage.Name("carrot"))
      menu.insertItem(item, at: 1)
      self.integrationPrompt = item
    }
  }

  @objc func installIntegrationForFrontmostApp() {
    if let app = NSWorkspace.shared.frontmostApplication,
       let provider = Integrations.providers[app.bundleIdentifier ?? ""],
       !provider.isInstalled {
      provider.promptToInstall(completion: nil)
    }
  }
}

extension NSApplication {
  var appDelegate: AppDelegate {
    // swiftlint:disable:next force_cast
    return NSApp.delegate as! AppDelegate
  }
}
