//
//  API+App.swift
//  fig
//
//  Created by Matt Schrage on 10/14/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import WebKit
import FigAPIBindings

struct FigApp {
    let identifier: String?
    var folder: URL? {
        guard let identifier = identifier else {
            return nil
        }
        return FigApp.appsDirectory.appendingPathComponent(identifier, isDirectory: true)
    }
    
    func writeToAppDirectory(_ name: String, data: Data?) throws {
        guard let data = data else { return }
        guard self.identifier != nil else {
            return APIError.generic(message: "App identifier is not set.")
        }
        try? FileManager.default.createDirectory(at: folder!,
                                                 withIntermediateDirectories: true,
                                                 attributes: nil)
        
        try data.write(to: folder!.appendingPathComponent(name))
    }
}
extension FigApp {
    static let appsDirectory = URL(fileURLWithPath: NSHomeDirectory() + "/.fig/apps")
    static func updateProperties(_ request: Fig_UpdateApplicationPropertiesRequest, for app: FigApp) throws -> Bool {
        
        if request.hasInterceptBoundKeystrokes {
            KeypressProvider.shared.setRedirectsEnabled(value: request.interceptBoundKeystrokes)
        }
        
        let actions = try request.actions.map({ action in
            try action.jsonString()
        })
        
        let actionsList = "[" + actions.joined(separator: ",\n") + "]"
        
        try app.writeToAppDirectory("actions.json",
                                    data: actionsList.data(using: .utf8))
        
        return true
        
    }
}


extension WKWebView {

    var appIdentifier: String? {
        guard let url = self.url else {
            return nil
        }
        
        // exception for legacy .../autocomplete/v6
        guard url.pathComponents[safe: url.pathComponents.count - 2] != "autocomplete" else {
            return "autocomplete"
        }
        
        // Otherwise, the appIdentifier is the final path component in the URL
        let identifier = url.pathComponents.last
        return identifier == "/" ? nil : identifier
        
    }
}
