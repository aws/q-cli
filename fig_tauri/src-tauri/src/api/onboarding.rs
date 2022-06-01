use fig_proto::fig::{
    OnboardingAction,
    OnboardingRequest,
};
use tokio::process::Command;

use super::{
    RequestResult,
    RequestResultImpl,
};

pub async fn onboarding(request: OnboardingRequest) -> RequestResult {
    match request.action() {
        OnboardingAction::InstallationScript => {
            // TODO(grant): Move install to a common lib and call directly
            match Command::new("fig")
                .args(&["_", "install", "--dotfiles", "--no-confirm"])
                .output()
                .await
            {
                Ok(_) => RequestResultImpl::success(),
                Err(err) => RequestResultImpl::error(err.to_string()),
            }
        },
        OnboardingAction::Uninstall => {
            // TODO(grant): Move uninstall to a common lib and call directly
            match Command::new("fig")
                .args(&["_", "uninstall", "--dotfiles"])
                .output()
                .await
            {
                Ok(_) => RequestResultImpl::success(),
                Err(err) => RequestResultImpl::error(err.to_string()),
            }
        },
        OnboardingAction::PromptForAccessibilityPermission
        | OnboardingAction::LaunchShellOnboarding
        | OnboardingAction::CloseAccessibilityPromptWindow
        | OnboardingAction::RequestRestart
        | OnboardingAction::CloseInputMethodPromptWindow => RequestResultImpl::error("Unimplemented"),
    }
}
