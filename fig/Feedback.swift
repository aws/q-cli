//
//  Feedback.swift
//  fig
//
//  Created by Matt Schrage on 1/11/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class Feedback {
  static func getFeedback(source: String = "unspecified", placeholder: String? = nil) {
    if let response = Feedback.prompt(title: "Feedback on Fig?",
                                      question: "Please let us know about any issues you have been running into or if there is anything we can do to improve the experience.",
                                      defaultText: placeholder) {
      let body = [ "email": Defaults.shared.email ?? "unknown", "text": response, "via": source]
      upload(to: "feedback",
             with: body)
    }
  }
  static func prompt(title: String, question: String, defaultText: String? = nil) -> String? {
    let prompt = NSAlert()
    prompt.addButton(withTitle: "Send")      // 1st button
    prompt.addButton(withTitle: "Cancel")  // 2nd button
    prompt.messageText = title
    prompt.informativeText = question

    let txt = NSTextView(frame: NSRect(x: 0, y: 0, width: 293, height: 150))
    txt.textContainerInset = NSSize(width: 3, height: 5)
    txt.string = defaultText ?? ""

    prompt.accessoryView = txt
    prompt.window.initialFirstResponder =  prompt.accessoryView

    let response: NSApplication.ModalResponse = prompt.runModal()

    if response == NSApplication.ModalResponse.alertFirstButtonReturn {
      return txt.string
    } else {
      return nil
    }
  }

  fileprivate static func upload(to endpoint: String, with body: [String: String], completion: ((Data?, URLResponse?, Error?) -> Void)? = nil) {
    guard let json = try? JSONSerialization.data(withJSONObject: body, options: .sortedKeys) else { return }
    print(json)
    var request = URLRequest(url: Remote.API.appendingPathComponent(endpoint))
    request.httpMethod = "POST"
    request.httpBody = json
    request.setValue("application/json; charset=utf-8", forHTTPHeaderField: "Content-Type")

    let task = URLSession.shared.dataTask(with: request) { (data, res, err) in
      if let handler = completion {
        handler(data, res, err)
      }
    }

    task.resume()
  }
}
