//
//  main.swift
//  figcli
//
//  Created by Matt Schrage on 5/14/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

func run() {

let arguments = CommandLine.arguments
    
// get stdin
var stdin = ""
//var line: String? = nil
//repeat {
//    line = readLine(strippingNewline: false)
//    if let line = line {
//        stdin += line
//    }
//} while (line != nil)

let env = ProcessInfo.processInfo.environment
print(env)
do {
    let jsonData = try JSONSerialization.data(withJSONObject: env, options: .prettyPrinted)
    print(jsonData)
} catch {
    print("Couldn't serialize ENV")
}

print(stdin)
print(arguments)

print("Hello, World! This is my command line tool")

}

run()
