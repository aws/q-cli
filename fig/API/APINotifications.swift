//
//  Notifications.swift
//  fig
//
//  Created by Matt Schrage on 9/21/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation
import FigAPIBindings

class APINotifications {
    
    // todo(mschrage): 
    static func post(_ notification: Fig_Notification) {
        
    }
    
    static func post(_ notification: Fig_EditBufferChangedNotification) {
        var wrapper = Fig_Notification()
        wrapper.editBufferNotification = notification
        APINotifications.post(wrapper)
    }
    
    static func post(_ notification: Fig_ProcessChangedNotification) {
        
    }
    
    static func post(_ notification: Fig_SettingsChangedNotification) {
        
    }
    
    static func post(_ notification: Fig_LocationChangedNotification) {
        
    }
    
}
