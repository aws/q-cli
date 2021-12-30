//
//  Plist.swift
//  fig
//
//  Created by Matt Schrage on 12/22/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

class PropertyList {
  static func read(from plistFilePath: URL) -> [String: AnyObject]? {
    var propertyListFormat =  PropertyListSerialization.PropertyListFormat.xml // Format of the Property List.
    var plistData: [String: AnyObject] = [:] // Our data
     // the path of the data
    let plistPath: String = plistFilePath.path
    guard let plistXML = FileManager.default.contents(atPath: plistPath) else {
      return nil
    }
    do {// convert the data to a dictionary and handle errors.
      // swiftlint:disable force_cast
      plistData = try PropertyListSerialization.propertyList(from: plistXML,
                                                          options: .mutableContainersAndLeaves,
                                                           format: &propertyListFormat) as! [String: AnyObject]
    } catch {
      print("Error reading plist: \(error), format: \(propertyListFormat)")
      return nil
    }

    return plistData
  }
}
