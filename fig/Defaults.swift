//
//  Defaults.swift
//  fig
//
//  Created by Matt Schrage on 7/8/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class Defaults {
    static var isStaging: Bool {
        return UserDefaults.standard.string(forKey: "build") == "staging"
    }
}
