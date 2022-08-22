//
//  DeepLink.swift
//  fig
//
//  Created by Ilkin Bayramli on 6/21/22.
//  Copyright © 2022 Matt Schrage. All rights reserved.
//

import Foundation

class DeepLinkHandler {

  private class func openPluginPage(path: String?) {
    MissionControl.openUI(MissionControl.Tab.plugins, additionalPathComponent: path)
  }

  class func handle(appName: String?, path: String?, queries: [URLQueryItem]?) {
    if let name = appName {
      switch name {
      case "plugins":
        openPluginPage(path: path)
      default:
        return
      }
    }
  }
}
