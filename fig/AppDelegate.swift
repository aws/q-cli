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
class AppDelegate: NSObject, NSApplicationDelegate,NSWindowDelegate {

    var window: NSWindow!
    var onboardingWindow: WebViewWindow!
    var statusBarItem: NSStatusItem!
    var frontmost: NSMenuItem?
    var integrationPrompt: NSMenuItem?

    var clicks:Int = 6;
    let updater = UpdateService.provider
    let processPool = WKProcessPool()
    
    let iTermObserver = WindowObserver(with: "com.googlecode.iterm2")
    let TerminalObserver = WindowObserver(with: "com.apple.Terminal")
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
            options.enabled = !Defaults.telemetryDisabled
        }
        warnToMoveToApplicationIfNecessary()
      
        // Set timeout to avoid hanging misbehaving 3rd party apps
//        Accessibility.setGlobalTimeout(seconds: 2)

        if let hideMenuBar = Settings.shared.getValue(forKey: Settings.hideMenubarIcon) as? Bool, hideMenuBar {
          print("Not showing menubarIcon because of \(Settings.hideMenubarIcon)")
        } else {
          let statusBar = NSStatusBar.system
          statusBarItem = statusBar.statusItem(
                 withLength: NSStatusItem.squareLength)
          statusBarItem.button?.title = "🍐"
          statusBarItem.button?.image = NSImage(imageLiteralResourceName: "statusbar@2x.png")//.overlayBadge()
          statusBarItem.button?.image?.isTemplate = true
          statusBarItem.button?.wantsLayer = true
        }
        
        configureStatusBarItem()

//        NSApp.setActivationPolicy(NSApplication.ActivationPolicy.accessory)
        // prevent multiple sessions
        let bundleID = Bundle.main.bundleIdentifier!
        if NSRunningApplication.runningApplications(withBundleIdentifier: bundleID).count > 1 {
            SentrySDK.capture(message: "Multiple Fig instances running!")
            Logger.log(message: "Multiple Fig instances running! Terminating now!")
            NSRunningApplication.runningApplications(withBundleIdentifier: bundleID).filter{ $0.processIdentifier != NSRunningApplication.current.processIdentifier }.forEach { (app) in
                Logger.log(message: "Existing Process Id = \(app.processIdentifier)")
                app.forceTerminate()
            }
//            NSApp.terminate(nil)
        }
        
        TelemetryProvider.track(event: .launchedApp, with:
                                ["crashed" : Defaults.launchedFollowingCrash ? "true" : "false"])
        Defaults.launchedFollowingCrash = true
        Config.set(value: nil, forKey: Config.userExplictlyQuitApp)
        Accessibility.checkIfPermissionRevoked()
      
//        AppMover.moveIfNecessary()
        let _ = ShellBridge.shared
        let _ = WindowManager.shared
        let _ = ShellHookManager.shared
        let _ = KeypressProvider.shared
        let _ = ShellHookTransport.shared
      
        DispatchQueue.global(qos: .userInitiated).async {
          let _ = AXWindowServer.shared
        }
      
        let _ = DockerEventStream.shared
        let _ = iTermIntegration.shared
        let _ = Settings.shared

        
        TelemetryProvider.register()
        Accessibility.listen()
                
//        updater?.checkForUpdateInformation()
//        updater?.delegate = self as SUUpdaterDelegate;
//        updater?.checkForUpdateInformation()
        
//        let domain = Bundle.main.bundleIdentifier!
//        UserDefaults.standard.removePersistentDomain(forName: domain)
//        UserDefaults.standard.synchronize()
//        WebView.deleteCache()

        handleUpdateIfNeeded()
        Defaults.useAutocomplete = true
        Defaults.autocompleteVersion = "v7"
        Defaults.autocompleteWidth = 250
        Defaults.ignoreProcessList = ["figcli", "gitstatusd-darwin-x86_64", "gitstatusd-darwin-arm64", "nc", "fig_pty", "starship", "figterm"]

        let hasLaunched = UserDefaults.standard.bool(forKey: "hasLaunched")
        let email = UserDefaults.standard.string(forKey: "userEmail")

        if (!hasLaunched || email == nil ) {
            Defaults.loggedIn = false
            Defaults.build = .production
            Defaults.clearExistingLineOnTerminalInsert = true
            Defaults.showSidebar = false
          
            Config.set(value: "0", forKey: Config.userLoggedIn)
//            Defaults.defaultActivePosition = .outsideRight
            
            let onboardingViewController = WebViewController()
            onboardingViewController.webView?.defaultURL = nil
            onboardingViewController.webView?.loadBundleApp("landing")
            onboardingViewController.webView?.dragShouldRepositionWindow = true
//            onboardingViewController.webView?.loadRemoteApp(at: URL(string: "https://app.withfig.com/onboarding/landing.html")!)

            onboardingWindow = WebViewWindow(viewController: onboardingViewController)
            onboardingWindow.makeKeyAndOrderFront(nil)
            onboardingWindow.setFrame(NSRect(x: 0, y: 0, width: 590, height: 480), display: true, animate: false)
            onboardingWindow.center()
            onboardingWindow.makeKeyAndOrderFront(self)
            
            UserDefaults.standard.set(true, forKey: "hasLaunched")
            UserDefaults.standard.synchronize()
        } else {
            // identify user for Sentry!
            let user = User()
            user.email = email
            SentrySDK.setUser(user)
            ShellBridge.symlinkCLI()
            Config.set(value: "1", forKey: Config.userLoggedIn)

            
          if (!Accessibility.enabled) {

                SentrySDK.capture(message: "Accesibility Not Enabled on Subsequent Launch")
                let enable = self.dialogOKCancel(question: "Turn on accessibility", text: "To add Fig to your terminal, select the Fig checkbox in Security & Privacy > Accessibility.", prompt: "Turn On Accessibility")
                
                if (enable) {
                    self.promptForAccesibilityAccess()
                }
            }
            let installed = "fig cli:installed".runAsCommand().trimmingCharacters(in: .whitespacesAndNewlines)
            let hasLegacyInstallation = FileManager.default.fileExists(atPath: "/usr/local/bin/fig") && installed != "true"
            let hasNewInstallation = FileManager.default.fileExists(atPath: "\(NSHomeDirectory())/.fig/bin/fig")
            if (!hasLegacyInstallation && !hasNewInstallation) {
                SentrySDK.capture(message: "CLI Tool Not Installed on Subsequent Launch")

                let enable = self.dialogOKCancel(question: "Install Fig CLI Tool?", text: "It looks like you haven't installed the Fig CLI tool. Fig doesn't work without it.")
                              
                  if (enable) {
                      ShellBridge.symlinkCLI()
                  } 
            }
//            updater?.installUpdatesIfAvailable()
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

        
        if let shouldLaunchOnStartup = Settings.shared.getValue(forKey: Settings.launchOnStartupKey) as? Bool {
          LoginItems.shared.currentApplicationShouldLaunchOnStartup = shouldLaunchOnStartup
        } else {
          LoginItems.shared.currentApplicationShouldLaunchOnStartup = true
        }
        
//        iTermTabIntegration.listenForHotKey()
        AutocompleteContextNotifier.listenForUpdates()
        SecureKeyboardInput.listen()
      
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
      
        if !VSCodeIntegration.isInstalled {
            VSCodeIntegration.install(withRestart: false, inBackground: true)
        }

        if !VSCodeInsidersIntegration.isInstalled {
            VSCodeInsidersIntegration.install(withRestart: false, inBackground: true)
        }
      
        if !HyperIntegration.isInstalled {
            HyperIntegration.install(withRestart: false, inBackground: true)
        }
      
        if !iTermIntegration.isInstalled {
            iTermIntegration.install(withRestart: false, inBackground: true)
        }
        
    }
  
    func warnToMoveToApplicationIfNecessary() {
      if Diagnostic.isRunningOnReadOnlyVolume && !Defaults.loggedIn {
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
        let nativeTerminals = NSWorkspace.shared.runningApplications.filter { Integrations.nativeTerminals.contains($0.bundleIdentifier ?? "")}
        
        let count = nativeTerminals.count
        guard count > 0 else { return }
        let iTermOpen = nativeTerminals.contains { $0.bundleIdentifier == "com.googlecode.iterm2" }
        let terminalAppOpen = nativeTerminals.contains { $0.bundleIdentifier == "com.apple.Terminal" }
        
        var emulators: [String] = []
        
        if (iTermOpen) {
            emulators.append("iTerm")
        }
        
        if (terminalAppOpen) {
            emulators.append("Terminal")
        }
                
        let restart = self.dialogOKCancel(question: "Restart existing terminal sessions?", text: "Any terminal sessions started before Fig are not tracked.\n\nRun `fig source` in each session to connect or restart your terminal\(emulators.count == 1 ? "" : "s").\n", prompt: "Restart \(emulators.joined(separator: " and "))", noAction: false, icon: NSImage.init(imageLiteralResourceName: NSImage.applicationIconName))
        
        if (restart) {
            print("restart")
            
            if (iTermOpen) {
                let iTerm = Restarter(with: "com.googlecode.iterm2")
                iTerm.restart()
            }
            
            if (terminalAppOpen) {
                let terminalApp = Restarter(with: "com.apple.Terminal")
                terminalApp.restart()
            }
        }
    }
        
    func openMenu() {
        if let menu = self.statusBarItem.menu {
            self.statusBarItem.popUpMenu(menu)
        }
//        self.statusBarItem.menu?.popUp(positioning: ,
//                                       at: self.statusBarItem.view?.frame.origin,
//                                       in: self.statusBarItem.view)
    }
    
    func validateMenuItem(menuItem: NSMenuItem) -> Bool {
        print("menuitem!!!")
//        if(menuItem.action == Selector("batteryStatus:")) {
//            NSLog("refresh!");
//            let now = NSDate()
//            menuItem.title = String(format:"%f", now.timeIntervalSince1970);
//            return true;
//        }
        return true;
    }
    
    @objc func statusBarButtonClicked(sender: NSStatusBarButton) {
        let event = NSApp.currentEvent!

        if event.type == NSEvent.EventType.leftMouseUp {

            sender.menu = self.defaultStatusBarMenu()
            if let menu = sender.menu {
                menu.popUp(positioning: nil, at: NSPoint(x: 0, y: statusBarItem.statusBar!.thickness), in: sender)
            }
//            sender.menu?.popUp(positioning: sender, at: <#T##NSPoint#>, in: )
//            popover.show(relativeTo: sender.bounds, of: sender, preferredEdge: NSRectEdge.minY)

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
        action:  #selector(AppDelegate.promptForAccesibilityAccess),
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
        action:  #selector(AppDelegate.quit),
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
        action:  #selector(AppDelegate.quit),
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

        // todo(mschrage): Renable when we can set the title using bi-directional IPC with figterm
      if AutocompleteContextNotifier.addIndicatorToTitlebar {
          let statusInTitle = integrationsMenu.addItem(
          withTitle: "Show '☑ fig' in Terminal",
          action: #selector(AppDelegate.toggleFigIndicator(_:)),
          keyEquivalent: "")
          statusInTitle.state = AutocompleteContextNotifier.addIndicatorToTitlebar ? .on : .off
          integrationsMenu.addItem(NSMenuItem.separator())
      }

      let itermIntegration = integrationsMenu.addItem(
      withTitle: "iTerm Integration",
      action: #selector(AppDelegate.toggleiTermIntegration(_:)),
      keyEquivalent: "")
      itermIntegration.state = iTermIntegration.isInstalled ? .on : .off
      
      let vscodeIntegration = integrationsMenu.addItem(
      withTitle: "VSCode Integration",
      action: #selector(AppDelegate.toggleVSCodeIntegration(_:)),
      keyEquivalent: "")
      vscodeIntegration.state = VSCodeIntegration.isInstalled ? .on : .off
    
      let hyperIntegration = integrationsMenu.addItem(
      withTitle: "Hyper Integration",
      action: #selector(AppDelegate.toggleHyperIntegration(_:)),
      keyEquivalent: "")
      hyperIntegration.state = HyperIntegration.isInstalled ? .on : .off
    
      let sshIntegration = integrationsMenu.addItem(
      withTitle: "SSH Integration",
      action: #selector(AppDelegate.toggleSSHIntegration(_:)),
      keyEquivalent: "")
      sshIntegration.state = Defaults.SSHIntegrationEnabled ? .on : .off
      
      integrationsMenu.addItem(NSMenuItem.separator())
      integrationsMenu.addItem(withTitle: "Edit Key Bindings", action: #selector(editKeybindingsFile), keyEquivalent: "")
      
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
      
      let debugAutocomplete = developerMenu.addItem(
       withTitle: "Force Popup to Appear",
       action: #selector(AppDelegate.toggleDebugAutocomplete(_:)),
       keyEquivalent: "")
      debugAutocomplete.state = Defaults.debugAutocomplete ? .on : .off
//        utilitiesMenu.addItem(NSMenuItem.separator())
      developerMenu.addItem(NSMenuItem.separator())
      developerMenu.addItem(
       withTitle: "Run Install/Update Script",
       action: #selector(AppDelegate.setupScript),
       keyEquivalent: "")
      
      if (!Defaults.isProduction) {
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
              action: #selector(AppDelegate.addAccesbilityObserver),
              keyEquivalent: "")
             developerMenu.addItem(
              withTitle: "Get Selected Text",
              action: #selector(AppDelegate.getSelectedText),
              keyEquivalent: "")
             developerMenu.addItem(
               withTitle: "Processes",
               action: #selector(AppDelegate.processes),
               keyEquivalent: "")
             developerMenu.addItem(
               withTitle: "Trigger ScreenReader mode in topmost app",
               action: #selector(AppDelegate.triggerScreenReader),
               keyEquivalent: "")
         }
      
      return developerMenu
    }
    
    func defaultStatusBarMenu() -> NSMenu {
        
        let statusBarMenu = NSMenu(title: "fig")
        statusBarMenu.addItem(NSMenuItem.separator())
        
        let autocomplete = statusBarMenu.addItem(
         withTitle: "Autocomplete", //(βeta)
         action: #selector(AppDelegate.toggleAutocomplete(_:)),
         keyEquivalent: "")
        autocomplete.state = Defaults.useAutocomplete ? .on : .off
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
        slack.image = NSImage(named: NSImage.Name("discord"))//.resized(to: NSSize(width: 16, height: 16))
        statusBarMenu.addItem(NSMenuItem.separator())
        let settings = statusBarMenu.addItem(
         withTitle: "Settings",
         action: #selector(Settings.openUI),
         keyEquivalent: "")
        settings.image = NSImage(imageLiteralResourceName: "gear")
        settings.target = Settings.self
        
        statusBarMenu.addItem(NSMenuItem.separator())

        if let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String, let build = Bundle.main.infoDictionary?["CFBundleVersion"] as? String {
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
         withTitle: "Report a bug...", //✉️
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
        
        if (!Defaults.isProduction || Defaults.beta) {
            statusBarMenu.addItem(NSMenuItem.separator())
            statusBarMenu.addItem(
              withTitle: "\(Defaults.beta ? "[Beta] ":"")\(Defaults.build.rawValue)",
             action: nil,
             keyEquivalent: "")
        }
      
      if let devMode = Settings.shared.getValue(forKey: Settings.developerModeKey) as? Bool, devMode {
        statusBarMenu.addItem(NSMenuItem.separator())
        let devMode = statusBarMenu.addItem(withTitle: "Developer mode", action: #selector(toggleDeveloperMode), keyEquivalent: "")
        devMode.state = .on
        devMode.indentationLevel = 1

      } else if let devModeCLI = Settings.shared.getValue(forKey: Settings.developerModeNPMKey) as? Bool, devModeCLI {
        statusBarMenu.addItem(NSMenuItem.separator())
        let devMode = statusBarMenu.addItem(withTitle: "Developer mode", action: #selector(toggleDeveloperMode), keyEquivalent: "")
        devMode.state = .on
        devMode.indentationLevel = 1


      }

        
        return statusBarMenu
    }
    
    func configureStatusBarItem() {
        guard self.statusBarItem != nil else {
            return
        }
        
        guard Defaults.loggedIn else {
            self.statusBarItem.menu = self.onboardingStatusBarMenu()
            self.statusBarItem.menu?.delegate = self
            return
        }
        
        if (Accessibility.enabled) {
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
          statusBarItem.button?.image = NSImage(imageLiteralResourceName: "statusbar@2x.png")//.overlayBadge()
          statusBarItem.button?.image?.isTemplate = true
          statusBarItem.button?.wantsLayer = true
          
        }
      }
      
      configureStatusBarItem()
    }
    
    func setUpAccesibilityObserver(){
        
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
        TelemetryProvider.flushAll(includingCurrentDay: true)
    }
    
    @objc func newTerminalWindow() {
        WindowManager.shared.newNativeTerminalSession()
    }
  
    @objc func openSettingsDocs() {
      NSWorkspace.shared.open(URL(string: "https://fig.io/docs/support/settings")!)
    }
  
    @objc func editKeybindingsFile() {
      NSWorkspace.shared.open(KeyBindingsManager.keymapFilePath)
    }
    
    @objc func uninstall() {
        
        let confirmed = self.dialogOKCancel(question: "Uninstall Fig?", text: "You will need to restart any currently running terminal sessions.", icon: NSImage(imageLiteralResourceName: NSImage.applicationIconName))
        
        if confirmed {
            TelemetryProvider.track(event: .uninstallApp, with: [:])
          
            ShellHookManager.shared.ttys().forEach { (pair) in
               let (_, tty) = pair
               tty.setTitle("Restart this terminal to finish uninstalling Fig...")
            }
          
            var uninstallScriptFile: String? = "\(NSHomeDirectory())/.fig/tools/uninstall-script.sh"
            if !FileManager.default.fileExists(atPath: uninstallScriptFile!) {
               uninstallScriptFile = Bundle.main.path(forResource: "uninstall", ofType: "sh")
            }

            if let general = uninstallScriptFile {
                NSWorkspace.shared.open(URL(string: "https://fig.io/uninstall?email=\(Defaults.email ?? "")")!)
                LoginItems.shared.currentApplicationShouldLaunchOnStartup = false
                
                let domain = Bundle.main.bundleIdentifier!
                let uuid = Defaults.uuid
                UserDefaults.standard.removePersistentDomain(forName: domain)
                UserDefaults.standard.removePersistentDomain(forName: "\(domain).shared")

                UserDefaults.standard.synchronize()
                        
                UserDefaults.standard.set(uuid, forKey: "uuid")
                UserDefaults.standard.synchronize()
                
                WebView.deleteCache()
                
                let out = "bash \(general)".runAsCommand()
                Logger.log(message: out)
                self.quit()
            }
        }
    }
    
    @objc func sendFeedback() {
//        NSWorkspace.shared.open(URL(string:"mailto:hello@withfig.com")!)
      
        Github.openIssue()
        TelemetryProvider.track(event: .sendFeedback, with: [:])
    }
    
    @objc func setupScript() {
        TelemetryProvider.track(event: .runInstallationScript, with: [:])
        Onboarding.setUpEnviroment()
    }
    

    @objc func toggleiTermIntegration(_ sender: NSMenuItem) {
      iTermIntegration.promptToInstall {
        sender.state = iTermIntegration.isInstalled ? .on : .off
      }
    }
        
    func dialogOKCancel(question: String, text: String, prompt:String = "OK", noAction:Bool = false, icon: NSImage? = nil, noActionTitle: String? = nil) -> Bool {
        let alert = NSAlert() //NSImage.cautionName
        alert.icon = icon ?? NSImage(imageLiteralResourceName: "NSSecurity").overlayAppIcon()
        alert.icon.size = NSSize(width: 32, height: 32)
        alert.messageText = question
        alert.informativeText = text
        alert.alertStyle = .warning
        let button = alert.addButton(withTitle: prompt)
        button.highlight(true)
        if (!noAction) {
            alert.addButton(withTitle: noActionTitle ?? "Not now")
        }
        return alert.runModal() == .alertFirstButtonReturn
    }

    func handleUpdateIfNeeded() {
        Logger.log(message: "Checking if app has updated...")
        guard let previous = Defaults.versionAtPreviousLaunch else {
            Defaults.versionAtPreviousLaunch = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String
            print("Update: First launch!")
            Logger.log(message: "First launch!")
            TelemetryProvider.track(event: .firstTimeUser, with: [:])
            Onboarding.setUpEnviroment()
            return
        }
        
        guard let current = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String else {
            print("Update: No version detected.")
            return
        }
        
        // upgrade path!
        if (previous != current) {
            // look for $BUNDLE/upgrade/$OLD-->$NEW
            let specific = Bundle.main.path(forResource: "\(previous)-->\(current)", ofType: "sh")
            // look for $BUNDLE/upgrade/$NEW
            let general = Bundle.main.path(forResource: "\(current)", ofType: "sh")
            
            let script = specific ?? general
            if let script = script {
                print("Update: Running script '\(script)' to upgrade to version \(current)")
                let _ = "sh \(script) '\(Bundle.main.resourcePath ?? "")'".runAsCommand()
            }
            
            Onboarding.setUpEnviroment()

            TelemetryProvider.track(event: .updatedApp, with: ["script": script ?? "<none>"])
            
            // Any defaults that should be set for upgrading users
            // For anyone upgrading, we are just going to assume that this is true
            Defaults.hasShownAutocompletePopover = true
            
            // if iTerm integration exists prior to update, reinstall because it should be symlinked
            let iTermIntegrationPath = "\(NSHomeDirectory())/Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.py"
            if (FileManager.default.fileExists(atPath: iTermIntegrationPath)) {
                try? FileManager.default.removeItem(atPath: iTermIntegrationPath)
                let localScript = Bundle.main.path(forResource: "fig-iterm-integration", ofType: "py")!
                try? FileManager.default.createSymbolicLink(atPath: iTermIntegrationPath, withDestinationPath: localScript)
            }
            
            //remindToSourceFigInExistingTerminals()
        }
        
        Defaults.versionAtPreviousLaunch = current
    }
    
    @objc func restart() {
        Defaults.launchedFollowingCrash = false
        Logger.log(message: "Restarting Fig...")
        let url = URL(fileURLWithPath: Bundle.main.resourcePath!)
        let path = url.deletingLastPathComponent().deletingLastPathComponent().absoluteString
        let task = Process()
        task.launchPath = "/usr/bin/open"
        task.arguments = [path]
        task.launch()
        NSApp.terminate(self)
    }

    
    func setupCompanionWindow() {
        Logger.log(message: "Setting up companion windows")
        Defaults.loggedIn = true
        
        Logger.log(message: "Configuring status bar")
        self.configureStatusBarItem()
        
        Logger.log(message: "Creating windows...")
        WindowManager.shared.createSidebar()
        WindowManager.shared.createAutocomplete()
        
//        Logger.log(message: "Registering keystrokeHandler...")
//        KeypressProvider.shared.registerKeystrokeHandler()
//        
//        Logger.log(message: "Registering window tracking...")
//        AXWindowServer.shared.registerWindowTracking()
        
        //let companion = CompanionWindow(viewController: WebViewController())
        //companion.positioning = CompanionWindow.defaultPassivePosition
        //window = companion

        //(window as! CompanionWindow).repositionWindow(forceUpdate: true, explicit: true)
        //self.hotKeyManager = HotKeyManager(companion: window as! CompanionWindow)
    }
    
    //https://stackoverflow.com/a/35138823
//    func keyName(scanCode: UInt16) -> String? {
//        let maxNameLength = 4
//        var nameBuffer = [UniChar](repeating: 0, count : maxNameLength)
//        var nameLength = 0
//
//        let modifierKeys = UInt32(alphaLock >> 8) & 0xFF // Caps Lock
//        var deadKeys: UInt32 = 0
//        let keyboardType = UInt32(LMGetKbdType())
//
//        let source = TISCopyCurrentKeyboardLayoutInputSource().takeRetainedValue()
//        guard let ptr = TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData) else {
//            NSLog("Could not get keyboard layout data")
//            return nil
//        }
//        let layoutData = Unmanaged<CFData>.fromOpaque(ptr).takeUnretainedValue() as Data
//        let osStatus = layoutData.withUnsafeBytes {
//            UCKeyTranslate($0.bindMemory(to: UCKeyboardLayout.self).baseAddress, scanCode, UInt16(kUCKeyActionDown),
//                           modifierKeys, keyboardType, UInt32(kUCKeyTranslateNoDeadKeysMask),
//                           &deadKeys, maxNameLength, &nameLength, &nameBuffer)
//        }
//        guard osStatus == noErr else {
//            NSLog("Code: 0x%04X  Status: %+i", scanCode, osStatus);
//            return nil
//        }
//
//        return  String(utf16CodeUnits: nameBuffer, count: nameLength)
//    }
//
    @objc func inviteToSlack() {
        NSWorkspace.shared.open(URL(string: "https://fig-core-backend.herokuapp.com/community")!)
        TelemetryProvider.track(event: .joinSlack, with: [:])

    }
  
    @objc func inviteAFriend() {
      
      guard let email = Defaults.email else {
        Alert.show(title: "You are not logged in!", message: "Run `fig util:logout` and try again.", icon: Alert.appIcon)
        return
      }
      
      TelemetryProvider.track(event: .inviteAFriend, with: [:])
      
      let request = URLRequest(url: Remote.API.appendingPathComponent("/waitlist/get-referral-link-from-email/\(email)"))
      let task = URLSession.shared.dataTask(with: request) { (data, res, err) in
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
        TelemetryProvider.track(event: .viewDocs, with: [:])
    }
  
    @objc func viewSupportForum() {
        
        NSWorkspace.shared.open(URL(string: "https://fig.io/support")!)
        TelemetryProvider.track(event: .viewSupportForum, with: [:])
    }

    @objc func getKeyboardLayout() {
        guard let v = KeyboardLayout.shared.keyCode(for: "V"),
          let e = KeyboardLayout.shared.keyCode(for: "E"),
          let u = KeyboardLayout.shared.keyCode(for: "U") else {
            return
      }

        print("v=\(v); e=\(e); u=\(u)")
//        for var i in 0...100 {
//            print(i, keyName(scanCode: UInt16(i)))
//        }

    }
    
    @objc func toggleAutocomplete(_ sender: NSMenuItem) {
        Defaults.useAutocomplete = !Defaults.useAutocomplete
        sender.state = Defaults.useAutocomplete ? .on : .off
//        KeypressProvider.shared.clean()
        TelemetryProvider.track(event: .toggledAutocomplete, with: ["status" : Defaults.useAutocomplete ? "on" : "off"])

        if (Defaults.useAutocomplete) {
            WindowManager.shared.createAutocomplete()

            KeypressProvider.shared.registerKeystrokeHandler()
            AXWindowServer.shared.registerWindowTracking()
        }

        Logger.log(message: "Toggle autocomplete \(Defaults.useAutocomplete ? "on" : "off")")

    }

    @objc func  getSelectedText() {
        

//        ShellBridge.registerKeyInterceptor()
//        return
            

        NSEvent.addGlobalMonitorForEvents(matching: .keyUp) { (event) in
            print("keylogger:", event.characters ?? "", event.keyCode)
//        let touple = KeystrokeBuffer.shared.handleKeystroke(event: event)
//            guard touple != nil else {
//                WindowManager.shared.requestWindowUpdate()
//                return
//
//            }
        let systemWideElement = AXUIElementCreateSystemWide()
        var focusedElement : AnyObject?

        let error = AXUIElementCopyAttributeValue(systemWideElement, kAXFocusedUIElementAttribute as CFString, &focusedElement)
        if (error != .success){
            print("Couldn't get the focused element. Probably a webkit application")
        } else {
//            AXUIElement
          var names: CFArray?
          _ = AXUIElementCopyAttributeNames(focusedElement as! AXUIElement, &names)
          print(names as Any)

          var parametrizedNames: CFArray?
          _ = AXUIElementCopyParameterizedAttributeNames(focusedElement as! AXUIElement, &parametrizedNames)
          print(parametrizedNames as Any)
          //KeypressProvider.shared.getTextRect()
//          var markerRange : AnyObject?
//          let markerError = AXUIElementCopyAttributeValue(focusedElement as! AXUIElement, "AXSelectedTextMarkerRange" as CFString, &markerRange)
////          var markerRangeValue : AnyObject?
////          AXValueGetValue(markerRange as! AXValue, .cfRange, &markerRangeValue)
////          print(markerRangeValue)
//          guard markerRange != nil else {
//            print("selectedRect: markerRange is nil")
//            return
//          }
//          var selectBoundsForTextMarkerRange : AnyObject?
//          let err = AXUIElementCopyParameterizedAttributeValue(focusedElement as! AXUIElement, "AXBoundsForTextMarkerRange" as CFString, markerRange!, &selectBoundsForTextMarkerRange)
//          var selectRect = CGRect()
//          AXValueGetValue(selectBoundsForTextMarkerRange as! AXValue, .cgRect, &selectRect)
//          print("selectedRect: ", selectRect)
          
          //AXBoundsForTextMarkerRange
          
//            var selectedRangeValue : AnyObject?
//            let selectedRangeError = AXUIElementCopyAttributeValue(focusedElement as! AXUIElement, kAXSelectedTextRangeAttribute as CFString, &selectedRangeValue)
//
//            if (selectedRangeError == .success){
//                var selectedRange = CFRange()
//                AXValueGetValue(selectedRangeValue as! AXValue, .cfRange, &selectedRange)
//                var selectRect = CGRect()
//                var selectBounds : AnyObject?
////                print("selected", selectedRange)
////                print("selected", selectedRange.location, selectedRange.length)
//                var updatedRange = CFRangeMake(selectedRange.location, 1)
//                print("selected", selectedRange, updatedRange)
//
//                withUnsafeMutablePointer(to: &updatedRange) { (ptr) in
//                    let updatedRangeValue = AXValueCreate(AXValueType(rawValue: kAXValueCFRangeType)!, ptr)
//                    let selectedBoundsError = AXUIElementCopyParameterizedAttributeValue(focusedElement as! AXUIElement, kAXBoundsForRangeParameterizedAttribute as CFString, updatedRangeValue!, &selectBounds)
//                    if (selectedBoundsError == .success){
//                        AXValueGetValue(selectBounds as! AXValue, .cgRect, &selectRect)
//                        //do whatever you want with your selectRect
//                        print("selected", selectRect)
//                        WindowManager.shared.sidebar?.setOverlayFrame(selectRect)
//
//                    }
//                }
//
//                //kAXInsertionPointLineNumberAttribute
//                //kAXRangeForLineParameterizedAttribute
//
//
//            }
        }
        }
    }
    @objc func newAccesibilityAPI() {
//        Onboarding.installation()
//        "whoami".runWithElevatedPrivileges()
//        ShellBridge.promptForAccesibilityAccess { (enabled) in
//            print("AXCallback:", enabled)
//        }
    }
    var observer: AXObserver?

    @objc func addAccesbilityObserver() {
        let first = WindowServer.shared.topmostWindow(for: NSWorkspace.shared.frontmostApplication!)!
        print(first.bundleId ?? "?")
        let axErr = AXObserverCreate(first.app.processIdentifier, { (observer: AXObserver, element: AXUIElement, notificationName: CFString, refcon: UnsafeMutableRawPointer?) -> Void in
                print("axobserver:", notificationName)
                print("axobserver:", element)
                print("axobserver:", observer)
                print("axobserver:", refcon as Any)

//            WindowManager.shared.requestWindowUpdate()
            
        }, &observer)
        
        //kAXWindowMovedNotification
        let out = AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXFocusedWindowChangedNotification as CFString, nil)
        print("axobserver:", out)
        let hi = AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXMainWindowChangedNotification as CFString, nil)
        print("axobserver:", hi)
        
        AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXWindowCreatedNotification as CFString, nil)
        AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXWindowMiniaturizedNotification as CFString, nil)
        AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXWindowDeminiaturizedNotification as CFString, nil)
        AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXWindowCreatedNotification as CFString, nil)
        AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXWindowCreatedNotification as CFString, nil)
        AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXApplicationShownNotification as CFString, nil)
        AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXApplicationHiddenNotification as CFString, nil)
        AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXApplicationActivatedNotification as CFString, nil)
        AXObserverAddNotification(observer!, AXUIElementCreateApplication(first.app.processIdentifier), kAXApplicationDeactivatedNotification as CFString, nil)
        
        AXObserverAddNotification(observer!, first.accesibilityElement!, kAXWindowMovedNotification as CFString, nil)
        AXObserverAddNotification(observer!, first.accesibilityElement!, kAXWindowResizedNotification as CFString, nil)
//        _ element: AXUIElement,
//        _ notification: CFString,
//        _ refcon: UnsafeMutableRawPointer?)
//        AXObserverAddNotification(observer,
        
        //[[NSRunLoop currentRunLoop] getCFRunLoop]
        print(axErr)
        print(observer as Any)
        CFRunLoopAddSource(CFRunLoopGetCurrent(), AXObserverGetRunLoopSource(observer!), CFRunLoopMode.defaultMode);
        

//        CFRunLoopAddSource( RunLoop.current.getCFRunLoop()), AXObserverGetRunLoopSource(observer), kCFRunLoopDefaultMode );
    }
    
    @objc func toggleOnlyTab(_ sender: NSMenuItem){
        Defaults.onlyInsertOnTab = !Defaults.onlyInsertOnTab
        sender.state = Defaults.onlyInsertOnTab ? .on : .off
    }
    
    @objc func toggleSidebar(_ sender: NSMenuItem) {
//         if let companion = self.window as? CompanionWindow,
//            let vc = companion.contentViewController as? WebViewController,
//            let webView = vc.webView {
//            companion.positioning = .icon
//            webView.loadRemoteApp(at: Remote.baseURL.appendingPathComponent("hide"))
//
//        }
        
        Defaults.showSidebar = !Defaults.showSidebar
        sender.state = Defaults.showSidebar ? .on : .off
        WindowManager.shared.requestWindowUpdate()
        
        TelemetryProvider.track(event: .toggledSidebar, with: ["status" : Defaults.useAutocomplete ? "on" : "off"])
    }
    
        @objc func toggleLogging(_ sender: NSMenuItem) {
            
            Defaults.broadcastLogs = !Defaults.broadcastLogs
            sender.state = Defaults.broadcastLogs ? .on : .off
            
        }
    
    @objc func toggleFigIndicator(_ sender: NSMenuItem) {
        AutocompleteContextNotifier.addIndicatorToTitlebar = !AutocompleteContextNotifier.addIndicatorToTitlebar
        sender.state = AutocompleteContextNotifier.addIndicatorToTitlebar ? .on : .off
    }
  
    @objc func toggleSSHIntegration(_ sender: NSMenuItem) {
        
        let SSHConfigFile = URL(fileURLWithPath:  "\(NSHomeDirectory())/.ssh/config")
        let configuration = try? String(contentsOf: SSHConfigFile)
        
        // config file does not exist or fig hasn't been enabled
        if (!(configuration?.contains("Fig SSH Integration: Enabled") ?? false)) {
            guard self.dialogOKCancel(question: "Install SSH integration?", text: "Fig will make changes to your SSH config (stored in ~/.ssh/config).") else {
                return
            }
            
            SSHIntegration.install()
            let _ = self.dialogOKCancel(question: "SSH Integration Installed!", text: "When you connect to a remote machine using SSH, Fig will show relevant completions.\n\nIf you run into any issues, please email hello@fig.io.", noAction: true, icon: NSImage.init(imageLiteralResourceName: NSImage.applicationIconName))
            return
        }
        
        Defaults.SSHIntegrationEnabled = !Defaults.SSHIntegrationEnabled
        sender.state = Defaults.SSHIntegrationEnabled ? .on : .off
    }
  
    @objc func toggleVSCodeIntegration(_ sender: NSMenuItem) {
        
      VSCodeIntegration.promptToInstall {
        sender.state = VSCodeIntegration.isInstalled ? .on : .off
      }
        
    }
  
    @objc func toggleHyperIntegration(_ sender: NSMenuItem) {
        
      HyperIntegration.promptToInstall {
        sender.state = HyperIntegration.isInstalled ? .on : .off
      }
        
    }
    
    @objc func toggleDebugAutocomplete(_ sender: NSMenuItem) {
        Defaults.debugAutocomplete = !Defaults.debugAutocomplete
        sender.state = Defaults.debugAutocomplete ? .on : .off
        
        if (!Defaults.debugAutocomplete) {
            WindowManager.shared.autocomplete?.maxHeight = 0
        }
        
    }

    
    @objc func terminalWindowToFront() {
        WindowManager.shared.bringTerminalWindowToFront()
    }
    

    @objc func pid() {
        if let window = WindowServer.shared.topmostWhitelistedWindow() {
            print("\(window.bundleId ?? "") -  pid:\(window.app.processIdentifier) - \(window.windowId)")
        }
    }
    
    @objc func checkForUpdates() {
          self.updater.checkForUpdates(self)
//        self.updater?.installUpdatesIfAvailable()
    }
    @objc func toggleVisibility() {
        if let window = self.window {
           let companion = window as! CompanionWindow
           let position = companion.positioning
           
           if(NSWorkspace.shared.frontmostApplication?.isFig ?? false) {
               ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
           }
           
            if position == CompanionWindow.defaultPassivePosition {
               companion.positioning = CompanionWindow.defaultActivePosition
                NSRunningApplication.current.activate(options: .activateIgnoringOtherApps)
           } else {
               companion.positioning = CompanionWindow.defaultPassivePosition
//            ShellBridge.shared.previousFrontmostApplication?.activate(options: .activateIgnoringOtherApps)
           }
       }
    }
    
    @available(macOS, deprecated: 10.11)
    @objc func applicationIsInStartUpItems() -> Bool {
      return LoginItems.shared.includesCurrentApplication
    }

    
    @objc func quit() {
        
        if let statusbar = self.statusBarItem {
            NSStatusBar.system.removeStatusItem(statusbar)
        }
      
        Config.set(value: "1", forKey: Config.userExplictlyQuitApp)
        
        TelemetryProvider.track(event: .quitApp, with: [:]) { (_, _, _) in
            DispatchQueue.main.async {
                 NSApp.terminate(self)
             }
        }
        
//        Timer.delayWithSeconds(15) {
//                DispatchQueue.main.async {
//                 NSApp.terminate(self)
//             }
//        }

    }
  
    @objc func toggleDeveloperMode() {
        Defaults.toggleDeveloperMode()
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
        print("spaceChanged!");
    }
    
    @objc func newActiveApp() {
        print("newActiveApp!");
    }
    func applicationWillTerminate(_ aNotification: Notification) {
        ShellBridge.shared.stopWebSocketServer()
        Defaults.launchedFollowingCrash = false
        AutocompleteContextNotifier.clearFigContext()
        
        Logger.log(message: "app will terminate...")
    }
    
    @objc func runScriptCmd() {
        let path = "~/session.fig"//getDocumentsDirectory().appendingPathComponent("user.fig")
        print(path)
        injectStringIntoTerminal("script -q -t 0 \(path)")
    }
    
    @objc func runTailCmd() {
        let path = "~/session.fig"//getDocumentsDirectory().appendingPathComponent("user.fig")

        let output = "tail -F \(path)".runAsCommand()
        
        print(output)
    }
        
    @objc func runExitCmd() {
         injectStringIntoTerminal("exit")
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
            
            if (result == .apiDisabled) {
                print("Accesibility needs to be enabled.")
                return
            }
            
            print(window ?? "<none>" )
            
            var position : AnyObject?
            var size : AnyObject?

            let result2 = AXUIElementCopyAttributeValue(window as! AXUIElement, kAXPositionAttribute as CFString, &position)
            AXUIElementCopyAttributeValue(window as! AXUIElement, kAXSizeAttribute as CFString, &size)

            switch(result2) {
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
                let point = AXValueGetters.asCGPoint(value: position as! AXValue)
                let bounds = AXValueGetters.asCGSize(value: size as! AXValue)
                print(point, bounds)
                
                
                let titleBarHeight:CGFloat = 23.0;
                
                let includeTitleBarHeight = false;
                
                let terminalWindowFrame = NSRect.init(x: point.x, y: (NSScreen.main?.visibleFrame.height)! - point.y + ((includeTitleBarHeight) ? titleBarHeight : 0), width: bounds.width, height: bounds.height - ((includeTitleBarHeight) ? 0 : titleBarHeight))
                    //CGRect.init(origin: point, size: bounds)
                print(terminalWindowFrame)
//                let terminalFrame = NSRectFromCGRect(terminalWindowFrame)
                self.window.windowController?.shouldCascadeWindows = false;
                
//                print("Before:", terminalWindowFrame)
//                let figWindow = overlayFrame(OverlayPositioning.init(rawValue: self.clicks % 7)!, terminalFrame: terminalWindowFrame, screenBounds: .zero)
//                print("After:", figWindow)

//                self.window.setFrame(figWindow, display: true)
//                self.window.setFrameTopLeftPoint(figWindow.origin)
                self.clicks += 1;
//                self.window.setFrameOrigin(NSPoint.init(x: point.x, y: (point.y < NSScreen.main!.frame.height/2) ? point.y + bounds.height : point.y - bounds.height) )
////                self.window.cascadeTopLeft(from: NSPointFromCGPoint(point))

                print(self.window.frame)
            }
            


            //
        }

//        let type = CGWindowListOption.optionOnScreenOnly
//        let windowList = CGWindowListCopyWindowInfo(type, kCGNullWindowID) as NSArray? as? [[String: AnyObject]]
//
//        for entry  in windowList!
//        {
//          let owner = entry[kCGWindowOwnerName as String] as! String
//          var bounds = entry[kCGWindowBounds as String] as? [String: Int]
//          let pid = entry[kCGWindowOwnerPID as String] as? Int32
//
//          if owner == "iTerm2"
//          {
//            let appRef = AXUIElementCreateApplication(pid!);  //TopLevel Accessability Object of PID
//
//            var value: AnyObject?
//            let result = AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &value)
//
//            if let windowList = value as? [AXUIElement]
//            { print ("windowList #\(windowList)")
//              if let window = windowList.first
//              {
//                print(window)
//                var position : CFTypeRef
//                var size : CFTypeRef
//                var  newPoint = CGPoint(x: 0, y: 0)
//                var newSize = CGSize(width: 800, height: 800)
//
//                position = AXValueCreate(AXValueType(rawValue: kAXValueCGPointType)!,&newPoint)!;
//                AXUIElementSetAttributeValue(windowList.first!, kAXPositionAttribute as CFString, position);
//
//               // AXUIElementCopyAttributeValue(windowList.first!, kAXPositionAttribute as CFString, )
//
//                size = AXValueCreate(AXValueType(rawValue: kAXValueCGSizeType)!,&newSize)!;
//                AXUIElementSetAttributeValue(windowList.first!, kAXSizeAttribute as CFString, size);
//
//                print(newSize)
//              }
//            }
//          }
//        }
    }
    
    @objc func processes() {
        printProcesses("")
        var size: Int32 = 0
        if let ptr = getProcessInfo("", &size) {
            let buffer = UnsafeMutableBufferPointer<fig_proc_info>(start: ptr, count: Int(size))
            
            buffer.forEach { (process) in
                var proc = process

//            var proc = ptr.pointee
//            String(cString: proc.tty),  String(cString: proc.cmd)
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
            
            print("proc: ",  proc.pid, cwd, cmd, tty)
            }
           free(ptr)
        }
    }
    @objc func triggerScreenReader() {
      VSCodeIntegration.install(inBackground: true) {
        Alert.show(title: "VSCode Integration Installed!", message: "The Fig extension was successfully added to VSCode.", hasSecondaryOption: false)
//      if let app = AXWindowServer.shared.topApplication, let window = AXWindowServer.shared.topWindow {
//        print("Triggering ScreenreaderMode in \(app.bundleIdentifier ?? "<unknown>")")
//        Accessibility.triggerScreenReaderModeInChromiumApplication(app)
//        let cursor = Accessibility.findXTermCursorInElectronWindow(window)
//        print("Detect cursor:", cursor ?? .zero)
      }
    }
    @objc func allWindows() {
        
        Timer.delayWithSeconds(3) {
            guard let jsons = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[String: Any]] else {
                return
            }

            let infos = jsons.compactMap({ WindowInfo(json: $0) })
            print (infos)
            
            print (infos.filter ({
                return NSRunningApplication(processIdentifier: pid_t($0.pid))?.bundleIdentifier == "com.apple.Spotlight"
            }))
        }

        
//        if let info = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[ String : Any]] {
//            for dict in info {
//                print(dict)
//            }
//        }
    }
    
    @objc func pasteStringToTerminal() {
        let terminals = NSRunningApplication.runningApplications(withBundleIdentifier: "com.googlecode.iterm2")
        if let activeTerminal = terminals.first {
            activeTerminal.activate(options: NSApplication.ActivationOptions.init())
            simulateKeyPress(pid: activeTerminal.processIdentifier)
        }
               
 
    }
    
    @objc func frontmostApplication() {
        print (NSWorkspace.shared.frontmostApplication?.localizedName ?? "")
    }
    
    @objc func copyToPasteboard() {
        let input = "echo \"hello world\""

        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(input, forType: .string)
        
    }
    
    func injectStringIntoTerminal(_ cmd: String, runImmediately: Bool = false) {
         if let currentApp = NSWorkspace.shared.frontmostApplication {
                
            if (currentApp.bundleIdentifier == "com.googlecode.iterm2") {
                // save current pasteboard
                let pasteboard = NSPasteboard.general
                let copiedString = pasteboard.string(forType: .string) ?? ""
                
                // add our script to pasteboard
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(cmd, forType: .string)
                print(pasteboard.string(forType: .string) ?? "")
                    self.simulate(keypress: .cmdV)
                    self.simulate(keypress: .rightArrow)
                    self.simulate(keypress: .enter)
 
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
        
        let input = "echo \"hello world\""
        if let currentApp = NSWorkspace.shared.frontmostApplication {
        
            if (currentApp.bundleIdentifier == "com.googlecode.iterm2") {
                // save current pasteboard
                let pasteboard = NSPasteboard.general
                let copiedString = pasteboard.string(forType: .string) ?? ""
                
                // add our script to pasteboard
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(input, forType: .string)
                print(pasteboard.string(forType: .string) ?? "")
//                simulateRawKeyPress(flag: true)
                    self.simulate(keypress: .cmdV)
                    self.simulate(keypress: .rightArrow)
                    self.simulate(keypress: .enter)
 
                // need delay so that terminal responds
                Timer.delayWithSeconds(1) {
                    // restore pasteboard
                    NSPasteboard.general.clearContents()
                    pasteboard.setString(copiedString, forType: .string)
                }
            }
        }
    }
    
    @objc func checkWinows() {
        
        let windowNumbers = NSWindow.windowNumbers(options: [])
        windowNumbers?.forEach( { print($0.decimalValue) })
    }
    
    // /Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/Events.h
    //https://gist.github.com/eegrok/949034
    enum Keypress: UInt16 {
        case cmdV = 9
        case enter = 36
        case rightArrow = 124
    }
    
    func simulate(keypress: Keypress) {
        let keyCode = keypress.rawValue as CGKeyCode
//        print(keypress.rawValue, keyCode)
        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let keydown = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: true)
        let keyup = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: false)
        
        if (keypress == .cmdV){
            keydown?.flags = CGEventFlags.maskCommand;
        }
        
        let loc = CGEventTapLocation.cghidEventTap
        keydown?.post(tap: loc)
        keyup?.post(tap: loc)
    }
    
    func simulateRawKeyPress(flag: Bool = false) {
        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let v_down = CGEvent(keyboardEventSource: src, virtualKey: 9 as CGKeyCode, keyDown: true)
        let v_up = CGEvent(keyboardEventSource: src, virtualKey: 9 as CGKeyCode, keyDown: false)
        
        if (flag){
            v_down?.flags = CGEventFlags.maskCommand;
        }
        
        let loc = CGEventTapLocation.cghidEventTap
        v_down?.post(tap: loc)
        v_up?.post(tap: loc)
    }

    func simulateKeyPress(pid: pid_t, flag: Bool = false) {
        print("Simulate keypress for process: \(pid)")

        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let v_down = CGEvent(keyboardEventSource: src, virtualKey: 9 as CGKeyCode, keyDown: true)
        let v_up = CGEvent(keyboardEventSource: src, virtualKey: 9 as CGKeyCode, keyDown: false)
//        let spcd = CGEvent(keyboardEventSource: src, virtualKey: 0x31, keyDown: true)
//        let spcu = CGEvent(keyboardEventSource: src, virtualKey: 0x31, keyDown: false)

        if (flag){
            v_down?.flags = CGEventFlags.maskCommand;
        }
//        v_up?.flags = CGEventFlags.maskCommand;

//        let loc = CGEventTapLocation.cghidEventTap
        

        v_down?.postToPid(pid)
        v_up?.postToPid(pid)

//        v_down?.post(tap: loc)
//        v_up?.post(tap: loc)

//        spcd?.post(tap: loc)
//        spcu?.post(tap: loc)
//        cmdu?.post(tap: loc)
    }
    
    func windowDidMove(_ notification: Notification) {
        print(notification.object ?? "<none>")
        
        print("WINDOW MOVED", window.frame)
//        print("SCREEN", NSScreen.main?.frame ?? "<none>")
    }


}

fileprivate func delayWithSeconds(_ seconds: Double, completion: @escaping () -> ()) {
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

        guard let x = rect["X"] as? CGFloat else {
            return nil
        }

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

extension AppDelegate : SUUpdaterDelegate {
    func updater(_ updater: SUUpdater, didAbortWithError error: Error) {
        
    }
    
    func updaterDidNotFindUpdate(_ updater: SUUpdater) {
        
    }
    
    func updater(_ updater: SUUpdater, didFindValidUpdate item: SUAppcastItem) {
        print("Found valid update")
    }
    
    func updater(_ updater: SUUpdater, didFinishLoading appcast: SUAppcast) {
//        let item = (appcast.items?.first! as! SUAppcastItem)
//        item.
    }
}

extension AppDelegate : NSMenuDelegate {
    func menuDidClose(_ menu: NSMenu) {
        print("menuDidClose")

    }
    
    @objc func windowDidChange(_ notification: Notification){
//        if let app = NSWorkspace.shared.frontmostApplication {
//            if Integrations.nativeTerminals.contains(app.bundleIdentifier ?? "") {
//                let window = AXWindowServer.shared.whitelistedWindow
//                let tty = window?.tty
//                var hasContext = false
//
//                if let window = window {
//                   let keybuffer = KeypressProvider.shared.keyBuffer(for: window)
//                   hasContext = keybuffer.buffer != nil
//                }
//
//                let hasWindow = window != nil
//                let hasCommand = tty?.cmd != nil
//                let isShell = tty?.isShell ?? true
//
//                var color: NSColor = .clear
//
//                if (!hasWindow) {
//                   color = .red
//
//                } else if (!hasContext) {
//                   color = .orange
//
//                } else if (!hasCommand) {
//                   color = .yellow
//
//                } else if (!isShell) {
//                   color = .cyan
//
//                } else {
//                   color = .green
//                }
//
//                statusBarItem.button?.image?.isTemplate = false
//                statusBarItem.button?.image = NSImage(imageLiteralResourceName: "statusbar@2x.png").overlayBadge(color: color, text: "")
//                return
//            }
//        }
//
//        statusBarItem.button?.image?.isTemplate = true
//        statusBarItem.button?.image = NSImage(imageLiteralResourceName: "statusbar@2x.png")
        
    }
    
    @objc func forceUpdateTTY() {
        if let tty = AXWindowServer.shared.whitelistedWindow?.tty {
            tty.update()
        }
    }
    
    @objc func addProcessToWhitelist() {
        if let tty = AXWindowServer.shared.whitelistedWindow?.tty, let cmd = tty.cmd {
            Defaults.processWhitelist = Defaults.processWhitelist + [cmd]
            tty.update()
        }
    }
    
    @objc func addProcessToIgnorelist() {
        if let tty = AXWindowServer.shared.whitelistedWindow?.tty, let cmd = tty.cmd {
            Defaults.ignoreProcessList = Defaults.ignoreProcessList + [cmd]
            tty.update()
        }
    }
    
    @objc func resetWindowTracking() {
        
//        AXWindowServer.shared.registerWindowTracking()
//        self.statusBarItem.menu?.cancelTracking()
        if let app = NSWorkspace.shared.frontmostApplication {
            AXWindowServer.shared.register(app, fromActivation: false)
        }
    }
    
    func menuWillOpen(_ menu: NSMenu) {
        print("menuWillOpen")
        DispatchQueue.global(qos: .background).async {
          TelemetryProvider.track(event: .openedFigMenuIcon, with: [:])
        }
        guard Defaults.loggedIn, Accessibility.enabled else {
            return
        }
        
        if let frontmost = self.frontmost {
            if menu.items.contains(frontmost) {
                menu.removeItem(frontmost)
            }
            
            self.frontmost = nil
        }
        
        if let app = NSWorkspace.shared.frontmostApplication, !app.isFig {
            let window = AXWindowServer.shared.whitelistedWindow
          if Integrations.terminalsWhereAutocompleteShouldAppear.contains(window?.bundleId ?? "") ||  Integrations.terminalsWhereAutocompleteShouldAppear.contains(app.bundleIdentifier ?? "") {
                let tty = window?.tty
                var hasContext = false
                var isHidden = false
                var bufferDescription: String? = nil
                var backedByShell = false
                var backing: String?


                if let window = window {
                    let keybuffer = KeypressProvider.shared.keyBuffer(for: window)
                    hasContext = keybuffer.buffer != nil && !keybuffer.writeOnly
                    isHidden = keybuffer.buffer != nil && keybuffer.writeOnly
                    bufferDescription = keybuffer.representation
                    backedByShell = keybuffer.backedByShell
                   
                  switch keybuffer.backing {
                  case .zle:
                    backing = "ZSH Line Editor"
                    break;
                  case .fish:
                    backing = "Fish Command Line"
                  case .bash:
                    backing = "Bash Command Line"
                  default:
                    backing = nil
                  }
                  
                }

                let hasWindow = window != nil
                let hasCommand = tty?.cmd != nil
                let isShell = tty?.isShell ?? true
                let runUsingPrefix = tty?.runUsingPrefix
              
                let cmd = tty?.cmd != nil ? "(\(tty?.name ?? tty!.cmd!))" : "(???)"
                
                var color: NSColor = .clear
                let legend = NSMenu(title: "legend")
                
                let companionWindow = WindowManager.shared.autocomplete
                if let (message, hexString, shouldDisplay) = companionWindow?.status, shouldDisplay {
                    color = hexString != nil ? (NSColor(hex: hexString!) ?? .red) : .red
                  
                  message.split(separator: "\n").forEach { (str) in
                    if str == "---" {
                      legend.addItem(NSMenuItem.separator())
                    } else {
                      legend.addItem(NSMenuItem(title: String(str), action: nil, keyEquivalent: ""))
                    }
                  }
                    
                } else if !Integrations.terminalsWhereAutocompleteShouldAppear.contains(window?.bundleId ?? "") {
                  color = .orange
                  legend.addItem(NSMenuItem(title: "Not tracking window...", action: nil, keyEquivalent: ""))
                  
                  legend.addItem(NSMenuItem.separator())
                  legend.addItem(NSMenuItem(title: "Switch to a different application", action: nil, keyEquivalent: ""))
                  legend.addItem(NSMenuItem(title: "and then return to current window", action: nil, keyEquivalent: ""))
                  
                } else if let loaded = companionWindow?.loaded, !loaded {
                    color = .red
                    legend.addItem(NSMenuItem(title: "Autocomplete could not be loaded.", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "Make sure you're connected to", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem(title: "the internet and try again.", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "Reload Autocomplete", action: #selector(restart), keyEquivalent: ""))

                } else if (!hasWindow) {
                    color = .red
                    legend.addItem(NSMenuItem(title: "Window is not being tracked.", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "Reset Window Tracking", action: #selector(resetWindowTracking), keyEquivalent: ""))
                    legend.addItem(NSMenuItem(title: "Restart Fig", action: #selector(restart), keyEquivalent: ""))


                } else if (!Diagnostic.installationScriptRan) {
                    color = .red
                    legend.addItem(NSMenuItem(title: "~/.fig directory is misconfigured", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "Re-run Install Script", action: #selector(setupScript), keyEquivalent: ""))
                  
                } else if (SecureKeyboardInput.enabled || (SecureKeyboardInput.wasEnabled && window?.bundleId == Integrations.Terminal)) {
                    // Also check previous value (wasEnabled) because clicking on menubar icon will disable secure keyboard input in Terminal.app
                    color = .systemPink
                    legend.addItem(NSMenuItem(title: "'Secure Keyboard Input' Enabled", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "This prevents Fig from", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem(title: "processing keypress events. ", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())

                  let app = SecureKeyboardInput.responsibleApplication ?? app
                  let pid = SecureKeyboardInput.responsibleProcessId ?? app.processIdentifier
                  if SecureKeyboardInput.enabled(by: window?.bundleId),
                     let name = app.localizedName {
                        let open = NSMenuItem(title: "Disable in '\(name)' (\(pid)).", action: #selector(SecureKeyboardInput.openRelevantMenu), keyEquivalent: "")
                        open.target = SecureKeyboardInput.self
                        legend.addItem(open)

                    } else {
                      //Run `ioreg -l -w 0 | grep SecureInput` to determine which app is responsible.
                        let lock = NSMenuItem(title: "Lock screen and log back in", action: #selector(SecureKeyboardInput.lockscreen), keyEquivalent: "")
                        lock.target = SecureKeyboardInput.self
                        legend.addItem(lock)
                    }
                  
                    legend.addItem(NSMenuItem.separator())
                    let support = NSMenuItem(title: "Learn more", action: #selector(SecureKeyboardInput.openSupportPage), keyEquivalent: "")
                    support.target = SecureKeyboardInput.self
                    legend.addItem(support)

                } else if (isHidden) {
                    color = .orange
                    legend.addItem(NSMenuItem(title: "Autocomplete is hidden.", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())

                    if let onlyShowOnTab = Settings.shared.getValue(forKey: Settings.onlyShowOnTabKey) as? Bool, onlyShowOnTab {
                      legend.addItem(NSMenuItem(title: "Press <tab> to show suggestions", action: nil, keyEquivalent: ""))
                      legend.addItem(NSMenuItem.separator())
                      legend.addItem(NSMenuItem(title: "Or update '\(Settings.onlyShowOnTabKey)' setting", action: nil, keyEquivalent: ""))

                    } else {
                      legend.addItem(NSMenuItem(title: "Press control + <escape>", action: nil, keyEquivalent: ""))
                      legend.addItem(NSMenuItem(title: "to toggle it back on", action: nil, keyEquivalent: ""))

                    }
                  
                } else if (!hasContext) {
                    color = .orange
                    legend.addItem(NSMenuItem(title: "Fig is unsure what you typed", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "Go to a new line by pressing", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem(title: "<enter> or ctrl+c", action: nil, keyEquivalent: ""))

                } else if (!hasCommand) {
                    color = .yellow
                    legend.addItem(NSMenuItem(title: "Not linked to TTY session.", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "Run `fig source` to connect.", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "window: \(window?.hash ?? "???")", action: nil, keyEquivalent: ""))

                } else if (!isShell) {
                    color = .cyan
                    legend.addItem(NSMenuItem(title: "Running proccess (\(tty?.cmd ?? "(???)")) is not a shell.", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "Fix: exit current process", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "Force Reset", action: #selector(forceUpdateTTY), keyEquivalent: ""))
                    legend.addItem(NSMenuItem(title: "Add as Shell", action: #selector(addProcessToWhitelist), keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "Ignore", action: #selector(addProcessToIgnorelist), keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "window: \(window?.hash ?? "???")", action: nil, keyEquivalent: ""))
                } else {
                    color = .green
                  
                    let path = Diagnostic.pseudoTerminalPathAppearsValid
                  
                    legend.addItem(NSMenuItem(title: "Everything should be working.", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem.separator())
                    legend.addItem(NSMenuItem(title: "window: \(window?.hash.truncate(length: 15, trailing: "...") ?? "???")", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem(title: "tty: \(tty?.descriptor ?? "???")", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem(title: "cwd: \(tty?.cwd ?? "???")", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem(title: "pid: \(tty?.pid ?? -1)", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem(title: "keybuffer: \(bufferDescription ?? "???")", action: nil, keyEquivalent: ""))
                    legend.addItem(NSMenuItem(title: "path: \( path != nil ? (path! ? "☑" : "☒ ") : "<generated dynamically>")", action: nil, keyEquivalent: ""))

                    if runUsingPrefix != nil {
                      legend.addItem(NSMenuItem.separator())
                      legend.addItem(NSMenuItem(title: "In SSH session or Docker container", action: nil, keyEquivalent: ""))
                    }
                  
                    if backedByShell {
                      legend.addItem(NSMenuItem.separator())
                      legend.addItem(NSMenuItem(title: "Backed by \(backing ?? "???")", action: nil, keyEquivalent: ""))
                    }
                }
                
                
                let title = "Debugger \(cmd)"//"\(app.localizedName ?? "Unknown") \(cmd)"
                var image: NSImage?
                if let pid = window?.app.processIdentifier, let windowApp = NSRunningApplication(processIdentifier: pid) {
                  image = windowApp.icon
                } else {
                  image = app.icon
                }

                let icon = image?.resized(to: NSSize(width: 16, height: 16))?.overlayBadge(color: color, text: "")
                
                let app = NSMenuItem(title: title, action: nil, keyEquivalent: "")
                app.image = icon
                app.submenu = legend
                menu.insertItem(app, at: 0)
                
                

                self.frontmost = app
            } else {
//                let title = "\(app.localizedName ?? "Unknown") \(cmd)"
                let icon = app.icon?.resized(to: NSSize(width: 16, height: 16))//?.overlayBadge(color: .red, text: "")
                
                let text = Integrations.autocompleteBlocklist.contains(app.bundleIdentifier ?? "") ? "has been disabled." : "is not supported."
            
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
        let item = NSMenuItem(title: "Rerun Install Script", action: #selector(AppDelegate.setupScript) , keyEquivalent: "")
         item.image = NSImage(named: NSImage.Name("alert"))
            menu.insertItem(item, at: 1)
            self.integrationPrompt = item
          return
       }
      
      if let app = NSWorkspace.shared.frontmostApplication,
        !app.isFig,
        let provider = Integrations.providers[app.bundleIdentifier ?? ""],
        !provider.isInstalled {
    
        
        
          let name: String!
          
          switch app.bundleIdentifier {
          case Integrations.iTerm:
            name = "iTerm"
          case Integrations.Hyper:
            name = "Hyper"
          case Integrations.VSCode:
            name = "VSCode"
          case Integrations.VSCodeInsiders:
            name = "VSCode Insiders"
          default:
            name = "Unknown"
          }

        let item = NSMenuItem(title: "Install \(name!) Integration", action: #selector(AppDelegate.installIntegrationForFrontmostApp) , keyEquivalent: "")
        item.image = NSImage(named: NSImage.Name("carrot"))
           menu.insertItem(item, at: 1)
           self.integrationPrompt = item
        }

        
    }
  
  @objc func installIntegrationForFrontmostApp() {
    if let app = NSWorkspace.shared.frontmostApplication, let provider = Integrations.providers[app.bundleIdentifier ?? ""], !provider.isInstalled {
      
        provider.promptToInstall(completion: nil)
      
    }
  }
}

extension NSApplication {
  var appDelegate: AppDelegate {
    return NSApp.delegate as! AppDelegate
  }
    
}
