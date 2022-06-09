use std::fmt::Display;

use fig_integrations::ibus::IbusIntegration;
use fig_integrations::shell::ShellExt;
use fig_integrations::{
    get_default_backup_dir,
    Integration,
};
use fig_proto::fig::install_response::{
    InstallationStatus,
    Response,
};
use fig_proto::fig::result::ResultEnum as ProtoResultEnum;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    InstallAction,
    InstallComponent,
    InstallRequest,
    InstallResponse,
    Result as ProtoResult,
};
use fig_util::Shell;

use super::RequestResult;

fn integration_status(integration: impl Integration) -> ServerOriginatedSubMessage {
    ServerOriginatedSubMessage::InstallResponse(InstallResponse {
        response: Some(Response::InstallationStatus(match integration.is_installed() {
            Ok(_) => InstallationStatus::InstallInstalled.into(),
            Err(_) => InstallationStatus::InstallInstalled.into(),
        })),
    })
}

fn integration_result(result: Result<(), impl Display>) -> ServerOriginatedSubMessage {
    ServerOriginatedSubMessage::InstallResponse(InstallResponse {
        response: Some(Response::Result(match result {
            Ok(()) => ProtoResult {
                result: ProtoResultEnum::ResultOk.into(),
                error: None,
            },
            Err(err) => ProtoResult {
                result: ProtoResultEnum::ResultError.into(),
                error: Some(err.to_string()),
            },
        })),
    })
}

pub async fn install(request: InstallRequest) -> RequestResult {
    let response = match (request.component(), request.action()) {
        (InstallComponent::Dotfiles, action @ (InstallAction::InstallAction | InstallAction::UninstallAction)) => {
            let mut errs: Vec<String> = vec![];
            for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
                match shell.get_shell_integrations() {
                    Ok(integrations) => {
                        for integration in integrations {
                            let res = match action {
                                InstallAction::InstallAction => {
                                    let backup_dir = get_default_backup_dir().unwrap();
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

            integration_result(match &errs[..] {
                [] => Ok(()),
                errs => Err(errs.join("\n")),
            })
        },
        (InstallComponent::Dotfiles, InstallAction::StatusAction) => {
            // TODO(grant): Add actual logic here!
            ServerOriginatedSubMessage::InstallResponse(InstallResponse {
                response: Some(Response::InstallationStatus(
                    InstallationStatus::InstallInstalled.into(),
                )),
            })
        },
        (InstallComponent::Ibus, InstallAction::InstallAction) => {
            let integration = IbusIntegration {};
            integration_result(integration.is_installed().or_else(|_| integration.install(None)))
        },
        (InstallComponent::Ibus, InstallAction::StatusAction) => integration_status(IbusIntegration {}),
        (InstallComponent::Ibus, InstallAction::UninstallAction) => {
            integration_result(Err("IBus cannot be uninstalled"))
        },
    };

    RequestResult::Ok(Box::new(response))
}
