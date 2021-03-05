//
//  Integrations.swift
//  fig
//
//  Created by Matt Schrage on 3/1/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Integrations {
    static let iTerm = "com.googlecode.iterm2"
    static let Terminal = "com.apple.Terminal"
    static let Hyper = "co.zeit.hyper"
    static let VSCode = "com.microsoft.VSCode"
  
    static let terminals: Set = ["com.googlecode.iterm2",
                                 "com.apple.Terminal",
                                 "io.alacritty",
                                 "co.zeit.hyper",
                                "net.kovidgoyal.kitty"]
    static let browsers:  Set = ["com.google.Chrome"]
    static let editors:   Set = ["com.apple.dt.Xcode",
                                 "com.sublimetext.3",
                                 "com.microsoft.VSCode"]
    static let nativeTerminals: Set = ["com.googlecode.iterm2",
                                       "com.apple.Terminal" ]
    static let searchBarApps: Set = ["com.apple.Spotlight",
                                     "com.runningwithcrayons.Alfred",
                                     "com.raycast.macos"]
  
    static let electronTerminals: Set = ["co.zeit.hyper",
                                        "com.microsoft.VSCode"]
    static let terminalsWhereAutocompleteShouldAppear: Set = nativeTerminals.union(electronTerminals)
  
    static var allowed: Set<String> {
        
        get {
            if let allowed = UserDefaults.standard.string(forKey: "allowedApps") {
                return Set(allowed.split(separator: ",").map({ String($0)}))
            } else {
                return []
            }
        }
    }
    
    static var blocked: Set<String> {
        get {
           if let allowed = UserDefaults.standard.string(forKey: "blockedApps") {
               return Set(allowed.split(separator: ",").map({ String($0)}))
           } else {
               return []
           }
       }
    }
    static var whitelist: Set<String> {
        get {
            return Integrations.terminals
            .union(Integrations.allowed)
      .subtracting(Integrations.blocked)
        }
    }
  static let providers: [String: IntegrationProvider.Type] =
                        [Integrations.iTerm : iTermTabIntegration.self,
                          Integrations.Hyper : HyperIntegration.self,
                          Integrations.VSCode : VSCodeIntegration.self]
}

protocol IntegrationProvider {
  static func install(withRestart: Bool, inBackground: Bool, completion: (() -> Void)?)
  static var isInstalled: Bool { get }
  static func promptToInstall(completion: (()->Void)?)
}
