//
//  AppMover.swift
//  AppMover
//
//  Created by Oskar Groth on 2019-12-20.
//  Copyright © 2019 Oskar Groth. All rights reserved.
//
import Cocoa
import AppKit
import Security

public enum AppMover {

  public static func moveIfNecessary() {
    let fm = FileManager.default
    guard !Bundle.main.isInstalled,
          let applications = preferredInstallDirectory() else { return }
    let bundleUrl = Bundle.main.bundleURL
    let bundleName = bundleUrl.lastPathComponent
    let destinationUrl = applications.appendingPathComponent(bundleName)
    let needDestAuth = fm.fileExists(atPath: destinationUrl.path) && !fm.isWritableFile(atPath: destinationUrl.path)
    let needAuth = needDestAuth || !fm.isWritableFile(atPath: applications.path)

    // Activate app -- work-around for focus issues related to "scary file from
    // internet" OS dialog.
    if !NSApp.isActive {
      NSApp.activate(ignoringOtherApps: true)
    }

    let alert = NSAlert()
    alert.messageText = "Move to Applications folder"
    alert.informativeText =
      "\(Bundle.main.localizedName) needs to move to your Applications folder in order to work properly."
    if needAuth {
      alert.informativeText.append(" You need to authenticate with your administrator password to complete this step.")
    }
    alert.addButton(withTitle: "Move to Applications Folder")
    alert.addButton(withTitle: "Quit")
    guard alert.runModal() == .alertFirstButtonReturn else {
      exit(0)
    }
    if needAuth {
      let result = authorizedInstall(from: bundleUrl, to: destinationUrl)
      guard !result.cancelled else { moveIfNecessary(); return }
      guard result.success else {
        NSApplication.shared.terminate(self)
        return
      }
    } else {
      if fm.fileExists(atPath: destinationUrl.path) {
        if AppMover.isApplicationAtUrlRunning(destinationUrl) {
          NSWorkspace.shared.open(destinationUrl)
          return
        } else {
          guard (try? fm.trashItem(at: destinationUrl, resultingItemURL: nil)) != nil else {
            return
          }
        }
      }
      guard (try? fm.copyItem(at: bundleUrl, to: destinationUrl)) != nil else {
        return
      }
    }

    // Trash the original app
    _ = try? fm.removeItem(at: bundleUrl)

    relaunch(at: destinationUrl.path, completionCallback: {
      DispatchQueue.main.async {
        exit(0)
      }
    })

  }

  static func authorizedInstall(from sourceURL: URL, to destinationURL: URL) -> (cancelled: Bool, success: Bool) {
    guard destinationURL.representsBundle,
          destinationURL.isValid,
          sourceURL.isValid else {
      return (false, false)
    }
    return sourceURL.withUnsafeFileSystemRepresentation({ sourcePath -> (cancelled: Bool, success: Bool) in
      return destinationURL.withUnsafeFileSystemRepresentation({ destinationPath -> (cancelled: Bool, success: Bool) in
        guard let sourcePath = sourcePath, let destinationPath = destinationPath else { return (false, false) }
        let deleteCommand = "rm -rf '\(String(cString: destinationPath))'"
        let copyCommand = "cp -pR '\(String(cString: sourcePath))' '\(String(cString: destinationPath))'"
        guard let script = NSAppleScript(
          source: "do shell script \"\(deleteCommand) && \(copyCommand)\" with administrator privileges"
        ) else {
          return (false, false)
        }
        var error: NSDictionary?
        script.executeAndReturnError(&error)
        return ((error?[NSAppleScript.errorNumber] as? Int16) == -128, error == nil)
      })
    })
  }

  static func preferredInstallDirectory() -> URL? {
    let fm = FileManager.default
    let dirs = fm.urls(for: .applicationDirectory, in: .allDomainsMask)
    // Find Applications dir with the most apps that isn't system protected
    return dirs.map({ $0.resolvingSymlinksInPath() }).filter({ url in
      var isDir: ObjCBool = false
      fm.fileExists(atPath: url.path, isDirectory: &isDir)
      return isDir.boolValue && url.path != "/System/Applications"
    }).sorted(by: { left, right in
      return left.numberOfFilesInDirectory < right.numberOfFilesInDirectory
    }).last
  }

  static func isApplicationAtUrlRunning(_ url: URL) -> Bool {
    let url = url.standardized
    return NSWorkspace.shared.runningApplications.contains(where: {
      $0.bundleURL?.standardized == url
    })
  }

  public static func relaunch(at path: String, completionCallback: @escaping () -> Void) {
    let pid = ProcessInfo.processInfo.processIdentifier
    Process.runTask(command: "/usr/bin/xattr", arguments: ["-d", "-r", "com.apple.quarantine", path], completion: { _ in
      let waitForExitScript = "(while /bin/kill -0 \(pid) >&/dev/null; do /bin/sleep 0.1; done; /usr/bin/open \"\(path)\") &"
      Process.runTask(command: "/bin/sh", arguments: ["-c", waitForExitScript])
      completionCallback()
    })
  }

}

fileprivate extension URL {

  var representsBundle: Bool {
    pathExtension == "app"
  }

  var isValid: Bool {
    !path.trimmingCharacters(in: .whitespaces).isEmpty
  }

  var numberOfFilesInDirectory: Int {
    (try? FileManager.default.contentsOfDirectory(atPath: path))?.count ?? 0
  }

}

fileprivate extension Bundle {

  var localizedName: String {
    NSRunningApplication.current.localizedName ?? "The App"
  }

  var isInstalled: Bool {
    NSSearchPathForDirectoriesInDomains(.applicationDirectory, .allDomainsMask, true)
      .contains(where: { $0.hasPrefix(bundlePath) }) || bundlePath.split(separator: "/").contains("Applications")
  }

  func copy(to url: URL) throws {
    try FileManager.default.copyItem(at: bundleURL, to: url)
  }

}

fileprivate extension Process {

  static func runTask(command: String, arguments: [String] = [], completion: ((Int32) -> Void)? = nil) {
    let task = Process()
    task.launchPath = command
    task.arguments = arguments
    task.terminationHandler = { task in
      completion?(task.terminationStatus)
    }
    task.launch()
  }

}
