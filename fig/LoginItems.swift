//
//  LoginItems.swift
//  fig
//
//  Created by Matt Schrage on 3/24/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

// https://stackoverflow.com/questions/31540446/how-to-silence-a-warning-in-swift
private protocol LoginItemsProtocol {
    func toggleLaunchAtStartup(shouldBeOff: Bool)
    func containsCurrentApplication() -> Bool
}

class LoginItems {
  static let shared = LoginItems()
  var currentApplicationShouldLaunchOnStartup: Bool {
    get {
      return (self as LoginItemsProtocol).containsCurrentApplication()
    }
    set (newValue) {
      (self as LoginItemsProtocol).toggleLaunchAtStartup(shouldBeOff: !newValue)
    }
  }
  
  var includesCurrentApplication: Bool {
    return (self as LoginItemsProtocol).containsCurrentApplication()
  }
  
    @available(macOS, deprecated: 10.11)
    func toggleLaunchAtStartup(shouldBeOff: Bool = false) {
      let itemReferences = itemReferencesInLoginItems()
      let shouldBeToggled = (itemReferences.existingReference == nil)
      let loginItemsRef = LSSharedFileListCreate(
        nil,
        kLSSharedFileListSessionLoginItems.takeRetainedValue(),
        nil
      ).takeRetainedValue() as LSSharedFileList?
      
      if loginItemsRef != nil {
        if shouldBeToggled {
            let appUrl = NSURL.fileURL(withPath: Bundle.main.bundlePath) as CFURL
            LSSharedFileListInsertItemURL(loginItemsRef, itemReferences.lastReference, nil, nil, appUrl, nil, nil)
            print("Application was added to login items")
        }
        else if (shouldBeOff) {
          if let itemRef = itemReferences.existingReference {
            LSSharedFileListItemRemove(loginItemsRef,itemRef);
            print("Application was removed from login items")
          }
        }
      }
    }

  @available(macOS, deprecated: 10.11)
  func itemReferencesInLoginItems() -> (existingReference: LSSharedFileListItem?, lastReference: LSSharedFileListItem?) {
        
      let itemUrl = UnsafeMutablePointer<Unmanaged<CFURL>?>.allocate(capacity: 1)

        let appUrl = NSURL(fileURLWithPath: Bundle.main.bundlePath)
        let loginItemsRef = LSSharedFileListCreate(
          nil,
          kLSSharedFileListSessionLoginItems.takeRetainedValue(),
          nil
        ).takeRetainedValue() as LSSharedFileList?
        
        if loginItemsRef != nil {
          let loginItems = LSSharedFileListCopySnapshot(loginItemsRef, nil).takeRetainedValue() as NSArray
          print("There are \(loginItems.count) login items")
          
          if(loginItems.count > 0) {
            let lastItemRef = loginItems.lastObject as! LSSharedFileListItem
        
            for i in 0...loginItems.count-1 {
                let currentItemRef = loginItems.object(at: i) as! LSSharedFileListItem
              
              if LSSharedFileListItemResolve(currentItemRef, 0, itemUrl, nil) == noErr {
                if let urlRef: NSURL = itemUrl.pointee?.takeRetainedValue() {
                    print("URL Ref: \(urlRef.lastPathComponent ?? "")")
                  if urlRef.isEqual(appUrl) {
                    return (currentItemRef, lastItemRef)
                  }
                }
              }
              else {
                print("Unknown login application")
              }
            }
            // The application was not found in the startup list
            return (nil, lastItemRef)
            
          } else  {
            let addatstart: LSSharedFileListItem = kLSSharedFileListItemBeforeFirst.takeRetainedValue()
            return(nil,addatstart)
          }
      }
      
      return (nil, nil)
    }
  
  @available(macOS, deprecated: 10.11)
  func containsCurrentApplication() -> Bool {
    return itemReferencesInLoginItems().existingReference != nil
  }
}

extension LoginItems: LoginItemsProtocol {}
