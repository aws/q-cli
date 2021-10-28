//
//  API+Extensions.swift
//  fig
//
//  Created by Matt Schrage on 9/28/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa
import FigAPIBindings
extension NSEvent {
    var fig_keyEvent: Fig_KeyEvent {
        return Fig_KeyEvent.with {

            $0.appleKeyCode = Int32(self.keyCode)
            
            if let characters = self.characters {
                $0.characters = characters
            }
            
            if let charactersIgnoringModifiers = self.charactersIgnoringModifiers {
                $0.charactersIgnoringModifiers = charactersIgnoringModifiers
            }
            
            $0.isRepeat = self.isARepeat
            
            $0.modifiers = {
                var modifiers: [Fig_Modifiers] = []

                if self.modifierFlags.contains(.command) {
                    modifiers.append(.command)
                }
                
                if self.modifierFlags.contains(.option) {
                    modifiers.append(.option)
                }
                
                if self.modifierFlags.contains(.control) {
                    modifiers.append(.control)
                }
                
                if self.modifierFlags.contains(.function) {
                    modifiers.append(.function)
                }
                
                if self.modifierFlags.contains(.shift) {
                    modifiers.append(.shift)
                }
                
                return modifiers
            }()
        }
    }
}

extension NSRect {
    var fig_frame: Fig_Frame {
        return Fig_Frame.with { frame in
            frame.origin = Fig_Point.with { origin in
                origin.x = Float(self.origin.x)
                origin.y = Float(self.origin.y)
            }
            
            frame.size = Fig_Size.with { size in
                size.width = Float(self.size.width)
                size.height = Float(self.size.height)
            }
        }
    }
}
extension ExternalWindow {
    var fig_window: Fig_Window {
        return Fig_Window.with { window in
            window.app = Fig_Application.with { app in
                if let bundleId = self.app.bundleIdentifier {
                    app.bundleIdentifier = bundleId
                }
                
                if let name = self.app.localizedName {
                    app.name = name
                }
            }
            
            if let frame = NSScreen.main?.frame.fig_frame {
                window.currentScreen = Fig_Screen.with { screen in
                    screen.frame = frame
                }
            }
            
            window.frame = self.frame.fig_frame
            window.windowID = String(self.windowId)
            
            if let tty = ShellHookManager.shared.tty(for: self.hash) {
                window.currentSession = Fig_Session.with{ session in
                    
                    if let pid = tty.pid, let executable = tty.cmd, let directory = tty.cwd {
                        session.frontmostProcess = Fig_Process.with{ process in
                            process.pid = pid
                            process.executable = executable
                            process.directory = directory
                        }
                        
                        if let sessionId = ShellHookManager.shared.getSessionId(for: self.hash) {
                            session.sessionID = sessionId
                        }
                    }
                }
            }
        }
    }
}

extension NSWorkspace {
  func handleOpenURLRequest(_ request: Fig_OpenInExternalApplicationRequest) throws -> Bool {
    guard request.hasURL else { throw APIError.generic(message: "Missing 'url' parameter") }
    
    guard let url = URL(string: request.url) else {
      throw APIError.generic(message: "Could not parse '\(request.url)' as a URL")
    }
   
    return open(url)

  }
}
