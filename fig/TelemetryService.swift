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
    static func post(event: TelemetryEvent, with payload: Dictionary<String, String>, completion: ((Data?, URLResponse?, Error?) -> Void)?)
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
    case joinSlack = "Join Slack"
    case sendFeedback = "Send Feedback"
    case dailyAggregates = "Aggregates"
    case firstTimeUser = "First Time User"
    case viaShell = "Event via Shell"
    case uninstallApp = "Uninstall App"
    case iTermSetup = "iTerm Setup"
    case launchedApp = "Launched App"


}

class TelemetryProvider: TelemetryService {
    static func obscure(_ input: String) -> String {
        return String(input.map{ $0.isLetter ? "x" : $0 }.map{ $0.isNumber ? "0" : $0 })
    }
    
    static func post(event: TelemetryEvent, with payload: Dictionary<String, String>, completion: ((Data?, URLResponse?, Error?) -> Void)? = nil) {
        
        guard Defaults.isProduction || Defaults.isStaging else {
            if let completion =  completion {
                completion(nil,nil,nil)
            }
            print("Not logging CLI usage when not in production.")
            return
        }
        
        let email = Defaults.email ?? ""
        let domain = String(email.split(separator: "@").last ?? "unregistered")
        let os = ProcessInfo.processInfo.operatingSystemVersion
        // add UUID to dict (overwritting 'anonymized_id', 'questions?' and 'version', 'domain' in payload if they exist)
        let properties = payload.reduce(into: [:]) { (dict, pair) in
            let (key, value) = pair
            dict["prop_\(key)"] = value
        }
        
        print("properties:", properties)
        
        let eventType = (event == .viaJS || event == .viaShell) ? payload["name"] ??  event.rawValue : event.rawValue
        let final = properties.merging(["anonymized_id" :  Defaults.uuid,
                                     "questions?" : "\n\nFig collects limited usage information to improve the product and detect bugs. \n\nIf you have more questions go to https://withfig.com/privacy or email the team at hello@withfig.com\n",
                                     "domain" : domain,
                                     "email" : email,
                                     "version" : Defaults.version,
                                     "os" :  "\(os.majorVersion).\(os.minorVersion).\(os.patchVersion)",
                                     "event" : eventType ]) { $1 }
        
        guard let json = try? JSONSerialization.data(withJSONObject: final, options: .sortedKeys) else { return }
        print(json)
        var request = URLRequest(url: Remote.baseURL.appendingPathComponent("anonymized_cli_usage"))
        request.httpMethod = "POST"
        request.httpBody = json
        request.setValue("application/json; charset=utf-8", forHTTPHeaderField: "Content-Type")
//        request.timeoutInterval = 15

        //URLSession.shared.dataTask(with: request)
        let task = URLSession.shared.dataTask(with: request) { (data, res, err) in
            if let handler = completion {
                handler(data, res, err)
            }
        }

        task.resume()
       
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
        print("aggregate:", countsForDate)
        // todo: add completion handler for success and failure
        // clean cache on success
        // reschedule on failure
        self.post(event: .dailyAggregates, with: payload
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

