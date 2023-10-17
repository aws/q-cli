use fig_integrations::shell::ShellExt;
use fig_proto::fig::{
    OnboardingAction,
    OnboardingRequest,
};
use fig_util::Shell;
#[cfg(target_os = "macos")]
use tokio::process::Command;
use tracing::error;
use wry::application::event_loop::ControlFlow;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::event::{
    Event,
    WindowEvent,
    WindowPosition,
};
use crate::webview::DASHBOARD_INITIAL_SIZE;
use crate::{
    EventLoopProxy,
    DASHBOARD_ID,
};

// Sync all of the user's data from the server and restart the daemon after they log in
pub async fn post_login(proxy: &EventLoopProxy) {
    proxy.send_event(Event::ReloadTray).ok();
}

pub async fn onboarding(request: OnboardingRequest, proxy: &EventLoopProxy) -> RequestResult {
    match request.action() {
        OnboardingAction::InstallationScript => {
            let mut errs: Vec<String> = vec![];
            for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
                match shell.get_shell_integrations() {
                    Ok(integrations) => {
                        for integration in integrations {
                            if let Err(err) = integration.install().await {
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
            use fig_install::{
                uninstall,
                InstallComponents,
            };

            let url = fig_install::get_uninstall_url(false);
            fig_util::open_url(url).ok();

            let result = match uninstall(InstallComponents::all()).await {
                Ok(_) => RequestResult::success(),
                Err(err) => RequestResult::error(err.to_string()),
            };

            proxy.send_event(Event::ControlFlow(ControlFlow::Exit)).ok();
            result
        },
        OnboardingAction::FinishOnboarding => {
            post_login(proxy).await;

            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID,
                    window_event: WindowEvent::UpdateWindowGeometry {
                        position: Some(WindowPosition::Centered),
                        size: Some(DASHBOARD_INITIAL_SIZE),
                        anchor: None,
                        tx: None,
                        dry_run: false,
                    },
                })
                .ok();

            RequestResult::success()
        },
        OnboardingAction::LaunchShellOnboarding => {
            fig_settings::state::set_value("user.onboarding", false).ok();

            cfg_if::cfg_if! {
                if #[cfg(target_os = "linux")] {
                    use fig_util::terminal::LINUX_TERMINALS;

                    for terminal_executable in LINUX_TERMINALS.iter().flat_map(|term| term.executable_names()) {
                        if let Ok(terminal_executable_path) = which::which(terminal_executable) {
                            tokio::spawn(tokio::process::Command::new(terminal_executable_path).output());
                            return RequestResult::success();
                        }
                    }
                    RequestResult::error("Failed to open any terminal")
                } else if #[cfg(target_os = "macos")] {
                    let home = fig_util::directories::home_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));

                    if let Err(err) = Command::new("open").args(["-b", "com.apple.Terminal"]).arg(home).spawn() {
                        error!(%err, "Failed to open onboarding");
                        return RequestResult::error("Failed to open onboarding");
                    }

                    RequestResult::success()
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
        OnboardingAction::PromptForAccessibilityPermission => {
            use crate::local_ipc::{
                commands,
                LocalResponse,
            };
            let res = commands::prompt_for_accessibility_permission()
                .await
                .unwrap_or_else(|e| e);
            match res {
                LocalResponse::Success(_) => RequestResult::success(),
                LocalResponse::Error {
                    message: Some(message), ..
                } => RequestResult::error(message),
                _ => RequestResult::error("Failed to prompt for accessibility permissions"),
            }
        },
        OnboardingAction::PostLogin => {
            post_login(proxy).await;
            RequestResult::success()
        },
        OnboardingAction::CloseAccessibilityPromptWindow
        | OnboardingAction::RequestRestart
        | OnboardingAction::CloseInputMethodPromptWindow => RequestResult::error("Unimplemented"),
    }
}
