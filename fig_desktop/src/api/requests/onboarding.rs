use anyhow::Context;
use fig_integrations::get_default_backup_dir;
use fig_integrations::shell::ShellExt;
use fig_proto::fig::{
    OnboardingAction,
    OnboardingRequest,
};
use fig_util::Shell;
use tokio::process::Command;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::event::Event;
use crate::{
    EventLoopProxy,
    MISSION_CONTROL_ID,
};

pub async fn onboarding(request: OnboardingRequest, proxy: &EventLoopProxy) -> RequestResult {
    match request.action() {
        OnboardingAction::InstallationScript => {
            let backup_dir = get_default_backup_dir().context("Failed to get backup dir")?;

            let mut errs: Vec<String> = vec![];
            for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
                match shell.get_shell_integrations() {
                    Ok(integrations) => {
                        for integration in integrations {
                            if let Err(err) = integration.install(Some(&backup_dir)) {
                                errs.push(format!("{integration}: {err}"));
                            }
                        }
                    },
                    Err(err) => {
                        errs.push(format!("{shell}: {err}"));
                    },
                }
            }

            match &errs[..] {
                [] => RequestResult::success(),
                errs => RequestResult::error(errs.join("\n")),
            }
        },
        OnboardingAction::Uninstall => {
            // TODO(grant): Move uninstall to a common lib and call directly
            match Command::new("fig")
                .args(&["_", "uninstall", "--dotfiles", "--daemon"])
                .output()
                .await
            {
                Ok(_) => RequestResult::success(),
                Err(err) => RequestResult::error(err.to_string()),
            }
        },
        OnboardingAction::FinishOnboarding => {
            match proxy.send_event(Event::WindowEvent {
                window_id: MISSION_CONTROL_ID,
                window_event: crate::event::WindowEvent::Resize {
                    width: 1030,
                    height: 720,
                },
            }) {
                Ok(_) => RequestResult::success(),
                Err(_) => RequestResult::error(""),
            }
        },
        OnboardingAction::LaunchShellOnboarding => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "linux")] {
                    // for terminal_executable in LINUX_TERMINALS.iter().flat_map(|term| term.executable_names()) {
                    //     if let Ok(terminal_executable_path) = which::which(terminal_executable) {
                    //         break
                    //     }
                    // }
                    RequestResult::error("Unimplemented")
                } else if #[cfg(target_os = "macos")] {
                    RequestResult::error("Unimplemented")
                } else if #[cfg(target_os = "windows")] {
                    // TODO(chay): impl auto launch of terminal
                    // start "" "%PROGRAMFILES%\\Git\\bin\\sh.exe" --login
                    RequestResult::error("Unimplemented")
                }
            }
        },
        OnboardingAction::PromptForAccessibilityPermission
        | OnboardingAction::CloseAccessibilityPromptWindow
        | OnboardingAction::RequestRestart
        | OnboardingAction::CloseInputMethodPromptWindow => RequestResult::error("Unimplemented"),
    }
}
