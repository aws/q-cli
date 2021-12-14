//
//  Constants.swift
//  fig
//
//  Created by Matt Schrage on 10/12/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

extension Bundle {
  var configURL: URL {
    return Bundle.main.resourceURL!.appendingPathComponent("config", isDirectory: true)
  }

}
