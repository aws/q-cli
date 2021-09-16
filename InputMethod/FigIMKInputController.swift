//
//  FigInputController.swift
//  InputMethod
//
//  Created by Matt Schrage on 9/1/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import InputMethodKit

//InputMethodType = palette
//ComponentInvisibleInSystemUI=1
class FigIMKInputController: IMKInputController {
    
    private var _originalString: String = ""
    var timer: Timer?
    var bundleIdForClient: String?
    var isActive: Bool = false

    
    // Called once per client the first time it gets focus
    override init!(server: IMKServer!, delegate: Any!, client inputClient: Any!) {
        print("Initializing server...")

        super.init(server: server, delegate: delegate, client: inputClient)
        print("Timer!")
       
        
        bundleIdForClient = self.client().bundleIdentifier()

        let center = DistributedNotificationCenter.default()
        let keypressNotification = NSNotification.Name("io.fig.keypress")
        
        center.addObserver(forName: keypressNotification, object: nil, queue: nil) { aNotification in
            if self.bundleIdForClient == aNotification.userInfo?["bundleIdentifier"] as? String {
                print("A keypress occurred in \(aNotification.userInfo?["bundleIdentifier"] ?? "??")")

                if let client = self.downcastSender(self.client()), self.isActive {
                    self.getCoordinates(client)
                }
            }
        }
//
    }
    
    // Called when the client loses focus
    override func deactivateServer(_ sender: Any!) {
        print("Deactivating server...")
        self.isActive = false

    }
    
    // called when the client gains focus
    override func activateServer(_ sender: Any!) {
        print("Activating server...")
        self.isActive = true

        // The user may have been changing keyboard layouts while we were deactivated
        // (but the controller survives so init() may not be called again)
        // Will use the most recent ASCII capable keyboard layout to translate key
        // Events (see TextInputSources.h:TISSetInputMethodKeyboardLayoutOverride())
        // Sets the candidates window to use the same keyboard layout
        let lastASCIIlayout = TISCopyCurrentASCIICapableKeyboardLayoutInputSource().takeRetainedValue()
        candidatesWindow.setSelectionKeysKeylayout(lastASCIIlayout)
    }
    
    // Called when an input mode is selected; the input mode's
    // Identifier (from ComponentInputModeDict in Info.plist) is the
    // Value and kTextServiceInputModePropertyTag is the tag
    override func setValue(_ value: Any!, forTag tag: Int, client sender: Any!) {
        
        print("value \(String(describing: value)) tag \(tag)")
        if tag == kTextServiceInputModePropertyTag {
            
        } else {
            print("unhandled tag \(tag)")
            super.setValue(value, forTag: tag, client: sender)
        }
    }
    
    
    // handle deficiencies in the swift API: untyped senders should cast successfully
    private func downcastSender(_ sender: Any!) -> (IMKTextInput & IMKUnicodeTextInput)? {
        guard let downcast = sender as? (IMKTextInput & IMKUnicodeTextInput) else {
            print("sender \(String(describing: sender)) did not downcast, trying client()")
            return client() as? (IMKTextInput & IMKUnicodeTextInput)
        }
        return downcast
    }
    
    // insert marked text at the cursor
    private func getCoordinates(_ client: (IMKTextInput & IMKUnicodeTextInput)) {
        var tempRect = NSRect.zero
        client.attributes(forCharacterIndex: 0, lineHeightRectangle: &tempRect)
        
        let windowInsertionPoint = NSPoint(x: tempRect.maxX, y: tempRect.minY)
        let payload = "\(tempRect.origin.x),\(tempRect.origin.y),\(tempRect.size.width),\(tempRect.size.height)"
        FileManager.default.createFile(atPath: NSHomeDirectory() + "/.fig/tools/cursor",
                                       contents: payload.data(using: .utf8),
                                       attributes: nil)

        print("Bundle Id = \(client.bundleIdentifier() ?? "")")
        print("Window Level = \(client.windowLevel())")
        print("Insertion Point \(windowInsertionPoint)")
//        Logger.appendToLog("Bundle Id = \(client.bundleIdentifier() ?? "")")
//        Logger.appendToLog("Window Level = \(client.windowLevel())")
        Logger.appendToLog("Insertion Point \(windowInsertionPoint)\n")
    }
    
    // handle user actions
    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let client = downcastSender(sender) else {
            return false
        }
        
        switch event.type {
        case .keyUp, .keyDown:
            getCoordinates(client)

        default:
            break;
        }
        
        
        return false

    }
    
    // handle user keypress
}


class Logger {
    static func appendToLog(_ line: String) {
        let filepath = URL(fileURLWithPath: NSHomeDirectory() + "/.fig/logs/ime.log")
        if let file = try? FileHandle(forUpdating: filepath) {
            file.seekToEndOfFile()
    
            file.write(line.data(using: .utf8)!)
            file.closeFile()
        } else {
//            FileManager.default.createFile(atPath: filepath.absoluteString, contents: nil, attributes: nil)
            do {
                try line.write(to: filepath, atomically: true, encoding: String.Encoding.utf8)
            } catch {
              print("\(filepath.absoluteString) does not exist and could not be created. Logs will not be written.")
            }
        }
    }
}

