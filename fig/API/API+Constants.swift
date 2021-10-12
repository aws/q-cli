//
//  API+Constants.swift
//  fig
//
//  Created by Matt Schrage on 10/6/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import WebKit

extension API {
    static let constants: Dictionary<String, String?> = [
             "version" : Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String,
               "build" : Bundle.main.infoDictionary?["CFBundleVersion"] as? String,
                 "cli" : Bundle.main.path(forAuxiliaryExecutable: "figcli"),
          "bundlePath" : Bundle.main.bundlePath,
              "remote" : Remote.baseURL.absoluteString,
                "home" : NSHomeDirectory(),
                "user" : NSUserName(),
         "defaultPath" : PseudoTerminal.defaultMacOSPath,
 "jsonMessageRecieved" : API.Encoding.json.eventName,
  "jsonMessageHandler" : API.Encoding.json.webkitMessageHandler,
"protoMessageRecieved" : API.Encoding.binary.eventName,
 "protoMessageHandler" : API.Encoding.binary.webkitMessageHandler,
              "themes" : {
                 let files = (try? FileManager.default.contentsOfDirectory(atPath: NSHomeDirectory() + "/.fig/themes")) ?? []
                 let themes = [ "dark", "light"] + files.map { String($0.split(separator: ".")[0]) }.sorted()
                 return themes.joined(separator: "\n")
             }(),
    ]
    
    
    // Must be included at top level and may have been renamed (eg. fig.KEY rather than fig.constants.KEY)
    fileprivate static let legacyConstants = [
        "version": "appversion",
        "cli": "clipath",
        "remote" : "remoteURL",
        "build" : "build"
    ]
    
    static func declareConstants() -> String {
        
        let payload = constants.map { (key, value) -> String in
            var script = ""
            
            if let value = value {

                script += "fig.constants.\(key) = `\(value)`;"

                if let legacy = legacyConstants[key] {
                    script += "\n"
                    script += "fig.\(legacy) = `\(value)`;"
                }
            }
            
            return script

        }.joined(separator: "\n")
        
        let script =
        """

        if (!window.fig) {
            window.fig = {}
        }
        
        if (!window.fig.constants) {
            fig.constants = {}
        }
        
        \(payload)
        
        console.log("[fig] declaring constants...")
        """
        
        return script
    }
    
    
}
