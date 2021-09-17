//
//  InstallationStatus.swift
//  fig
//
//  Created by Matt Schrage on 9/14/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

enum InstallationDependency: String, Codable {
    case applicationRestart
    case inputMethodActivation
}

enum InstallationStatus: Equatable {
    case applicationNotInstalled  // target app not installed,
    case unattempted    // we have not tried to install the integration
    case pending(event: InstallationDependency)
    case installed      // integration has been successfully installed
    
    case failed(error: String, supportURL: URL? = nil)
    
    func encoded() -> Data? {
        let encoder = JSONEncoder()
        return try? encoder.encode(self)
    }
    
    init?(data: Data?) {
        guard let data = data else {
            return nil
        }
        
        let decoder = JSONDecoder()

        guard let status = try? decoder.decode(InstallationStatus.self, from: data) else {
            return nil
        }
        
        self = status
    }
    
    //
    func staticallyVerifiable() -> Bool {
        return ![InstallationStatus.pending(event: .applicationRestart)].contains(self)
    }
    
    var description: String {
        switch self {
        case .applicationNotInstalled:
            return "application is not present."
        case .installed:
            return "installed!"
        case .unattempted:
            return "unattempted"
        case .failed(let error, let supportURL):
            return error + ((supportURL != nil) ? "\(supportURL!)" : "")
        case .pending(event: .applicationRestart):
            return "pending application restart"
        case .pending(event: .inputMethodActivation):
            return "pending input method activation"

        }
    }
}

extension InstallationStatus: Codable {
    enum CodingKeys: CodingKey {
        case unattempted, pending, installed, failed, applicationNotInstalled
    }
    
    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let key = container.allKeys.first
        
        switch key {
        case .failed:
            var nestedContainer = try container.nestedUnkeyedContainer(forKey: .failed)
            let error = try nestedContainer.decode(String.self)
            let supportURL = try nestedContainer.decode(URL?.self)
            self = .failed(error: error,
                           supportURL: supportURL)
        case .pending:
            var nestedContainer = try container.nestedUnkeyedContainer(forKey: .pending)
            let dependency = try nestedContainer.decode(InstallationDependency.self)
            self = .pending(event: dependency)
        case .unattempted:
            self = .unattempted
        case .installed:
            self = .installed
        case .applicationNotInstalled:
            self = .applicationNotInstalled
        default:
            throw DecodingError.dataCorrupted(
                DecodingError.Context(
                    codingPath: container.codingPath,
                    debugDescription: "Unabled to decode enum."
                )
            )
        }
    }

    
    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        
        switch self {
        case .unattempted:
            try container.encode(true, forKey: .unattempted)
        case .installed:
            try container.encode(true, forKey: .installed)
        case .applicationNotInstalled:
            try container.encode(true, forKey: .applicationNotInstalled)
        case .pending(let dependency):
            var nestedContainer = container.nestedUnkeyedContainer(forKey: .pending)
            try nestedContainer.encode(dependency)

        case .failed(let error, let supportURL):
            var nestedContainer = container.nestedUnkeyedContainer(forKey: .failed)
            try nestedContainer.encode(error)
            try nestedContainer.encode(supportURL)
        }
    }
    
}
