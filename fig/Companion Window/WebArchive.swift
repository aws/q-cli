//
//  WebArchive.swift
//  fig
//
//  Created by Matt Schrage on 10/14/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

import WebArchiver

extension WebView {
    static let doNotArchive: Set<URL> = [ Onboarding.loginURL ]
    
    fileprivate var cacheDirectory: URL {
        guard let cachesDirectory = FileManager.default.urls(for: .cachesDirectory, in: .allDomainsMask).first else {
            return URL(fileURLWithPath: NSHomeDirectory() + "/.fig/.cache")
        }
        
        return  cachesDirectory.appendingPathComponent("fig",
                                                       isDirectory: true)
    }
    
    fileprivate func archivePath(for url: URL) -> URL {
        try? FileManager.default.createDirectory(atPath: cacheDirectory.path,
                                            withIntermediateDirectories: true,
                                            attributes: nil)
        
        // drop first to remove leading '/'
        let identifier = url.pathComponents.dropFirst().joined(separator: "-")
        
        return cacheDirectory.appendingPathComponent(identifier + ".webarchive")
    }
    
    func archive() {
        
        guard let url = self.url, !url.isFileURL, url.host != "localhost", !WebView.doNotArchive.contains(url) else {
            return
        }
        self.archive(to: archivePath(for: url).path)
        
    }
    
    func archive(to path: String) {
        guard let url = self.url else {
            return
        }
      
        guard !WebView.doNotArchive.contains(url) else {
            Logger.log(message: "NOT archiving \(url)...")
            return
        }
      
        Logger.log(message: "Archiving \(url)...")

        WebArchiver.archive(url: url,
                            cookies: [],
                            includeJavascript: true,
                            skipCache: false) { archive in
            
            guard archive.errors.count == 0 else {
                Logger.log(message: "Failed to archive \(url)...")
                
                archive.errors.forEach { Logger.log(message: $0.localizedDescription) }
                return
            }
            
            let fileURL = URL(fileURLWithPath: path)
            try? FileManager.default.removeItem(at: fileURL)
            try? archive.plistData?.write(to: fileURL)
            
            Logger.log(message: "Successfully archived \(url) as \(fileURL.path)")

            
        }
    }
      
    func loadArchivedURL() {
        // ensure the requestedURL is not a fileURL to avoid infinite loop
      guard let url = self.requestedURL, !url.isFileURL, !WebView.doNotArchive.contains(url) else { return }
        let archived = archivePath(for: url)
        Logger.log(message: "Attempting to load archived version of \(url) (\(archived))")
        self.loadFileURL(archived, allowingReadAccessTo: cacheDirectory)
    }
}
