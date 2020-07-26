//
//  Defaults.swift
//  fig
//
//  Created by Matt Schrage on 7/8/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

enum Build: String {
    case production = "prod"
    case staging = "staging"
    case dev = "dev"
}

class Defaults {
    static var isProduction: Bool {
        return Defaults.build == .production
    }
    
    static var isStaging: Bool {
        return UserDefaults.standard.string(forKey: "build") == "staging"
    }
    
    static var build: Build {
        get {
            return Build(rawValue: UserDefaults.standard.string(forKey: "build") ?? "") ?? .production
        }
        set(value) {
            UserDefaults.standard.set(value.rawValue, forKey: "build")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var uuid: String {
        guard let uuid = UserDefaults.standard.string(forKey: "uuid") else {
            let uuid = UUID().uuidString
            UserDefaults.standard.set(uuid, forKey: "uuid")
            UserDefaults.standard.synchronize()
            return uuid
        }
        
        return uuid
    }
    
    static var showSidebar:Bool {
        get {
            return UserDefaults.standard.string(forKey: "sidebar") != "hidden"
        }
        
        set(value) {
            UserDefaults.standard.set(value ? "visible" : "hidden", forKey: "sidebar")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var email: String? {
        get {
            return UserDefaults.standard.string(forKey: "userEmail")
        }
        
        set(email) {
            UserDefaults.standard.set(email, forKey: "userEmail")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var version: String {
         return Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "<unknown>"
    }
    
    static var automaticallyLaunchWebAppsInDetachedWindow: Bool {
        get {
            return UserDefaults.standard.string(forKey: "undockWebApps") == "true"
        }

        set(flag) {
            UserDefaults.standard.set(flag ? "true" : "false", forKey: "undockWebApps")
            UserDefaults.standard.synchronize()
        }
    }
}
