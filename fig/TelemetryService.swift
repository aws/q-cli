//
//  TelemetryService.swift
//  fig
//
//  Created by Matt Schrage on 7/15/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

protocol TelemetryService {
    static func obscure(_ input: String) -> String
    static func post(event: TelemetryEvent, with payload: Dictionary<String, String>, completion: (() -> Void)?)
}

enum TelemetryEvent: String {
    case ranCommand = "Ran CLI command"
    case selectedShortcut = "Selected a Shortcut"
    case viaJS = "Event via JS"
    case updatedApp = "Updated App"
    case promptedForAXPermission = "Prompted for AX Permission"
    case toggledAutocomplete = "Toggled Autocomplete"
    case toggledSidebar = "Toggled Sidebar"
    case quitApp = "Quit App"

}

class TelemetryProvider: TelemetryService {
    static func obscure(_ input: String) -> String {
        return String(input.map{ $0.isLetter ? "x" : $0 }.map{ $0.isNumber ? "0" : $0 })
    }
    
    static func post(event: TelemetryEvent, with payload: Dictionary<String, String>, completion: (() -> Void)? = nil) {
        
        guard Defaults.isProduction else {
            print("Not logging CLI usage when not in production.")
            return
        }
        
        let email = Defaults.email ?? ""
        let domain = String(email.split(separator: "@").last ?? "unregistered")
        let os = ProcessInfo.processInfo.operatingSystemVersion
        // add UUID to dict (overwritting 'anonymized_id', 'questions?' and 'version', 'domain' in payload if they exist)
        let eventType = (event == .viaJS) ? payload["name"] ??  event.rawValue : event.rawValue
        let final = payload.merging(["anonymized_id" :  Defaults.uuid,
                                     "questions?" : "\n\nFig collects anonymized usage information - this is not tied to any personally identifiable data. \n\nIf you have more questions go to https://withfig.com/telemetry or email me at matt@withfig.com\n",
                                     "domain" : domain,
                                     "version" : Defaults.version,
                                     "os" :  "\(os.majorVersion).\(os.minorVersion).\(os.patchVersion)",
                                     "event" : eventType ]) { $1 }
        
        guard let json = try? JSONSerialization.data(withJSONObject: final, options: .sortedKeys) else { return }
        print(json)
        var request = URLRequest(url: Remote.baseURL.appendingPathComponent("anonymized_cli_usage"))
        request.httpMethod = "POST"
        request.httpBody = json
        request.setValue("application/json; charset=utf-8", forHTTPHeaderField: "Content-Type")

        //URLSession.shared.dataTask(with: request)
        let task = URLSession.shared.dataTask(with: request) { (data, res, err) in
            if let handler = completion {
                handler()
            }
        }

        task.resume()
        
       
    }
    
    
}
