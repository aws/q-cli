//
//  Github.swift
//  fig
//
//  Created by Matt Schrage on 4/13/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Github {
  static func openIssue(with body: String? = nil) {
        let os = ProcessInfo.processInfo.operatingSystemVersion

        let body =
        """
        ### Description:
        > Please include a detailed description of the issue (and an image, if applicable)
        
        \(body ?? "")
        
        
        
         
        ### Details:
        |macOS|Fig|Shell|
        |-|-|-|
        |\(os.majorVersion).\(os.minorVersion).\(os.patchVersion)|\(Diagnostic.distribution)|\(Defaults.userShell)|
        <details><summary><code>fig diagnostic</code></summary>
        <p>
        <pre>\(Diagnostic.summary.trimmingCharacters(in: .whitespacesAndNewlines))</pre>
        </p>
        </details>
        """
    NSWorkspace.shared.open(URL(string:"https://github.com/withfig/fig/issues/new?labels=bug&assignees=mattschrage&body=\(body.addingPercentEncoding(withAllowedCharacters: .urlHostAllowed)!)")!)
  }
}
