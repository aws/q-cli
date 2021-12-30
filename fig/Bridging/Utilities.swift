//
//  Utilities.swift
//  fig
//
//  Created by Matt Schrage on 11/22/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

extension String {
  func jsonStringToDict() -> [String: Any]? {
    if let data = self.data(using: .utf8) {
      do {
        return try JSONSerialization.jsonObject(with: data, options: []) as? [String: Any]
      } catch {
        print(error.localizedDescription)
      }
    }
    return nil
  }
}

extension Collection {

  /// Returns the element at the specified index if it is within bounds, otherwise nil.
  subscript (safe index: Index) -> Element? {
    return indices.contains(index) ? self[index] : nil
  }
}

//https://stackoverflow.com/a/48360631
extension URL {
  func relativePath(from base: URL) -> String? {
    // Ensure that both URLs represent files:
    guard self.isFileURL && base.isFileURL else {
      return nil
    }

    // Remove/replace "." and "..", make paths absolute:
    let destComponents = self.standardized.pathComponents
    let baseComponents = base.standardized.pathComponents

    // Find number of common path components:
    var count = 0
    while count < destComponents.count && count < baseComponents.count
            && destComponents[count] == baseComponents[count] {
      count += 1
    }

    // Build relative path:
    var relComponents = Array(repeating: "..", count: baseComponents.count - count)
    relComponents.append(contentsOf: destComponents[count...])
    return relComponents.joined(separator: "/")
  }
}

extension String {
  func groups(for regexPattern: String) -> [[String]] {
    do {
      let text = self
      let regex = try NSRegularExpression(pattern: regexPattern)
      let matches = regex.matches(in: text,
                                  range: NSRange(text.startIndex..., in: text))
      return matches.map { match in
        return (0..<match.numberOfRanges).map {
          let rangeBounds = match.range(at: $0)
          guard let range = Range(rangeBounds, in: text) else {
            return ""
          }
          return String(text[range])
        }
      }
    } catch let error {
      print("invalid regex: \(error.localizedDescription)")
      return []
    }
  }
}

extension String {
  var unescapingUnicodeCharacters: String {
    let mutableString = NSMutableString(string: self)
    CFStringTransform(mutableString, nil, "Any-Hex/Java" as NSString, true)

    return mutableString as String
  }
}

extension String {
  var unescaped: String {
    let entities = ["\0", "\t", "\n", "\r", "\"", "\'", "\\"]
    var current = self.replacingOccurrences(of: "\\/", with: "/")
    for entity in entities {
      let descriptionCharacters = entity.debugDescription.dropFirst().dropFirst().dropLast().dropLast()
      let description = String(descriptionCharacters)
      current = current.replacingOccurrences(of: description, with: entity)
    }
    return current
  }
}

extension String {

  func stringByReplacingFirstOccurrenceOfString(_ target: String, withString replaceString: String) -> String {
    if let range = self.range(of: target) {
      return self.replacingCharacters(in: range, with: replaceString)
    }
    return self
  }

}

extension URL {
  static let applicationSupport =
    URL(fileURLWithPath: NSSearchPathForDirectoriesInDomains(.applicationSupportDirectory,
                                                             .userDomainMask,
                                                             true).first!)
}
