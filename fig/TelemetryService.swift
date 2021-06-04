//
//  TelemetryService.swift
//  fig
//
//  Created by Matt Schrage on 7/15/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

enum LocalTelemetryEvent: String {
    case terminalUsage = "terminalUsage"
    case keybufferEntered = "keybufferEntered"
    case showAutocompletePopup = "showAutocompletePopup"
    case insertViaAutocomplete = "insertViaAutocomplete"
}

// Persists, aggregates and posts local telemetry events
protocol LocalTelemetryService {
    static func store(event: LocalTelemetryEvent, with increment: Int, date: Date)
    static func flush(eventsFor date: Date)
    static func flushAll(includingCurrentDay: Bool)
    static func register()
}

protocol TelemetryService {
    static func obscure(_ input: String) -> String
    static func track(event: TelemetryEvent, with payload: Dictionary<String, String>, completion: ((Data?, URLResponse?, Error?) -> Void)?)
}

enum TelemetryEvent: String {
    case ranCommand = "Ran CLI command"
    case selectedShortcut = "Selected a Shortcut"
    case viaJS = "Event via JS"
    case updatedApp = "Updated App"
    case promptedForAXPermission = "Prompted for AX Permission"
    case grantedAXPermission = "Granted AX Permission"
    case toggledAutocomplete = "Toggled Autocomplete"
    case toggledSidebar = "Toggled Sidebar"
    case quitApp = "Quit App"
    case viewDocs = "View Docs"
    case viewSupportForum = "View Support Forum"
    case joinSlack = "Join Slack"
    case sendFeedback = "Send Feedback"
    case dailyAggregates = "Aggregates"
    case firstTimeUser = "First Time User"
    case viaShell = "Event via Shell"
    case uninstallApp = "Uninstall App"
    case iTermSetup = "iTerm Setup"
    case launchedApp = "Launched App"
    case firstAutocompletePopup = "First Autocomplete Popup"
    case restartForOnboarding = "Restart for Shell Onboarding"
    case newWindowForOnboarding = "New Window for Shell Onboarding"
    case iTermSetupPrompted = "Prompted iTerm Setup"
    case showSecureInputEnabledAlert = "Show Secure Input Enabled Alert"
    case openSecureInputSupportPage = "Open Secure Input Support Page"
    case openedFigMenuIcon = "Opened Fig Menu Icon"
    case inviteAFriend = "Prompt to Invite"
    case runInstallationScript = "Running Installation Script"
    case telemetryToggled = "Toggled Telemetry"


}

class TelemetryProvider: TelemetryService {
    static func obscure(_ input: String) -> String {
        return String(input.map{ $0.isLetter ? "x" : $0 }.map{ $0.isNumber ? "0" : $0 })
    }
    
    static func track(event: TelemetryEvent, with properties: Dictionary<String, String>, completion: ((Data?, URLResponse?, Error?) -> Void)? = nil) {
        
        TelemetryProvider.track(event: event.rawValue, with: properties, completion: completion)
        
    }
    
    static func track(event: String, with properties: Dictionary<String, String>, needsPrefix prefix: String? = "prop_", completion: ((Data?, URLResponse?, Error?) -> Void)? = nil) {
        var body: Dictionary<String, String> = [:]
        
        if let prefix = prefix {
            body = TelemetryProvider.addPrefixToKeys(prefix: prefix, dict: properties)
        } else {
            body = properties
        }
        
        body = TelemetryProvider.addDefaultProperties(to: body)
        body["event"] = event
        body["userId"] = Defaults.uuid
      
        if Defaults.telemetryDisabled {
          let eventsToSendEvenWhenDisabled: [TelemetryEvent] = [.telemetryToggled]
          let sendEvent = eventsToSendEvenWhenDisabled.reduce(false, { (ignore, whitelistedEvent) -> Bool in
            return ignore || whitelistedEvent.rawValue == event
          })
          
          guard sendEvent else {
            print("telemetry: not sending event because telemetry is diabled")
            completion?(nil, nil, nil)
            return
          }
          
        }
        
        upload(to: "track", with: body, completion: completion)
    }
    
    static func identify(with traits: Dictionary<String, String>, needsPrefix prefix: String? = "trait_") {
        var body: Dictionary<String, String> = [:]
        if let prefix = prefix {
            body = TelemetryProvider.addPrefixToKeys(prefix: prefix, dict: traits)
        } else {
            body = traits
        }
        
        body["userId"] = Defaults.uuid
      
        if Defaults.telemetryDisabled {
            print("telemetry: not sending identification event because telemetry is diabled")
           return
        }

        upload(to: "identify", with: body)
    }
    
    static func alias(userId: String?) {
      
        if Defaults.telemetryDisabled {
            print("telemetry: not sending identification event because telemetry is diabled")
           return
        }
      
        upload(to: "alias", with: ["previousId" : Defaults.uuid, "userId": userId ?? ""])
    }
    
    fileprivate static func upload(to endpoint: String, with body:  Dictionary<String, String>, completion: ((Data?, URLResponse?, Error?) -> Void)? = nil) {
        guard let json = try? JSONSerialization.data(withJSONObject: body, options: .sortedKeys) else { return }
        print(json)
        var request = URLRequest(url: Remote.telemetryURL.appendingPathComponent(endpoint))
        request.httpMethod = "POST"
        request.httpBody = json
        request.setValue("application/json; charset=utf-8", forHTTPHeaderField: "Content-Type")

        let task = URLSession.shared.dataTask(with: request) { (data, res, err) in
            if let handler = completion {
                handler(data, res, err)
            }
        }

        task.resume()
    }
    
    fileprivate static func addPrefixToKeys(prefix: String, dict: Dictionary<String, String>) -> Dictionary<String, String> {
        
        return dict.reduce([:]) { (out, pair) -> Dictionary<String, String> in
            var new = out
            let (key, value) = pair
            new["\(prefix)\(key)"] = value
            return new
        }
    }
    
    fileprivate static func addDefaultProperties(to properties: Dictionary<String, String>, prefixedWith prefix: String = "prop_") -> Dictionary<String, String> {
        let email = Defaults.email ?? ""
        let domain = String(email.split(separator: "@").last ?? "unregistered")
        let os = ProcessInfo.processInfo.operatingSystemVersion

        return properties.merging([
                                    "\(prefix)domain" : domain,
                                    "\(prefix)email" : email,
                                    "\(prefix)version" : Defaults.version,
                                    "\(prefix)os" :  "\(os.majorVersion).\(os.minorVersion).\(os.patchVersion)",
                                    ]) { $1 }
    }
}

extension TelemetryProvider : LocalTelemetryService {
    static var terminalObserver:TerminalUsageObserver?
    static func register() {
        self.terminalObserver = TerminalUsageObserver()

        NotificationCenter.default.addObserver(self, selector:#selector(calendarDayDidChange), name:.NSCalendarDayChanged, object:nil)
        // flush previous events
        flushAll()
        
        // register other telemetry observers!
        NotificationCenter.default.addObserver(self, selector:#selector(lineAcceptedInKeystrokeBuffer), name: KeystrokeBuffer.lineAcceptedInKeystrokeBufferNotification, object:nil)
        NotificationCenter.default.addObserver(self, selector:#selector(insertionInTerminal), name: .insertCommandInTerminal, object:nil)
        NotificationCenter.default.addObserver(self, selector:#selector(showAutocompletePopup), name: NSNotification.Name("showAutocompletePopup"), object:nil)
        NotificationCenter.default.addObserver(self, selector:#selector(logTerminalUsage(_:)), name: TerminalUsageObserver.terminalApplicationLostFocusNotification, object:nil)
    }
    
    @objc fileprivate static func calendarDayDidChange() {
        Logger.log(message: "Calendar Day changed")
        self.flushAll()
    }
    
    // Local Telemetry Observers
    @objc fileprivate static func lineAcceptedInKeystrokeBuffer() {
        Logger.log(message: "lineAcceptedInKeystrokeBuffer")
        self.store(event: .keybufferEntered)
    }
    
    @objc fileprivate static func insertionInTerminal() {
        Logger.log(message: "insertionInTerminal")
        self.store(event: .insertViaAutocomplete)
    }
    
    @objc fileprivate static func showAutocompletePopup() {
        Logger.log(message: "showAutocompletePopup")
        self.store(event: .showAutocompletePopup)
        
        if (!Defaults.hasShownAutocompletePopover) {
            Defaults.hasShownAutocompletePopover = true
            TelemetryProvider.track(event: .firstAutocompletePopup, with: [:])
        }
    }

    @objc fileprivate static func logTerminalUsage(_ notification: Notification) {
        Logger.log(message: "logTerminalUsage")
        if let time = notification.object as? TimeInterval {
            self.store(event: .terminalUsage, with: Int(time))
        }
    }
    
    static func flushAll(includingCurrentDay: Bool = false) {
        let today = Date(timeIntervalSinceNow: 0).telemetryDayIdentifier
        self.pending.forEach {
            // exclude current day unless explictly pushing all events
            if (includingCurrentDay || $0 != today) {
                self.flush(eventsFor: $0)
            }
        }
    }
    
    static var pending: Set<TelemetryUTCDate> {
        return  Set(UserDefaults.standard.stringArray(forKey: "pendingTelemetryUpload") ?? [])
    }
    
    static func store(event: LocalTelemetryEvent, with increment: Int = 1, date: Date = Date(timeIntervalSinceNow: 0)) {
        DispatchQueue.global(qos: .utility).async {
            let dateIdentifier = date.telemetryDayIdentifier
            let key = "\(dateIdentifier)#\(event.rawValue)"
            let aggregate = UserDefaults.standard.integer(forKey: key)
            UserDefaults.standard.set(aggregate + increment, forKey: key)
            
            // update what dates have data to send
            var pending:Set<String> = Set(UserDefaults.standard.stringArray(forKey: "pendingTelemetryUpload") ?? [])
            Logger.log(message: pending.joined(separator: ","))
            
            pending.insert(dateIdentifier)
            UserDefaults.standard.set(Array(pending), forKey: "pendingTelemetryUpload")
        }
    }
    
    // send logged & aggregated events to server
    fileprivate static func flush (eventsFor dateIdentifier: TelemetryUTCDate) {
        let aggregatableEvents: Set<LocalTelemetryEvent> = [.insertViaAutocomplete, .keybufferEntered, .showAutocompletePopup, .terminalUsage]
        var keys: Set<String> = []
        let countsForDate = aggregatableEvents.map { (event) -> (LocalTelemetryEvent, Int) in
            let key = "\(dateIdentifier)#\(event.rawValue)"
          keys.insert(key)
          let total = UserDefaults.standard.integer(forKey: key)
          return (event, total)
        }
        var payload:[String:String] = countsForDate.reduce(into: [:], { (dict, pair) in
          let (event, count) = pair
          dict[event.rawValue] = "\(count)"
        })
        payload["date"] = dateIdentifier
        payload["telemetryDisabled"] = UserDefaults.standard.bool(forKey: "\(dateIdentifier)#telemetryDisabled") ? "true" : "false"
        print("aggregate:", countsForDate)
        // todo: add completion handler for success and failure
        // clean cache on success
        // reschedule on failure
        self.track(event: .dailyAggregates, with: payload
        ) { (data, res, error) in
          guard error == nil else {
              // Don't delete cached data, try to send later
              Logger.log(message: "Failed to flush events with error:\(error!.localizedDescription)")
              return
          }
          
          // delete cached data
          keys.forEach {
            Logger.log(message: "Delete telemetry for key: \($0)")
            UserDefaults.standard.removeObject(forKey: $0)
          }
          
          // remove date from [pendingUpload] store
          if let pending = UserDefaults.standard.stringArray(forKey: "pendingTelemetryUpload") {
              let filtered = pending.filter { $0 != dateIdentifier}
              UserDefaults.standard.set(filtered, forKey: "pendingTelemetryUpload")
          }
        }
    }
    
    static func flush(eventsFor date: Date) {
        let dateIdentifier = date.telemetryDayIdentifier
        self.flush(eventsFor: dateIdentifier)
    }
}

typealias TelemetryUTCDate = String
extension Date {
    var telemetryDayIdentifier: TelemetryUTCDate {
        let cal: Calendar = Calendar(identifier: .gregorian)
        let fmt = DateFormatter()
        fmt.dateFormat = " yyyy-MM-dd'T'HH:mm:ssZ"
        return fmt.string(from: cal.startOfDay(for: self))
    }
}

