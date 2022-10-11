use fig_integrations::shell::ShellExt;
use fig_proto::fig::{
    OnboardingAction,
    OnboardingRequest,
};
use fig_util::{
    directories,
    Shell,
};
use tokio::process::Command;
use tracing::error;

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
            let backups_dir = directories::utc_backup_dir().map_err(|err| err.to_string())?;

            let mut errs: Vec<String> = vec![];
            for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
                match shell.get_shell_integrations() {
                    Ok(integrations) => {
                        for integration in integrations {
                            if let Err(err) = integration.install(Some(&backups_dir)) {
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
            // Sync all of the user's files when they finish onboarding
            tokio::spawn(async {
                // Settings has to be synced first because it contains information that might
                // modify the behavior of other syncs
                if let Err(err) = fig_api_client::settings::sync().await {
                    error!(%err, "Failed to sync settings");
                }

                tokio::spawn(async {
                    if let Err(err) = fig_sync::dotfiles::download_and_notify(false).await {
                        error!(%err, "Failed to download dotfiles");
                    }
                });

                tokio::spawn(async {
                    if let Err(err) = fig_sync::plugins::fetch_installed_plugins(false).await {
                        error!(%err, "Failed to fetch installed plugins");
                    }
                });
            });

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
            fig_settings::state::set_value("user.onboarding", false).ok();

            cfg_if::cfg_if! {
                if #[cfg(target_os = "linux")] {
                    use fig_util::terminal::LINUX_TERMINALS;

                    for terminal_executable in LINUX_TERMINALS.iter().flat_map(|term| term.executable_names()) {
                        if let Ok(terminal_executable_path) = which::which(terminal_executable) {
                            tokio::spawn(Command::new(terminal_executable_path).output());
                            return RequestResult::success();
                        }
                    }
                    RequestResult::error("Failed to open any terminal")
                } else if #[cfg(target_os = "macos")] {
                    RequestResult::error("Unimplemented")
                } else if #[cfg(target_os = "windows")] {
                    use std::os::windows::process::CommandExt;

                    let create_new_console = 0x10;
                    match std::process::Command::new("cmd").creation_flags(create_new_console).arg("/c").raw_arg(r#"""%PROGRAMFILES%/Git/bin/bash.exe"""#).spawn() {
                        Ok(_) => RequestResult::success(),
                        Err(e) => RequestResult::error(format!("Failed to start Git Bash: {e}")),
                    }
                }
            }
        },
        OnboardingAction::PromptForAccessibilityPermission
        | OnboardingAction::CloseAccessibilityPromptWindow
        | OnboardingAction::RequestRestart
        | OnboardingAction::CloseInputMethodPromptWindow => RequestResult::error("Unimplemented"),
    }
}
