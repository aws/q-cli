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
    func containsCurrentApplication() -> Bool
    
    func addLoginItem(_ url: URL)
    func removeLoginItem(_ url: URL)
    func itemForBundleURL(_ bundleURL: URL) -> LSSharedFileListItem?
}

class LoginItems {
  static let shared = LoginItems()
  var currentApplicationShouldLaunchOnStartup: Bool {
    get {
        return ((self as LoginItemsProtocol).itemForBundleURL(Bundle.main.bundleURL) != nil)
    }
    set (launchOnStartup) {
        if (launchOnStartup && !self.includesCurrentApplication) {
            (self as LoginItemsProtocol).addLoginItem(Bundle.main.bundleURL)
        } else if (!launchOnStartup && self.includesCurrentApplication) {
            (self as LoginItemsProtocol).removeLoginItem(Bundle.main.bundleURL)
        }
    }
  }
  
  var includesCurrentApplication: Bool {
    return ((self as LoginItemsProtocol).itemForBundleURL(Bundle.main.bundleURL) != nil)
  }
  
  func containsURL(_ url: URL) -> Bool {
    return ((self as LoginItemsProtocol).itemForBundleURL(url) != nil)
  }
  
  func removeURLIfExists(_ url: URL) {
    return (self as LoginItemsProtocol).removeLoginItem(url)
  }
  
    @available(macOS, deprecated: 10.11)
    func addLoginItem(_ url: URL = Bundle.main.bundleURL) {
        if let loginReference = LSSharedFileListCreate(kCFAllocatorDefault,
                                                       kLSSharedFileListSessionLoginItems.takeUnretainedValue(),
                                                       nil) {
           let loginListValue = loginReference.takeUnretainedValue()
           LSSharedFileListInsertItemURL(loginListValue,
                                         lastItem(),
                                         nil,
                                         nil,
                                         url as CFURL,
                                         nil,
                                         nil)

       }
    }
    @available(macOS, deprecated: 10.11)
    func lastItem() -> LSSharedFileListItem? {
        guard let loginReference = LSSharedFileListCreate(kCFAllocatorDefault,
                                                          kLSSharedFileListSessionLoginItems.takeUnretainedValue(),
                                                          nil) else { return nil }
        let loginListValue = loginReference.takeUnretainedValue()

        let items = LSSharedFileListCopySnapshot(loginListValue,
                                                 nil).takeRetainedValue() as NSArray
        
        guard items.count > 0 else { return nil }
        
        return (items.lastObject as! LSSharedFileListItem)
    }
    
    @available(macOS, deprecated: 10.11)
    func itemForBundleURL(_ bundleURL: URL = Bundle.main.bundleURL) -> LSSharedFileListItem? {
        guard let loginReference = LSSharedFileListCreate(kCFAllocatorDefault,
                                                          kLSSharedFileListSessionLoginItems.takeUnretainedValue(),
                                                          nil) else { return nil }
        let loginListValue = loginReference.takeUnretainedValue()

        let items = LSSharedFileListCopySnapshot(loginListValue,
                                                 nil).takeRetainedValue() as NSArray
        
        guard items.count > 0 else { return nil }
        
        let urlPtr = UnsafeMutablePointer<Unmanaged<CFURL>?>.allocate(capacity: 1)

        for item in items {
            let itemRef = item as! LSSharedFileListItem
            if LSSharedFileListItemResolve(itemRef,
                                           0,
                                           urlPtr,
                                           nil) == noErr {
              if let url: NSURL = urlPtr.pointee?.takeRetainedValue() {
                  print("URL Ref: \(url.lastPathComponent ?? "")")
                if url.isEqual(bundleURL) {
                  return itemRef
                }
              }
            }
        }
        
        return nil
    }
    
    @available(macOS, deprecated: 10.11)
    func removeLoginItem(_ url: URL = Bundle.main.bundleURL) {
        guard let itemRef = itemForBundleURL(url) else {
            return
        }

        if let loginReference = LSSharedFileListCreate(kCFAllocatorDefault,
                                                       kLSSharedFileListSessionLoginItems.takeUnretainedValue(),
                                                       nil) {
           let loginListValue = loginReference.takeUnretainedValue()
         LSSharedFileListItemRemove(loginListValue, itemRef)

       }
    }
    
    
    
    
    
  @available(macOS, deprecated: 10.11)
  func containsCurrentApplication() -> Bool {
    return ((self as LoginItemsProtocol).itemForBundleURL(Bundle.main.bundleURL) != nil)

  }
}

extension LoginItems: LoginItemsProtocol {
    
    // Note: this is used to resolve a bug where we added to many entries to login items
    func removeAllItemsMatchingBundleURL() {
        while (LoginItems.shared.containsURL(Bundle.main.bundleURL)) {
            LoginItems.shared.removeURLIfExists(Bundle.main.bundleURL)
        }
    }
}
