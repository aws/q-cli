use fig_integrations::get_default_backup_dir;
use fig_integrations::shell::ShellExt;
use fig_proto::fig::install_response::InstallationStatus;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    InstallAction,
    InstallComponent,
    InstallRequest,
    InstallResponse,
};
use fig_util::Shell;

use super::{
    RequestResult,
    RequestResultImpl,
};

pub async fn install(request: InstallRequest) -> RequestResult {
    #[allow(unreachable_patterns)]
    match (request.component(), request.action()) {
        (InstallComponent::Dotfiles, action @ (InstallAction::InstallAction | InstallAction::UninstallAction)) => {
            let mut errs: Vec<String> = vec![];
            for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
                match shell.get_shell_integrations() {
                    Ok(integrations) => {
                        for integration in integrations {
                            let res = match action {
                                InstallAction::InstallAction => {
                                    let backup_dir = get_default_backup_dir()?;
                                    integration.install(Some(&backup_dir))
                                },
                                InstallAction::UninstallAction => integration.uninstall(),
                                InstallAction::StatusAction => unreachable!(),
                            };

                            if let Err(err) = res {
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
        (InstallComponent::Dotfiles, InstallAction::StatusAction) => {
            RequestResult::Ok(Box::new(ServerOriginatedSubMessage::InstallResponse(InstallResponse {
                installation_status: InstallationStatus::InstallInstalled.into(),
            })))
        },
        (InstallComponent::Ibus, InstallAction::InstallAction) => todo!(),
        (InstallComponent::Ibus, InstallAction::StatusAction) => todo!(),
        (InstallComponent::Ibus, InstallAction::UninstallAction) => RequestResult::error("IBus cannot be uninstalled"),
    }
}
