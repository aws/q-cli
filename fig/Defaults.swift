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
            WindowManager.shared.createSidebar()
            WindowManager.shared.createAutocomplete()

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
    
    static var loggedIn: Bool {
        get {
            return UserDefaults(suiteName: "com.mschrage.fig.shared")?.bool(forKey: "loggedIn") ?? false
        }

        set(loggedIn) {
            UserDefaults(suiteName: "com.mschrage.fig.shared")?.set(loggedIn, forKey: "loggedIn")
            UserDefaults(suiteName: "com.mschrage.fig.shared")?.synchronize()
        }
    }
    static var domainToken: String? {
        get {
            return UserDefaults.standard.string(forKey: "domainToken")
        }
        
        set(token) {
            UserDefaults.standard.set(token, forKey: "domainToken")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var defaultActivePosition: CompanionWindow.OverlayPositioning {
        get {
             
            return  UserDefaults.standard.bool(forKey: "updatedDefaultActivePosition") ? CompanionWindow.OverlayPositioning(rawValue: UserDefaults.standard.integer(forKey: "defaultActivePosition")) ?? .outsideRight :  .outsideRight
        }
        
        set(id) {
            UserDefaults.standard.set(id.rawValue, forKey: "defaultActivePosition")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var shouldTrackTargetWindow: Bool {
        get {
            return
                UserDefaults.standard.bool(forKey: "shouldTrackTargetWindow")
        }
        
        set(token) {
            UserDefaults.standard.set(token, forKey: "shouldTrackTargetWindow")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var clearExistingLineOnTerminalInsert: Bool {
        get {
            return
                UserDefaults.standard.bool(forKey: "clearExistingLineOnTerminalInsert")
        }
        
        set(token) {
            UserDefaults.standard.set(token, forKey: "clearExistingLineOnTerminalInsert")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var triggerSidebarWithMouse: Bool {
        get {
            return
                UserDefaults.standard.bool(forKey: "triggerSidebarWithMouse")
        }
        
        set(token) {
            UserDefaults.standard.set(token, forKey: "triggerSidebarWithMouse")
            UserDefaults.standard.synchronize()
        }
    }
    
    static var useAutocomplete: Bool {
        get {
            return
                UserDefaults.standard.bool(forKey: "useAutocomplete")
        }
        
        set(flag) {
            UserDefaults.standard.set(flag, forKey: "useAutocomplete")
            UserDefaults.standard.synchronize()
        }

    }
    
    static var playSoundWhenContextIsLost: Bool {
           get {
               return
                   UserDefaults.standard.bool(forKey: "playSoundWhenContextIsLost")
           }
           
           set(flag) {
               UserDefaults.standard.set(flag, forKey: "playSoundWhenContextIsLost")
               UserDefaults.standard.synchronize()
           }

       }
    
    static var versionAtPreviousLaunch: String? {
        get {
            return  UserDefaults.standard.string(forKey: "versionAtPreviousLaunch")
        }
        
        set(version){
            UserDefaults.standard.set(version, forKey: "playSoundWhenContextIsLost")
            UserDefaults.standard.synchronize()
        }
    }
}
