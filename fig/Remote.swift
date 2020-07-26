//
//  Server.swift
//  fig
//
//  Created by Matt Schrage on 7/8/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class Remote {
    static var productionURL: URL = URL(string: "https://app.withfig.com")!
    static var stagingURL: URL = URL(string: "https://frozen-basin-27070.herokuapp.com")!
    static var localhost: URL = URL(string: "http://localhost:3000")!

    static var baseURL: URL {
        switch Defaults.build {
        case .production:
            return productionURL
        case .staging:
            return stagingURL
        case .dev:
            return localhost
        }
    }
}
