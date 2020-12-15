//
//  Onboarding.swift
//  fig
//
//  Created by Matt Schrage on 8/19/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation
import Sentry

class Onboarding {
    
    // check current hash with ...
    static let commitHashForVersion = ""//["1.0.24" : "hi"]
    
    static func setUpEnviroment(completion:( () -> Void)? = nil) {
        
        DispatchQueue.global(qos: .userInitiated).async {
            let githubURL = URL(string: "https://raw.githubusercontent.com/withfig/config/main/tools/install_and_upgrade.sh")!
            if let envSetupScript = try? String(contentsOf: githubURL) {
                let scriptsURL = FileManager.default.urls(for: .applicationScriptsDirectory, in: .userDomainMask)[0] as NSURL

                guard let folderPath = scriptsURL.path else {
                    Logger.log(message: "Folder path does not exist")
                    return
                }
                
                Logger.log(message: String(describing: scriptsURL.path))

                guard let script = scriptsURL.appendingPathComponent("install_and_upgrade.sh") else {
                    Logger.log(message: "Could not create PATH for install_and_upgrade.sh")
                    SentrySDK.capture(message: "Could not create PATH for install_and_upgrade.sh")
                    return
                }
                Logger.log(message: script.path)

                do {
                    try FileManager.default.createDirectory(atPath: folderPath, withIntermediateDirectories: true)
                    try envSetupScript.write(to: script, atomically: true, encoding: String.Encoding.utf8)
                } catch {
                    SentrySDK.capture(message: "Could not write to file.")
                    Logger.log(message: "Could not write to file.")

                    return
                }


                print("onboarding: ", script)
                
                guard let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String else {
                    Logger.log(message: "No version availible")
                    return
                }
                
                let out = "/bin/bash '\(script.path)' v\(version)".runAsCommand()
                
                guard !out.starts(with: "Error:") else {
                    Logger.log(message: out)
                    SentrySDK.capture(message: "Onboarding: \(out)")
                    return
                }
                
                Logger.log(message: "Successfully ran installation script!")
                Logger.log(message: "\(out)")
                SentrySDK.capture(message: "Script: \(out)")

                
            } else {
                Logger.log(message: "Could not download installation script")
                SentrySDK.capture(message: "Could not download installation script")
                // What should we do when this happens?
            }
        }
    }

//    static func installation() {
//        let userShell = "dscl . -read ~/ UserShell".runAsCommand().trimmingCharacters(in: .whitespacesAndNewlines).replacingOccurrences(of: "UserShell: ", with: "")
//        var script = try? String(contentsOf: URL(string: "\(Remote.baseURL)/onboarding/install?shell=\(userShell)")!)
//        var welcome = try? String(contentsOf: URL(string: "\(Remote.baseURL)/onboarding/welcome.run")!)
//
//        script = script ?? Onboarding.defaultInstallScript
//        welcome = welcome ?? ""
//
//        let _ = script!.runInBackground(completion: {
//            try? welcome!.write(to: URL(string:  NSHomeDirectory() + "/run/welcome.run")!, atomically: true, encoding: .utf8)
//        })
//    }
//
//    static let defaultInstallScript: String =
//    """
//    mkdir -p ~/.fig/exports/;
//
//    touch ~/.fig/exports/env.sh;
//
//    touch ~/.fig/exports/searchIndex.txt;
//
//    touch ~/.fig/exports/global.fig;
//
//    mkdir -p ~/run/;
//
//    echo 'export FIGPATH="~/.fig/bin:~/run:"\\nFIGPATH=$FIGPATH\\n\\n##Run aliases shell script\\nsource $(dirname "$0")/aliases.sh' > ~/.fig/exports/env.sh;
//
//    touch ~/.fig/exports/aliases.sh;
//
//    mkdir -p ~/.fig/bin;
//
//    touch ~/.fig/bin/installedApps.fig;
//
//    echo '\\n\\n#### FIG ENV VARIABLES ####\\n[ -f ~/.fig/exports/env.sh ] && source ~/.fig/exports/env.sh \\n#### END FIG ENV VARIABLES ####\\n\\n' | tee -a ~/.profile ~/.zprofile ~/.bash_profile;
//    """
}
