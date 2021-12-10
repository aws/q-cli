//
//  ShellBridge.swift
//  fig
//
//  Created by Matt Schrage on 4/18/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Cocoa
import OSLog

protocol ShellBridgeEventListener {
    func recievedDataFromPipe(_ notification: Notification)
    func recievedUserInputFromTerminal(_ notification: Notification)
    func recievedStdoutFromTerminal(_ notification: Notification)
    
    func recievedDataFromPty(_ notification: Notification)
    func currentDirectoryDidChange(_ notification: Notification)
    func currentTabDidChange(_ notification: Notification)
    func startedNewTerminalSession(_ notification: Notification)
    func shellPromptWillReturn(_ notification: Notification)

}

extension Notification.Name {
    static let shellPromptWillReturn = Notification.Name("shellPromptWillReturn")
    static let startedNewTerminalSession = Notification.Name("startedNewTerminalSession")
    static let currentTabDidChange = Notification.Name("currentTabDidChange")
    static let currentDirectoryDidChange = Notification.Name("currentDirectoryDidChange")
    static let recievedShellTrackingEvent = Notification.Name("recievedShellTrackingEvent")
    static let recievedDataFromPipe = Notification.Name("recievedDataFromPipe")
    static let recievedUserInputFromTerminal = Notification.Name("recievedUserInputFromTerminal")
    static let recievedStdoutFromTerminal = Notification.Name("recievedStdoutFromTerminal")
    static let recievedDataFromPty = Notification.Name("recievedDataFromPty")

}

protocol MouseMonitoring {
    func requestStopMonitoringMouseEvents(_ notification: Notification)
    func requestStartMonitoringMouseEvents(_ notification: Notification)
}

extension Notification.Name {
    static let requestStopMonitoringMouseEvents = Notification.Name("requestStopMonitoringMouseEvents")
    static let requestStartMonitoringMouseEvents = Notification.Name("requestStartMonitoringMouseEvents")
}

class ShellBridge {
    static let shared = ShellBridge()
    
    var rawOutput = ""
    var streamHandlers: Set<String> = []
    var executeHandlers: Set<String> = []
    
    var previousFrontmostApplication: NSRunningApplication?
//    var socket: WebSocketConnection?
//    var socketServer: Process?
    init() {
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(setPreviousApplication(notification:)), name: NSWorkspace.didDeactivateApplicationNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(spaceChanged), name: NSWorkspace.activeSpaceDidChangeNotification, object: nil)
        self.startWebSocketServer()
    }
    
    let executeDelimeter = "-----------------"
    let streamDelimeter = "================="
    
    //http://www.physics.udel.edu/~watson/scen103/ascii.html
    enum ControlCode : String {
        typealias RawValue = String
        case EOT = "^D"
        case ETX = "^C"
        
    }
    
    func startWebSocketServer() {
//        self.socketServer = WebSocketServer.bridge
    }
    
    func stopWebSocketServer( completion:(() -> Void)? = nil) {
        if let completion = completion {
         completion()
        }
    }
    
    // This fixes an issue where focus would bounce to an application in the previous workspace. Essentially this resets previous application anytime the workspace is changed.
    
    @objc func spaceChanged() {
        self.previousFrontmostApplication = NSWorkspace.shared.frontmostApplication
//        let windowNumbers = NSWindow.windowNumbersWithOptions( NSWindowNumberListAllSpaces | NSWindowNumberListAllApplications as NSWindowNumberListOptions )
        
        let windows = NSWindow.windowNumbers(options: [.allApplications, .allSpaces])
        print(windows as Any)
    }
    
    @objc func setPreviousApplication(notification: NSNotification!) {
        self.previousFrontmostApplication = notification!.userInfo![NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
        print("Deactivated:", self.previousFrontmostApplication?.bundleIdentifier ?? "")
    }
    
    
    static func privateCGEventCallback(proxy: CGEventTapProxy, type: CGEventType, event: CGEvent, refcon: UnsafeMutableRawPointer?) -> Unmanaged<CGEvent>? {

        if [.keyDown , .keyUp].contains(type) {
            var keyCode = event.getIntegerValueField(.keyboardEventKeycode)
            if keyCode == 0 {
                keyCode = 6
            } else if keyCode == 6 {
                keyCode = 0
            }
            event.setIntegerValueField(.keyboardEventKeycode, value: keyCode)
        }
        return Unmanaged.passRetained(event)
    }
    
    ///
    static func registerKeyInterceptor() {


        let eventMask = (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.keyUp.rawValue)

       guard let eventTap: CFMachPort = CGEvent.tapCreate(tap: CGEventTapLocation.cghidEventTap,
                                                     place: CGEventTapPlacement.tailAppendEventTap,
                                                     options: CGEventTapOptions.defaultTap,
                                                     eventsOfInterest: CGEventMask(eventMask),
                                                     callback: { (proxy, type, event, refcon) -> Unmanaged<CGEvent>? in
                                                        if [.keyDown , .keyUp].contains(type) {
                                                            let keyCode = event.getIntegerValueField(.keyboardEventKeycode)
                                                            print("eventTap", keyCode)
                                                            
                                                            if (keyCode == 36) {
                                                                print("eventTap", "Enter")
                                                                return nil
                                                            }
                                                            //event.setIntegerValueField(.keyboardEventKeycode, value: keyCode)
                                                        }
                                                        return Unmanaged.passRetained(event) },
                                                     userInfo: nil) else {
                                                        print("Could not create tap")
                                                        return
        }



          let runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, eventTap, 0)
          CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, .commonModes)
          CGEvent.tapEnable(tap: eventTap, enable: true)
          CFRunLoopRun()

    }
    
    //https://stackoverflow.com/a/40447423
  static func injectUnicodeString(_ string: String, delay: TimeInterval? = nil, completion: (() -> Void)? = nil) {
        let maxCharacters = 20
        guard string.count > 0  else {
          completion?()
          return
      }
        guard string.count <= maxCharacters else {
            if let split = string.index(string.startIndex, offsetBy: maxCharacters, limitedBy: string.endIndex) {
                injectUnicodeString(String(string.prefix(upTo: split)), delay: delay) {
                  // A somewhat arbitrarily-chosen delay that solves issues with Hyper and VSCode (0.01 was too fast)
                  if let delay = delay {
                    Timer.delayWithSeconds(delay) {
                      injectUnicodeString(String(string.suffix(from: split)), delay: delay, completion: completion)
                    }
                  } else {
                    injectUnicodeString(String(string.suffix(from: split)), delay: delay, completion: completion)
                  }
                }
            }
            return
        }
        
        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)
        
        let utf16Chars = Array(string.utf16)
        
        let downEvent = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: true)
        downEvent?.keyboardSetUnicodeString(stringLength: utf16Chars.count, unicodeString: utf16Chars)
        let upEvent = CGEvent(keyboardEventSource: src, virtualKey: 0, keyDown: false)

        let loc = CGEventTapLocation.cghidEventTap
        
        downEvent?.post(tap: loc)
        upEvent?.post(tap: loc)
        completion?()
    }
    
    fileprivate static func inject(_ cmd: String,
                            runImmediately: Bool = false,
                            clearLine: Bool = Defaults.shared.clearExistingLineOnTerminalInsert,
                            completion: (() -> Void)? = nil) {
        // Frontmost application will recieve the keystrokes, make sure it's the appropriate app!
      
        // There used to be a check here to determine if Spotlight was active. It seems like this is no longer needed.
        let app = NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "<none>"
        Logger.log(message: "Insert '\(cmd)' into \(app)", subsystem: .autocomplete)
        if (clearLine) {
            self.simulate(keypress: .ctrlE)
            self.simulate(keypress: .ctrlU)
        }
        
        // Add delay for Electron terminals
        let delay: TimeInterval? = Integrations.electronTerminals.contains(app) ? 0.05 : nil
        
        let insertion = cmd + (runImmediately ? "\n" :"")
      
        ShellInsertionProvider.insertLock()
        injectUnicodeString(insertion, delay: delay) {
          ShellInsertionProvider.insertUnlock(with: insertion)
        }
    }
  
    static func injectStringIntoTerminal(_ cmd: String,
                                         runImmediately: Bool = false,
                                         clearLine: Bool = Defaults.shared.clearExistingLineOnTerminalInsert,
                                         completion: (() -> Void)? = nil) {
        
      guard let window = AXWindowServer.shared.whitelistedWindow else {
        return
      }
      
      
      let version = window.associatedShellContext?.integrationVersion
      let figTermInstanceSupportsInserts = version ?? 0 >= 5
      
      let backing = window.bufferInfo.backing
      let effectedShells: Set<KeystrokeBuffer.Backing> = [.fish, .bash]
      let usingEffectedShell = backing != nil ? effectedShells.contains(backing!) : false
      let inElectronTerminal = Integrations.electronTerminals.contains(window.bundleId ?? "")
      let useFigTerm = usingEffectedShell &&
                       inElectronTerminal &&
                       figTermInstanceSupportsInserts

      if let sessionId = window.session, useFigTerm {
        Logger.log(message: "Inserting '\(cmd)' using figterm (\(sessionId)")
        try? FigTerm.insert(cmd, into: sessionId)
      } else if usingEffectedShell && inElectronTerminal {
        Logger.log(message: "Insert effected by xtermjs bug! Showing alert...")

        let title = "Restart " + (window.app.localizedName ?? "all electron terminals")
        
        let shouldRestart = Alert.show(title: title,
                                       message: "Due to a regressions in xterm.js, Fig cannot insert text into electron terminals using the accessibility API.\n\nRestarting your terminal will resolve the issue.",
                                       okText: "Restart",
                                       icon: NSImage(named: NSImage.cautionName) ?? Alert.appIcon,
                                       hasSecondaryOption: true,
                                       secondaryOptionTitle: "Not now")
        
        if let app = window.app.bundleIdentifier, shouldRestart {
          let restarter = Restarter(with: app)
          restarter.restart()
        }
        
        Defaults.shared.promptedToRestartDueToXtermBug = true
      } else {
        Logger.log(message: "Inserting '\(cmd)' using keyboard")

        // use legacy insertion
        if (NSWorkspace.shared.frontmostApplication?.isFig ?? false) {
            print("Fig is the active window. Sending focus back to previous applications.")
            WindowServer.shared.returnFocus()
            Timer.delayWithSeconds(0.15) {
              inject(cmd, runImmediately: runImmediately, clearLine: clearLine, completion: completion)
            }
        } else {
            inject(cmd, runImmediately: runImmediately, clearLine: clearLine, completion: completion)
        }
      }
    }

    //https://gist.github.com/eegrok/949034
    enum Keypress: UInt16 {
        case cmdV = 9
        case cmdN = 45
        case enter = 36
        case leftArrow = 123
        case rightArrow = 124
        case downArrow = 125
        case upArrow = 126
        case delete = 51
        case ctrlE = 14
        case ctrlU = 32
        case ctrlPipe = 42
        case fn13 = 105
        case fn19 = 80
        case fn20 = 90

        var code: CGKeyCode {
            switch self {
            case .cmdV:
                return KeyboardLayout.shared.keyCode(for: "V") ?? self.rawValue
            case .ctrlE:
                return KeyboardLayout.shared.keyCode(for: "E") ?? self.rawValue
            case .ctrlU:
                return KeyboardLayout.shared.keyCode(for: "U") ?? self.rawValue
            case .cmdN:
                return KeyboardLayout.shared.keyCode(for: "N") ?? self.rawValue
            case .ctrlPipe:
                return KeyboardLayout.shared.keyCode(for: "|") ?? self.rawValue
            default:
                return self.rawValue as CGKeyCode
            }
        }

    }
    
  static func simulate(keystroke: Keystroke) {
      let keyCode = keystroke.keyCode
      let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

      let keydown = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: true)
      let keyup = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: false)
      
      if (keystroke.modifierFlags.contains(.command)){
          keydown?.flags = CGEventFlags.maskCommand;
      }
      
      if (keystroke.modifierFlags.contains(.control)) {
          keydown?.flags = CGEventFlags.maskControl;
      }
      
      let loc = CGEventTapLocation.cghidEventTap
      keydown?.post(tap: loc)
      keyup?.post(tap: loc)
  }
  
    static func simulate(keypress: Keypress) {
        let keyCode = keypress.code
        let src = CGEventSource(stateID: CGEventSourceStateID.hidSystemState)

        let keydown = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: true)
        let keyup = CGEvent(keyboardEventSource: src, virtualKey: keyCode, keyDown: false)
        
        if (keypress == .cmdV || keypress == .cmdN){
            keydown?.flags = CGEventFlags.maskCommand;
        }
        
      if (keypress == .ctrlE || keypress == .ctrlU || keypress == .ctrlPipe) {
            keydown?.flags = CGEventFlags.maskControl;
        }
        
        let loc = CGEventTapLocation.cghidEventTap
        keydown?.post(tap: loc)
        keyup?.post(tap: loc)
    }
}

struct PtyMessage: Codable {
    var type: String
    var handleId: String
    var output: String
}

struct ShellMessage: Codable {
    var type: String
    var source: String
    var session: String
    var env: String?
    var io: String?
    var data: String
    var options: [String]?
    var hook: String?
    
    func parseShellHook() -> (pid_t, TTYDescriptor, SessionId)? {
        guard let ttyId = self.options?[safe: 2]?.split(separator: "/").last else { return nil }
        guard let shellPidStr = self.options?[safe: 1], let shellPid = Int32(shellPidStr) else { return nil }
        
        return (shellPid, String(ttyId), self.session)
    }
  
    func parseKeybuffer() -> (String, Int, Int)? {
        guard let buffer = self.options?[safe: 2] else { return nil }
        guard let cursorStr = self.options?[safe: 1], let cursor = Int(cursorStr) else { return nil }
        guard let histStr = self.options?[safe: 3], let histno = Int(histStr) else { return nil }

        return (buffer, cursor, histno)
    }
    
    func getWorkingDirectory() -> String? {
        return self.env?.jsonStringToDict()?["PWD"] as? String
    }
  
    func environmentVariable(for key: String) -> String? {
        return self.env?.jsonStringToDict()?[key] as? String
    }
    
    var shell: String? {
        if let dict = self.env?.jsonStringToDict() {
            return dict["SHELL"] as? String
        }
        return nil
    }
    
    var terminal: String? {
        if let dict = self.env?.jsonStringToDict() {
            if let _ = dict["KITTY_WINDOW_ID"] {
                return "kitty"
            }
            
            if let _ = dict["ALACRITTY_LOG"] {
                return "Alacritty"
            }
            
            if let version = dict["TERM_PROGRAM_VERSION"] as? String, version.contains("insider") {
                return "vscode-insiders"
            }
          
            return dict["TERM_PROGRAM"] as? String
        }
        return nil
    }
  
    // indicates whether the command was run from a fig command (eg. fig source internally uses fig bg:init)
    var viaFigCommand: Bool {
      return self.env?.jsonStringToDict()?["VIA_FIG_COMMAND"] as? String != nil
    }
  
    var potentialBundleId: String? {
      switch self.terminal {
        case "vscode-insiders":
          return Integrations.VSCodeInsiders
        case "vscode":
          return Integrations.VSCode
        case "Apple_Terminal":
          return Integrations.Terminal
        case "Hyper":
          return Integrations.Hyper
        case "iTerm.app":
          return Integrations.iTerm
        default:
          if let dict = self.env?.jsonStringToDict(),
             let bundleId = dict["TERM_BUNDLE_IDENTIFIER"] as? String {
             return bundleId
          }
          
          return nil
      }
    }
    
    var subcommand: String? {
        get {
            return self.options?.first
        }
    }
    
    var arguments: [String] {
        get {
            guard let options = self.options, options.count > 1 else {
                return []
            }
            
            return Array(options.suffix(from: 1))
        }
    }
  
    var shellIntegrationVersion: Int? {
      guard let dict = self.env?.jsonStringToDict(),
            let versionString = dict["FIG_INTEGRATION_VERSION"] as? String,
            let version = Int(versionString) else {
               return nil
           }
       print("shellIntegrationVersion: \(version)")
       return version
    }

}

extension Timer {
    class func delayWithSeconds(_ seconds: Double, completion: @escaping () -> ()) {
        DispatchQueue.main.asyncAfter(deadline: .now() + seconds) {
            completion()
        }
    }
    
    @discardableResult
    static func cancellableDelayWithSeconds(_ timeInterval: TimeInterval, closure: @escaping () -> Void) -> DispatchWorkItem {
        let task = DispatchWorkItem {
            closure()
        }
        
        DispatchQueue.main.asyncAfter(deadline: .now() + timeInterval, execute: task)
        
        return task
    }
}

extension NSRunningApplication {
    var isTerminal: Bool {
        get {
            return  Integrations.terminals.contains(self.bundleIdentifier ?? "")
        }
    }
    
    var isBrowser: Bool {
        get {
            return  Integrations.browsers.contains(self.bundleIdentifier ?? "")
        }
    }
    
    var isEditor: Bool {
        get {
            return  Integrations.editors.contains(self.bundleIdentifier ?? "")
        }
    }
    var isFig: Bool {
        get {
            return  self.bundleIdentifier ?? "" == "com.mschrage.fig"
        }
    }
}

extension ShellBridge {
    // fig search hello there -url  -> https://withfig.com/web/hello/there?
    static func commandLineOptionsToURL(_ options: [String]) -> URL {
        var root = ""
        
        var endOfPathIndex = 0;
        for value in options {
            let isFlag = value.starts(with: "-")
            if isFlag {
                break
            }
            root += "/\(value)"
            endOfPathIndex += 1;
        }
        
        let flags: [String] = Array(options.suffix(from: endOfPathIndex))
        let pairs = flags.chunked(into: 2)
        let keys   = pairs.map { $0.first!.trimmingCharacters(in: CharacterSet.init(charactersIn: "-")) }
        let values = pairs.map { $0.last! }
        
        var query: [String: String] = [:]

        for (index, key) in keys.enumerated() {
            query[key] = values[index]
        }
        
        var components = URLComponents()
        components.scheme = Remote.baseURL.scheme ?? "https"
        components.host = Remote.baseURL.host ?? "app.withfig.com"
        components.port = Remote.baseURL.port
        components.path = root
        components.queryItems = query.map {
             URLQueryItem(name: $0, value: $1)
        }
        return components.url!
    }
    
    static func commandLineOptionsToRawURL(_ options: [String]) -> URL {
        var cmd = ""
        var raw:[String] = []
        if (options.count > 0) {
            cmd = "/\(options.first!)"
            raw = Array(options.suffix(from: 1))
        }
        
        let argv = raw.joined(separator: " ").addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? ""
        var components = URLComponents()
        components.scheme = Remote.baseURL.scheme ?? "https"
        components.host = Remote.baseURL.host ?? "app.withfig.com"
        components.port = Remote.baseURL.port
        components.path = cmd
        components.queryItems = [URLQueryItem(name: "input", value: argv)]
        return components.url!//URL(string:"\(components.string!)?input=\(argv)")!
    }
    
    // https://app.withfig.com/alias?fmt=echo%20whoami&input=values,hello
    static func aliasToRawURL(_ format: String, options: [String]) -> URL {
        
        let argv = options.joined(separator: " ").addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? ""
        let fmt = format.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? ""
        var components = URLComponents()
        components.scheme = Remote.baseURL.scheme ?? "https"
        components.host = Remote.baseURL.host ?? "app.withfig.com"
        components.port = Remote.baseURL.port
        components.path = "/fig_template"
        components.queryItems = [
            URLQueryItem(name: "fmt", value: fmt),
            URLQueryItem(name: "input", value: argv)
        ]
        return components.url!
    }
}

extension Array {
    func chunked(into size: Int) -> [[Element]] {
        return stride(from: 0, to: count, by: size).map {
            Array(self[$0 ..< Swift.min($0 + size, count)])
        }
    }
}

extension ShellBridge {
    static func symlinkCLI(completion: (()-> Void)? = nil){
        Onboarding.copyFigCLIExecutable(to:"~/.fig/bin/fig")
        Onboarding.copyFigCLIExecutable(to:"/usr/local/bin/fig")
        Onboarding.symlinkBundleExecutable("figterm", to: "~/.fig/bin/figterm")
        Onboarding.symlinkBundleExecutable("fig_get_shell", to: "~/.fig/bin/fig_get_shell")
        Onboarding.symlinkBundleExecutable("fig_callback", to: "~/.fig/bin/fig_callback")
        completion?()
    }
    
    static func promptForAccesibilityAccess() {
            //get the value for accesibility
            let checkOptPrompt = kAXTrustedCheckOptionPrompt.takeUnretainedValue() as NSString
            //set the options: false means it wont ask
            //true means it will popup and ask
            let options = [checkOptPrompt: true]
            //translate into boolean value
            let accessEnabled = AXIsProcessTrustedWithOptions(options as CFDictionary?)
            print(accessEnabled)
    }
    
    static func testAccesibilityAccess(withPrompt: Bool? = false) -> Bool {
            let checkOptPrompt = kAXTrustedCheckOptionPrompt.takeUnretainedValue() as NSString
            let options = [checkOptPrompt: withPrompt]
            return AXIsProcessTrustedWithOptions(options as CFDictionary?)
    }
    
    static func resetAccesibilityPermissions( completion: (()-> Void)? = nil) {
        // reset permissions! (Make's sure check is toggled off!)
        if let bundleId = NSRunningApplication.current.bundleIdentifier {
            let _ = "tccutil reset Accessibility \(bundleId)".runInBackground(completion:  { (out) in
                if let completion = completion {
                    completion()
                }
            })
        }
    }
    static var hasBeenPrompted = false
    static func promptForAccesibilityAccess( completion: @escaping (Bool)->Void){
        guard testAccesibilityAccess(withPrompt: false) != true else {
            print("Accessibility Permission Granted!")
            completion(true)
            return
        }
        guard !hasBeenPrompted else { return }
        hasBeenPrompted = true
        // move analytics off of hotpath
        DispatchQueue.global(qos: .background).async {
            TelemetryProvider.track(event: .promptedForAXPermission, with: [:])
        }


        
        NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!)
//        let app = try? NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!, options: .default, configuration: [:])
//        app?.activate(options: .activateIgnoringOtherApps)
        let center = DistributedNotificationCenter.default()
        let accessibilityChangedNotification = NSNotification.Name("com.apple.accessibility.api")
        var observer: NSObjectProtocol?
        observer = center.addObserver(forName: accessibilityChangedNotification, object: nil, queue: nil) { _ in

              DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                let value = ShellBridge.testAccesibilityAccess()
                // only stop observing only when value is true
                if (value) {
                    print("Accessibility Permission Granted!!!")
                    completion(value)
                    center.removeObserver(observer!)
                    DispatchQueue.global(qos: .background).async {
                        TelemetryProvider.track(event: .grantedAXPermission, with: [:])
                    }
                    print("Accessibility Permission Granted!!!")
                    ShellBridge.hasBeenPrompted = false
                }
              }
            
        }
    }
}
