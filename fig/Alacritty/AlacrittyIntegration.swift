//
//  AlacrittyIntegration.swift
//  fig
//
//  Created by Matt Schrage on 9/13/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class AlacrittyIntegration: TerminalIntegrationProvider {
    static let `default` = AlacrittyIntegration(bundleIdentifier: Integrations.Alacritty)

    override init(bundleIdentifier: String) {
        super.init(bundleIdentifier: bundleIdentifier)
        
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(inputMethodStatusDidChange),
                                               name: InputMethod.statusDidChange,
                                               object: nil)
    }
    
    deinit {
        NotificationCenter.default.removeObserver(self)
    }
    
    @objc func inputMethodStatusDidChange() {
        self.status = self._verifyInstallation()
    }
    
    func verifyInstallation() -> InstallationStatus {
        let inputMethodStatus = InputMethod.default.verifyInstallation()
        guard inputMethodStatus == .installed else {
            return .pending(event: .inputMethodActivation)
        }
        
        return .installed
    }
    
    func install() -> InstallationStatus {
        guard self.applicationIsInstalled else {
            return .applicationNotInstalled
        }
        
        if !InputMethod.default.isInstalled {
            let status = InputMethod.default.install()
            guard status == .installed else {
                return .pending(event: .inputMethodActivation)
            }
            
        }
        
        return .installed
    }
    
}
