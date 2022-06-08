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

pub async fn onboarding(request: OnboardingRequest) -> RequestResult {
    match request.action() {
        OnboardingAction::InstallationScript => {
            let backup_dir = get_default_backup_dir()?;

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
                .args(&["_", "uninstall", "--dotfiles"])
                .output()
                .await
            {
                Ok(_) => RequestResult::success(),
                Err(err) => RequestResult::error(err.to_string()),
            }
        },
        OnboardingAction::PromptForAccessibilityPermission
        | OnboardingAction::LaunchShellOnboarding
        | OnboardingAction::CloseAccessibilityPromptWindow
        | OnboardingAction::RequestRestart
        | OnboardingAction::CloseInputMethodPromptWindow => RequestResult::error("Unimplemented"),
    }
}
