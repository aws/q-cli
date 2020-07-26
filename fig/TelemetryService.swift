//
//  TelemetryService.swift
//  fig
//
//  Created by Matt Schrage on 7/15/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

protocol TelemetryService {
    static func obscure(_ input: String) -> String
    static func post(event: TelemetryEvent, with payload: Dictionary<String, String>)
}

enum TelemetryEvent: String {
    case ranCommand = "Ran CLI command"
    case selectedShortcut = "Selected a Shortcut"

}

class TelemetryProvider: TelemetryService {
    static func obscure(_ input: String) -> String {
        return String(input.map{ $0.isLetter ? "x" : $0 }.map{ $0.isNumber ? "0" : $0 })
    }
    
    static func post(event: TelemetryEvent, with payload: Dictionary<String, String>) {
        
        guard Defaults.isProduction else {
            print("Not logging CLI usage when not in production.")
            return
        }
        
        let email = Defaults.email ?? ""
        let domain = String(email.split(separator: "@").last ?? "unregistered")
        // add UUID to dict (overwritting 'anonymized_id', 'questions?' and 'version', 'domain' in payload if they exist)
        let final = payload.merging(["anonymized_id" :  Defaults.uuid,
                                     "questions?" : "\n\nFig collects anonymized usage information - this is not tied to any personally identifiable data. \n\nIf you have more questions go to https://withfig.com/telemetry or email me at matt@withfig.com\n",
                                     "domain" : domain,
                                     "version" : Defaults.version,
                                     "event" : event.rawValue ]) { $1 }
        
        guard let json = try? JSONSerialization.data(withJSONObject: final, options: .sortedKeys) else { return }
        print(json)
        var request = URLRequest(url: Remote.baseURL.appendingPathComponent("anonymized_cli_usage"))
        request.httpMethod = "POST"
        request.httpBody = json
        request.setValue("application/json; charset=utf-8", forHTTPHeaderField: "Content-Type")

        
        let task = URLSession.shared.dataTask(with: request)

        task.resume()
    }
    
    
}
