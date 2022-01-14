//
//  CommandIntegration.swift
//  fig
//
//  Created by Matt Schrage on 1/12/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Foundation

protocol CommandIntegration {
  func update(tty: TTY, for process: proc)
  func runUsingPrefix() -> String?
  func initialize(tty: TTY)
  static var command: String { get }
  func shouldHandleProcess(_ process: proc) -> Bool
}
