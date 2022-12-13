use std::fmt::Display;

use fig_integrations::shell::ShellExt;
use fig_integrations::ssh::SshIntegration;
use fig_integrations::Integration;
use fig_proto::fig::install_response::{
    InstallationStatus,
    Response,
};
use fig_proto::fig::result::Result as ProtoResultEnum;
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

#[allow(dead_code)]
async fn integration_status(integration: impl fig_integrations::Integration) -> ServerOriginatedSubMessage {
    ServerOriginatedSubMessage::InstallResponse(InstallResponse {
        response: Some(Response::InstallationStatus(match integration.is_installed().await {
            Ok(_) => InstallationStatus::Installed.into(),
            Err(_) => InstallationStatus::NotInstalled.into(),
        })),
    })
}

#[allow(dead_code)]
fn integration_unsupported() -> ServerOriginatedSubMessage {
    ServerOriginatedSubMessage::InstallResponse(InstallResponse {
        response: Some(Response::InstallationStatus(InstallationStatus::NotSupported.into())),
    })
}

fn integration_result(result: Result<(), impl Display>) -> ServerOriginatedSubMessage {
    ServerOriginatedSubMessage::InstallResponse(InstallResponse {
        response: Some(Response::Result(match result {
            Ok(()) => ProtoResult {
                result: ProtoResultEnum::Ok.into(),
                error: None,
            },
            Err(err) => ProtoResult {
                result: ProtoResultEnum::Error.into(),
                error: Some(err.to_string()),
            },
        })),
    })
}

pub async fn install(request: InstallRequest) -> RequestResult {
    let response = match (request.component(), request.action()) {
        (InstallComponent::Dotfiles, action) => {
            let mut errs: Vec<String> = vec![];
            for shell in Shell::all() {
                match shell.get_shell_integrations() {
                    Ok(integrations) => {
                        for integration in integrations {
                            let res = match action {
                                InstallAction::Install => integration.install().await,
                                InstallAction::Uninstall => integration.uninstall().await,
                                InstallAction::Status => integration.is_installed().await,
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

            match action {
                InstallAction::Install | InstallAction::Uninstall => integration_result(match &errs[..] {
                    [] => Ok(()),
                    errs => Err(errs.join("\n")),
                }),
                InstallAction::Status => ServerOriginatedSubMessage::InstallResponse(InstallResponse {
                    response: Some(Response::InstallationStatus(
                        if errs.is_empty() {
                            InstallationStatus::Installed
                        } else {
                            InstallationStatus::NotInstalled
                        }
                        .into(),
                    )),
                }),
            }
        },
        (InstallComponent::Ssh, action) => match SshIntegration::new() {
            Ok(ssh_integration) => match action {
                InstallAction::Install => integration_result(ssh_integration.install().await),
                InstallAction::Uninstall => integration_result(ssh_integration.uninstall().await),
                InstallAction::Status => integration_status(ssh_integration).await,
            },
            Err(err) => integration_result(Err(err)),
        },
        (InstallComponent::Ibus, _) => integration_result(Err("IBus install is legacy")),
        (InstallComponent::Accessibility, InstallAction::Install) => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    use macos_utils::accessibility::{
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
        (InstallComponent::Accessibility, InstallAction::Status) => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    use macos_utils::accessibility::accessibility_is_enabled;

                    ServerOriginatedSubMessage::InstallResponse(InstallResponse {
                        response: Some(Response::InstallationStatus(if accessibility_is_enabled() {
                            InstallationStatus::Installed.into()
                        } else {
                            InstallationStatus::NotInstalled.into()
                        })),
                    })
                } else {
                    integration_result(Err("Accessibility permissions cannot be queried"))
                }
            }
        },
        (InstallComponent::Accessibility, InstallAction::Uninstall) => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    integration_result(Ok::<(), &str>(()))
                } else {
                    integration_result(Err("Accessibility permissions cannot be queried"))
                }
            }
        },
        (InstallComponent::InputMethod, InstallAction::Install) => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    use fig_integrations::input_method::{
                        InputMethod,
                    };
                    use fig_integrations::Integration;

                    integration_result(match InputMethod::default().install().await {
                        Ok(_) => Ok(()),
                        Err(err) => Err(format!("Could not install input method: {err}")),
                    })
                } else {
                    integration_result(Err("Input method install is only supported on macOS"))
                }
            }
        },
        (InstallComponent::InputMethod, InstallAction::Uninstall) => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    use fig_integrations::input_method::{
                        InputMethod,
                        InputMethodError,
                    };
                    use fig_integrations::Error;
                    use fig_integrations::Integration;

                    integration_result(match InputMethod::default().uninstall().await {
                        Ok(_) | Err(Error::InputMethod(InputMethodError::CouldNotListInputSources)) => {
                            Ok(())
                        },
                        Err(err) => Err(format!("Could not uninstall input method: {err}")),
                    })
                } else {
                    integration_result(Err("Input method uninstall is only supported on macOS"))
                }
            }
        },
        (InstallComponent::InputMethod, InstallAction::Status) => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    use fig_integrations::input_method::{
                        InputMethod,
                    };

                    integration_status(InputMethod::default()).await
                } else {
                    integration_unsupported()
                }
            }
        },
    };

    RequestResult::Ok(Box::new(response))
}
