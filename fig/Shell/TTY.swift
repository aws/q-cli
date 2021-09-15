//
//  TTY.swift
//  fig
//
//  Created by Matt Schrage on 12/18/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

struct proc {
  let pid: pid_t
  let cmd: String
  let tty: String?
  var name: String {
    return String(cmd.split(separator: "/").last ?? "")
  }
  var _cwd: String?
  
  init(line: String) {
    let tokens = line.split(separator: " ")
    pid = Int32(String(tokens[1])) ?? -1
    cmd = tokens.suffix(from: 2).joined(separator: " ")
    tty = nil
  }
  
  init(pid: pid_t, cmd: String, cwd: String, tty: String) {
    self.pid = pid
    self.cmd = cmd
    self._cwd = cwd
    self.tty = tty
  }
  
  var cwd: String? {
    guard let cwd = self._cwd else {
      return "/usr/sbin/lsof -an -p \(self.pid) -d cwd -F n | tail -1 | cut -c2-".runAsCommand().trimmingCharacters(in: .whitespaces)
    }
    return cwd
  }
  
  // Run cat /etc/shells
  var isShell: Bool {
    
    return (Defaults.processWhitelist + ["zsh","fish","bash", "csh","dash","ksh","tcsh", "ssh", "docker", "tmux"]).reduce(into: false) { (res, shell) in
      res = res || cmd.contains(shell)
    }
  }
  
  var isIntegratedShell: Bool {
    guard let executable = cmd.split(separator: "/").last else {
      return false
    }
    return ["zsh", "bash", "fish" ].contains( String(executable) )
  }
}

class TTY {
  static let processUpdated: NSNotification.Name = .init("processUpdated")
  
  // set from $FIG_INTEGRATION_VERSION
  var shellIntegrationVersion: Int?
  
  let descriptor: String
  var runUsingPrefix: String? = nil
  var pty: PseudoTerminalHelper? = nil
  var processes: [proc] {
    return ProcessStatus.getProcesses(for: self.descriptor).filter({ (process) -> Bool in
      return !(Defaults.ignoreProcessList.contains(process.cmd) || Defaults.ignoreProcessList.contains(process.name))
    }).reversed()
  }
  
  var running: proc? {
    return processes.last
  }
  
  init(fd: String) {
    descriptor = fd
  }
  
  func setTitle(_ title: String) {
    // https://tldp.org/HOWTO/Xterm-Title-3.html
    // ESC[2;titleBEL
    let pattern = "\u{1B}]2;\(title)\u{007}"
    
    //https://pubs.opengroup.org/onlinepubs/007904875/functions/open.html
    // writing escape sequence directly to STDIN to update title
    // writeonly, don't take control of tty, append
    let fd = Darwin.open("/dev/\(self.descriptor)", O_WRONLY | O_NOCTTY | O_APPEND, 0o644)
    let bytes: [UInt8] =  Array(pattern.utf8)
    
    bytes.withUnsafeBytes { (buffer) in
        let unsafeBufferPtr = buffer.bindMemory(to: UInt8.self)
        if let unsafePtr = unsafeBufferPtr.baseAddress {
             Darwin.write(fd, unsafePtr, bytes.count)
        }
    }

    
    //remember to close file descriptor
    Darwin.close(fd)

  }
  
  func update(for pid: pid_t? = nil) {
   guard self.shell == nil else {
      // if shell is set, then updating the list of processes is handled through the integration!
      return
    }
    
    let list = self.processes
    
    var process: proc? = nil
    if let pid = pid {
      process = list.filter { $0.pid == pid }.first // there should only be one
    } else {
      
      process = list.last
    }
    
    guard let runningProcess = process else { return }
    let cmd = runningProcess.cmd
    let cwd = runningProcess.cwd
    print("tty: running \(cmd) \(cwd ?? "<none>")")
    
    if let integration = self.integrationForProcess(runningProcess) {
      integration.update(tty: self, for: runningProcess)
    } else {
      self.cwd = cwd
      self.cmd = cmd
      self.pid = runningProcess.pid
      self.isShell = runningProcess.isShell
    }
  }
  
  var name: String? {
    guard let cmd = self.cmd else { return nil }
    return String(cmd.split(separator: "/").last ?? "")
  }
  var cwd: String?
  var cmd: String? {
    didSet {
      NotificationCenter.default.post(name: TTY.processUpdated, object: nil)
    }
  }
  var pid: pid_t?
  var isShell: Bool?
  var shell: proc?
  let integrations: [ String : CommandIntegration] = [
                                                      SSHIntegration.command : SSHIntegration(),
                                                      DockerIntegration.command : DockerIntegration()
                                                     ]
  
  func integrationForProcess(_ process: proc) -> CommandIntegration? {
    return integrations.reduce(nil) { (handler, kv) -> CommandIntegration? in
      guard handler == nil else { return handler }
      
      let (_, integration) = kv
      
      if integration.shouldHandleProcess(process) {
        return integration
      } else {
        return nil
      }
    }
  }
  func startedNewShellSession(for shellPid: pid_t) {
    self.shell = self.processes.filter { $0.pid == shellPid }.first
    precmd()
  }
  
  func precmd() {
    guard let shell = self.shell else { return }
    // ignore the prexec update, if it hasn't already happened.
    self.preexecWorkItem?.cancel()
    let updatedShell = self.processes.filter { $0.pid == shell.pid }.first
    if let runningProcess = updatedShell {
      self.cwd = runningProcess.cwd
      self.cmd = runningProcess.cmd
      self.pid = runningProcess.pid
      self.isShell = runningProcess.isShell
      self.runUsingPrefix = nil
      NotificationCenter.default.post(name: TTY.processUpdated, object: nil)
    }
  }
  
  fileprivate var preexecWorkItem: DispatchWorkItem?
  
  func preexec() {
    // this delay is a necessary hack, because if we run immediately upon recieving the preexec call
    // the shell process is still active...
    // Short lived processes can return control to shell before delay is over,
    // so this closure is cancelled by the precmd function
    self.preexecWorkItem = Timer.cancellableDelayWithSeconds(0.1, closure: {
      // if the process is a shell, it will be handled by a precmd hook.
      if let runningProcess = self.processes.last, runningProcess.isIntegratedShell != true {
        self.cwd = runningProcess.cwd
        self.cmd = runningProcess.cmd
        self.pid = runningProcess.pid
        self.isShell = runningProcess.isShell
        self.runUsingPrefix = nil
        NotificationCenter.default.post(name: TTY.processUpdated, object: nil)

        if let integration = self.integrationForProcess(runningProcess) {
          self.shell = nil
          if self.pty == nil {
            self.pty = PseudoTerminalHelper()
            self.pty?.start(with: [:])
          }
          integration.initialize(tty: self)
        }
      }
    })
  }
  
  func returnedToShellPrompt(for shellPid: pid_t) {
    if let shell = shell, shell.pid == shellPid {
      precmd()
      Logger.log(message: "Returned to shell prompt (\(shell.pid))", priority: .info, subsystem: .tty)
      return
    }
    self.startedNewShellSession(for: shellPid)
  }
}

extension TTY: Hashable {
  static func == (lhs: TTY, rhs: TTY) -> Bool {
    return lhs.descriptor == rhs.descriptor
  }
  
  func hash(into hasher: inout Hasher) {
    hasher.combine(self.descriptor)
  }
}
