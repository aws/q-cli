//
//  File.swift
//  fig
//
//  Created by Matt Schrage on 5/23/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Cocoa

class File {
    static func contentsOfFile(_ path: String, type: String ) -> String? {
        if let filepath = Bundle.main.path(forResource: path, ofType: type) {
            do {
                let contents = try String(contentsOfFile: filepath)
                return contents
//                print(contents)
            } catch {
                // contents could not be loaded
            }
        } else {
            // example.txt not found!
        }
        return nil
    }
}
