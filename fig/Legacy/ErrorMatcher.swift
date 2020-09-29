//
//  ErrorMatcher.swift
//  fig
//
//  Created by Matt Schrage on 6/2/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import ANSITerminal

class ErrorMatcher {

    static let shared = ErrorMatcher()
    static func shouldMatchOn(data: String) -> Bool {
        return false && (data.contains("Error") || data.contains("error"))
    }
    //AIzaSyDWSCSwCRs4GrDLKMVK-dAw4W1dwMAe5Ig

    //AIzaSyDWSCSwCRs4GrDLKMVK-dAw4W1dwMAe5Ig
    static func matchOn(error: String,  match: @escaping ((Data) -> Void)) {
        let cleaned = ANSITerminal.stripAttributes(from: error)
        print("Error: ", error, "Cleaned: ",cleaned)
        let searchURL = URL(string: "https://www.googleapis.com/customsearch/v1?key=AIzaSyDWSCSwCRs4GrDLKMVK-dAw4W1dwMAe5Ig&cx=006414049974653183155:pajwd6ff6rn&q=\(cleaned.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? "")")!
        
            let task = URLSession.shared.dataTask(with: searchURL) {(data, response, error) in
                       guard let data = data else { return }
                       let res = String(data: data, encoding: .utf8)!
                       print(res)
                       match(data)
                   }

            task.resume()
        
    }
    
}
