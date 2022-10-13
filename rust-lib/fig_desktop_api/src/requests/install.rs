use std::fmt::Display;

use fig_integrations::shell::ShellExt;
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
use fig_util::directories::utc_backup_dir;
use fig_util::Shell;

use super::RequestResult;

#[allow(dead_code)]
fn integration_status(integration: impl fig_integrations::Integration) -> ServerOriginatedSubMessage {
    ServerOriginatedSubMessage::InstallResponse(InstallResponse {
        response: Some(Response::InstallationStatus(match integration.is_installed() {
            Ok(_) => InstallationStatus::InstallInstalled.into(),
            Err(_) => InstallationStatus::InstallNotInstalled.into(),
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
                                InstallAction::InstallAction => integration.install(utc_backup_dir().ok().as_deref()),
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
        (InstallComponent::Ibus, _) => integration_result(Err("IBus install is legacy")),
        (InstallComponent::Accessibility, InstallAction::InstallAction) => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    use macos_accessibility_position::accessibility::{
                        open_accessibility,
                        accessibility_is_enabled
                    };

                    if !accessibility_is_enabled() {
                        open_accessibility();

                        tokio::spawn(async move {
                            fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
                                fig_telemetry::TrackEventType::PromptedForAXPermission,
                                fig_telemetry::TrackSource::Desktop,
                                env!("CARGO_PKG_VERSION").into(),
                                std::iter::empty::<(&str, &str)>(),
                            ))
                            .await
                            .ok();
                        });
                    }

                    integration_result(Ok::<(), &str>(()))
                } else {
                    integration_result(Err("Accessibility permissions cannot be queried"))
                }
            }
        },
        (InstallComponent::Accessibility, InstallAction::StatusAction) => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    use macos_accessibility_position::accessibility::accessibility_is_enabled;

                    ServerOriginatedSubMessage::InstallResponse(InstallResponse {
                        response: Some(Response::InstallationStatus(if accessibility_is_enabled() {
                            InstallationStatus::InstallInstalled.into()
                        } else {
                            InstallationStatus::InstallNotInstalled.into()
                        })),
                    })
                } else {
                    integration_result(Err("Accessibility permissions cannot be queried"))
                }
            }
        },
        (InstallComponent::Accessibility, InstallAction::UninstallAction) => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    integration_result(Ok::<(), &str>(()))
                } else {
                    integration_result(Err("Accessibility permissions cannot be queried"))
                }
            }
        },
    };

    RequestResult::Ok(Box::new(response))
}
