//
//  Onboarding.swift
//  fig
//
//  Created by Matt Schrage on 8/19/20.
//  Copyright © 2020 Matt Schrage. All rights reserved.
//

import Foundation

class Onboarding {

    static func installation() {
        let userShell = "dscl . -read ~/ UserShell".runAsCommand().trimmingCharacters(in: .whitespacesAndNewlines).replacingOccurrences(of: "UserShell: ", with: "")
        var script = try? String(contentsOf: URL(string: "\(Remote.baseURL)/onboarding/install?shell=\(userShell)")!)
        var welcome = try? String(contentsOf: URL(string: "\(Remote.baseURL)/onboarding/welcome.run")!)
        
        script = script ?? Onboarding.defaultInstallScript
        welcome = welcome ?? ""
        
        let _ = script!.runInBackground(completion: {
            try? welcome!.write(to: URL(string:  NSHomeDirectory() + "/run/welcome.run")!, atomically: true, encoding: .utf8)
        })
    }
    
    static let defaultInstallScript: String =
    """
    mkdir -p ~/.fig/exports/;

    touch ~/.fig/exports/env.sh;

    touch ~/.fig/exports/searchIndex.txt;

    touch ~/.fig/exports/global.fig;

    mkdir -p ~/run/;

    echo 'export FIGPATH="~/.fig/bin:~/run:"\\nFIGPATH=$FIGPATH\\n\\n##Run aliases shell script\\nsource $(dirname "$0")/aliases.sh' > ~/.fig/exports/env.sh;

    touch ~/.fig/exports/aliases.sh;

    mkdir -p ~/.fig/bin;

    touch ~/.fig/bin/installedApps.fig;

    echo '\\n\\n#### FIG ENV VARIABLES ####\\n[ -f ~/.fig/exports/env.sh ] && source ~/.fig/exports/env.sh \\n#### END FIG ENV VARIABLES ####\\n\\n' | tee -a ~/.profile ~/.zprofile ~/.bash_profile;
    """
}
