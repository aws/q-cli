//
//  DockerIntegration.swift
//  fig
//
//  Created by Matt Schrage on 1/28/21.
//  Copyright © 2021 Matt Schrage. All rights reserved.
//

import Cocoa

class DockerIntegration: CommandIntegration {
    static var command = "com.docker.cli"
    var container: String?
    func runUsingPrefix() -> String? {
      //docker exec mycontainer /bin/sh -c
      //"docker exec -it \(container) "
        if let container = container {
            return "docker exec \(container) /bin/sh -c "
        }
        
        return nil
    }

  
    func update(tty: TTY, for process: proc) {
        if tty.pty == nil {
            print("Starting PTY...!")
            tty.pty = PseudoTerminalHelper()
            tty.pty?.start(with: [:])
            return
        }
      
        if container == nil {
          return
        }
      

//        let semaphore = DispatchSemaphore(value: 0)
//        let scriptPath = Bundle.main.path(forResource: "remote_cwd", ofType: "sh")!
        guard let prefix = self.runUsingPrefix() else {
            return
        }
      
        tty.pty!.execute("\(prefix) 'readlink /proc/1/cwd'") { output in
            print("Docker: working directory = ", output)
//            guard tty.pid == process.pid else {
//                print("Docker: Process out of sync, abort update - \(tty.pid) != \(process.pid)")
////                semaphore.signal()
//                return
//            }
          
            // do some error checking - does output match a directory regex.
            tty.cwd = output.trimmingCharacters(in: .whitespacesAndNewlines)
            tty.cmd = process.cmd
            tty.pid = process.pid
            tty.isShell = process.isShell
            tty.runUsingPrefix = prefix
//            semaphore.signal()

        }
      
//        semaphore.wait()

    }

    func initialize(tty: TTY) {
      
      DockerEventStream.shared.onNextEvent(matching: ["create", "start", "exec_create", "exec_start", "resize"]) { (event) in
        guard let id = event.id else {
          print("Docker: event did not have an associated container id")
          return
        }
        print("Docker: recieved event '\(event.status ?? "unknown")', setting container id = \(id)")
        self.container = id
        tty.update()
      }
    }

  
    // --context is included because when a new docker container is created, it can sometimes appear as the subcommand temporarily
    let supportedDockerSubcommands = ["run", "attach", "exec", "start", "--context"]
    func shouldHandleProcess(_ process: proc) -> Bool {
      guard process.name == DockerIntegration.command || process.name == "docker" else {
        print("Docker: \(process.name) is not docker.")
        return false
      }
      guard let subcommand = lsof.arguments(fromPid: process.pid).split(separator: " ")[safe: 1] else {
        print("Docker: No subcommands for docker process")

        return false
      }
      print("Docker: command is '\(lsof.arguments(fromPid: process.pid))'")
      print("Docker: \(subcommand) is supported? \(supportedDockerSubcommands.contains(String(subcommand)))")
      
      return supportedDockerSubcommands.contains(String(subcommand))
    }

}

class DockerEventStream {
  static let shared = DockerEventStream()
  let socket = UnixSocketClient(path: "/var/run/docker.sock")
  var activeContainers: [String] = []
  init() {
    
    guard self.appIsInstalled else {
      print("Docker is not installed.")
      return
    }
    socket.delegate = self
    
    if daemonIsRunning {
      socket.connect()
      
      // can we get a callback from connect?
      Timer.delayWithSeconds(1) {
        self.socket.send(message: "GET /events HTTP/1.0\r\n\r\n")
      }
      
    } else {
      // if docker is installed, periodically check if docker is running

      // schedule timer to periodically check
    }
    
  }
  
  var appIsInstalled: Bool {
    return NSWorkspace.shared.urlForApplication(withBundleIdentifier: "com.docker.docker") != nil
  }
  
  var daemonIsRunning: Bool {
    return true
  }
  
  typealias DockerEventHandler = (ContainerEvent) -> Void
  var handlers: [ ([String], DockerEventHandler) ] = []
  func onNextEvent(matching status: [String] = [], handler: @escaping DockerEventHandler) {
    handlers.append((status, handler))
  }
  
  func processEvent(_ raw: String) {
    guard let data = raw.data(using: .utf8) else {
      print("Docker: could not convert to data")
      return
    }
    let jsonDecoder = JSONDecoder()
    if let event = try? jsonDecoder.decode(DockerEventStream.ContainerEvent.self, from: data) {
      print("Docker: Event '\(event.status ?? "unknown")' in container '\(event.id ?? "???")'")
      print("Docker: handlers = \(handlers.count)")

      handlers = handlers.reduce([]) { (remaining, item) -> [([String], DockerEventHandler)] in
        let (conditions, handler) = item

        if let status = event.status, conditions.count == 0 || conditions.contains(status) {
          handler(event)
          return remaining
        } else {
          return remaining + [item]
        }
      }
    }
  }

}

extension DockerEventStream: UnixSocketDelegate {
  func socket(_ socket: UnixSocketClient, didReceive message: String) {
    print("Docker: recieved message")

    message.split(separator:"\n").forEach { (item) in
      processEvent(String(item))
    }
  }
  
  func socket(_ socket: UnixSocketClient, didReceive data: Data) {

  }
  
  func socketDidClose(_ socket: UnixSocketClient) {
    // schedule attempts to reconnnect
  }
}

extension DockerEventStream {
  
  struct Actor : Codable {
    let id : String?
    let attributes : Attributes?

    enum CodingKeys: String, CodingKey {

      case id = "ID"
      case attributes = "Attributes"
    }

    init(from decoder: Decoder) throws {
      let values = try decoder.container(keyedBy: CodingKeys.self)
      id = try values.decodeIfPresent(String.self, forKey: .id)
      attributes = try values.decodeIfPresent(Attributes.self, forKey: .attributes)
    }

  }
  
  struct Attributes : Codable {
    let exitCode : String?
    let image : String?
    let name : String?

    enum CodingKeys: String, CodingKey {

      case exitCode = "exitCode"
      case image = "image"
      case name = "name"
    }

    init(from decoder: Decoder) throws {
      let values = try decoder.container(keyedBy: CodingKeys.self)
      exitCode = try values.decodeIfPresent(String.self, forKey: .exitCode)
      image = try values.decodeIfPresent(String.self, forKey: .image)
      name = try values.decodeIfPresent(String.self, forKey: .name)
    }

  }
  
  struct ContainerEvent : Codable {
    let status : String?
    let id : String?
    let from : String?
    let type : String?
    let action : String?
    let actor : Actor?
    let scope : String?
    let time : Int?
    let timeNano : Int?

    enum CodingKeys: String, CodingKey {

      case status = "status"
      case id = "id"
      case from = "from"
      case type = "Type"
      case action = "Action"
      case actor = "Actor"
      case scope = "scope"
      case time = "time"
      case timeNano = "timeNano"
    }

    init(from decoder: Decoder) throws {
      let values = try decoder.container(keyedBy: CodingKeys.self)
      status = try values.decodeIfPresent(String.self, forKey: .status)
      id = try values.decodeIfPresent(String.self, forKey: .id)
      from = try values.decodeIfPresent(String.self, forKey: .from)
      type = try values.decodeIfPresent(String.self, forKey: .type)
      action = try values.decodeIfPresent(String.self, forKey: .action)
      actor = try values.decodeIfPresent(Actor.self, forKey: .actor)
      scope = try values.decodeIfPresent(String.self, forKey: .scope)
      time = try values.decodeIfPresent(Int.self, forKey: .time)
      timeNano = try values.decodeIfPresent(Int.self, forKey: .timeNano)
    }
  }
}
