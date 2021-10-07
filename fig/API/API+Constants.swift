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
         "remote" : Remote.baseURL.absoluteString,
           "home" : NSHomeDirectory(),
           "user" : NSUserName(),
    "defaultPath" : PseudoTerminal.defaultMacOSPath,
         "themes" : {
            let files = (try? FileManager.default.contentsOfDirectory(atPath: NSHomeDirectory() + "/.fig/themes")) ?? []
            let themes = [ "dark", "light"] + files.map { String($0.split(separator: ".")[0]) }.sorted()
            return themes.joined(separator: "\n")
        }(),
    ]
    
    // Incuded for backwards compatibilty
    fileprivate static let aliasedConstants = [
        "version": "appversion",
        "cli": "clipath",
        "remote" : "remoteURL"
    ]
    
    static func declareConstants() -> String {
        
        let payload = constants.map { (key, value) -> String in
            var script = ""
            
            if let value = value {
                script += "fig.\(key) = `\(value)`;"

                if let alias = aliasedConstants[key] {
                    script += "\n"
                    script += "fig.\(alias) = `\(value)`;"
                }
            }
            
            return script

        }.joined(separator: "\n")
        
        let script =
        """

        if (!window.fig) {
            window.fig = {}
        }
        
        \(payload)
        
        console.log("[fig] declaring constants")
        """
        
        return script
    }
    
    
}
