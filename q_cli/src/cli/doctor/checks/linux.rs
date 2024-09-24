use std::borrow::Cow;
use std::process::Command;
use std::sync::Arc;

use async_trait::async_trait;
use dbus::gnome_shell::{
    ExtensionInstallationStatus,
    ShellExtensions,
    get_extension_status,
};
use fig_os_shim::Context;
use fig_util::consts::{
    CLI_BINARY_NAME,
    PRODUCT_NAME,
};
use fig_util::system_info::linux::{
    DesktopEnvironment,
    DisplayServer,
    get_desktop_environment,
    get_display_server,
};
use futures::FutureExt;
use owo_colors::OwoColorize;

use crate::cli::doctor::{
    DoctorCheck,
    DoctorCheckType,
    DoctorError,
    DoctorFix,
    Platform,
    doctor_error,
    doctor_fix,
    doctor_warning,
};

#[derive(Debug)]
pub struct LinuxContext {
    ctx: Arc<Context>,
    shell_extensions: Arc<ShellExtensions>,
}

impl LinuxContext {
    fn new(ctx: Arc<Context>, shell_extensions: Arc<ShellExtensions>) -> Self {
        Self { ctx, shell_extensions }
    }
}

impl Default for LinuxContext {
    fn default() -> Self {
        Self {
            ctx: Context::new(),
            shell_extensions: Default::default(),
        }
    }
}

impl From<Arc<Context>> for LinuxContext {
    fn from(ctx: Arc<Context>) -> Self {
        Self {
            ctx,
            ..Default::default()
        }
    }
}

pub async fn get_linux_context() -> eyre::Result<LinuxContext> {
    let ctx = Context::new();
    let ctx_clone = Arc::clone(&ctx);
    Ok(LinuxContext::new(ctx, ShellExtensions::new(ctx_clone).into()))
}

pub struct IBusEnvCheck;

#[async_trait]
impl DoctorCheck<LinuxContext> for IBusEnvCheck {
    fn name(&self) -> Cow<'static, str> {
        "IBus Env Check".into()
    }

    async fn get_type(&self, _: &LinuxContext, _: Platform) -> DoctorCheckType {
        DoctorCheckType::NormalCheck
    }

    async fn check(&self, ctx: &LinuxContext) -> Result<(), DoctorError> {
        let ctx = &ctx.ctx;

        #[derive(Debug)]
        struct EnvErr {
            var: Cow<'static, str>,
            actual: Option<String>,
            expected: Cow<'static, str>,
        }

        let env = ctx.env();
        let mut checks = vec![("QT_IM_MODULE", "ibus"), ("XMODIFIERS", "@im=ibus")];
        let mut errors: Vec<EnvErr> = vec![];
        match get_desktop_environment(ctx)? {
            DesktopEnvironment::Gnome => {
                // GNOME's default input method is ibus, so GTK_IM_MODULE is not required (and
                // may not set by default on Ubuntu).
                // Only error if it's set to something other than ibus.
                match env.get("GTK_IM_MODULE") {
                    Ok(actual) if actual != "ibus" => errors.push(EnvErr {
                        var: "ibus".into(),
                        actual: actual.into(),
                        expected: "ibus".into(),
                    }),
                    _ => (),
                }
            },
            _ => checks.push(("GTK_IM_MODULE", "ibus")),
        };
        errors.append(
            &mut checks
                .iter()
                .filter_map(|(var, expected)| match env.get(var) {
                    Ok(actual) if actual.contains(expected) => None,
                    Ok(actual) => Some(EnvErr {
                        var: (*var).into(),
                        actual: Some(actual),
                        expected: (*expected).into(),
                    }),
                    Err(_) => Some(EnvErr {
                        var: (*var).into(),
                        actual: None,
                        expected: (*expected).into(),
                    }),
                })
                .collect::<Vec<_>>(),
        );

        if !errors.is_empty() {
            let mut info = vec![
                "The input method is required to be configured for IBus in order for autocomplete to work.".into(),
            ];
            info.append(
                &mut errors
                    .iter()
                    .map(|err| {
                        if let Some(actual) = &err.actual {
                            format!("{} is '{}', expected '{}'", err.var, actual, err.expected).into()
                        } else {
                            format!("{} is not set, expected '{}'", err.var, err.expected).into()
                        }
                    })
                    .collect::<Vec<_>>(),
            );
            Err(DoctorError::Error {
                reason: "IBus environment variable is not set".into(),
                info,
                fix: None,
                error: None,
            })
        } else {
            Ok(())
        }
    }
}

pub struct GnomeExtensionCheck;

#[async_trait]
impl DoctorCheck<LinuxContext> for GnomeExtensionCheck {
    fn name(&self) -> Cow<'static, str> {
        "GNOME Extension Check".into()
    }

    async fn get_type(&self, _: &LinuxContext, _: Platform) -> DoctorCheckType {
        DoctorCheckType::NormalCheck
    }

    async fn check(&self, ctx: &LinuxContext) -> Result<(), DoctorError> {
        let (ctx, shell_extensions) = (Arc::clone(&ctx.ctx), Arc::clone(&ctx.shell_extensions));

        if get_desktop_environment(&ctx)? != DesktopEnvironment::Gnome {
            return Ok(());
        }

        match get_display_server(&ctx).unwrap() {
            DisplayServer::X11 => Ok(()),
            DisplayServer::Wayland => match get_extension_status(&ctx, &shell_extensions, None).await.map_err(eyre::Report::from)? {
                ExtensionInstallationStatus::GnomeShellNotRunning => Err(DoctorError::Error {
                    reason: format!(
                        "The gnome-shell process doesn't appear to be running. If you believe this is an error, please file an issue by running {}",
                        format!("{CLI_BINARY_NAME} issue").magenta()
                    ).into(),
                    info: vec![],
                    fix: None,
                    error: None,
                }),
                ExtensionInstallationStatus::NotInstalled => Err(DoctorError::Error {
                    reason: format!(
                        "The {PRODUCT_NAME} GNOME extension is not installed. Please restart the desktop app and try again."
                    ).into(),
                    info: vec![],
                    fix: None,
                    error: None,
                }),
                ExtensionInstallationStatus::RequiresReboot => Err(DoctorError::Error {
                    reason: format!(
                        "The {PRODUCT_NAME} GNOME extension is installed but not loaded. Please restart your login session and try again."
                    ).into(),
                    info: vec![],
                    fix: None,
                    error: None,
                }),
                // Should not match since we're currently not checking against the version here.
                ExtensionInstallationStatus::UnexpectedVersion { .. } => Err(DoctorError::Error {
                    reason: format!(
                        "The {PRODUCT_NAME} GNOME extension is currently outdated. Please restart the desktop app and try again."
                    ).into(),
                    info: vec![],
                    fix: None,
                    error: None,
                }),
                ExtensionInstallationStatus::NotEnabled => Err(DoctorError::Error {
                    reason: format!("The {PRODUCT_NAME} GNOME extension is not enabled.").into(),
                    info: vec![],
                    fix: Some(DoctorFix::Async(async move {
                        shell_extensions.enable_extension().await?;
                        Ok(())
                    }.boxed())),
                    error: None,
                }),
                ExtensionInstallationStatus::Enabled => Ok(()),
            },
        }
    }
}

pub struct IBusCheck;

#[async_trait]
impl DoctorCheck<LinuxContext> for IBusCheck {
    fn name(&self) -> Cow<'static, str> {
        "IBus Check".into()
    }

    async fn get_type(&self, _: &LinuxContext, _: Platform) -> DoctorCheckType {
        DoctorCheckType::NormalCheck
    }

    async fn check(&self, _: &LinuxContext) -> Result<(), DoctorError> {
        use sysinfo::{
            ProcessRefreshKind,
            RefreshKind,
        };

        let system = sysinfo::System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));

        if system.processes_by_exact_name("ibus-daemon").next().is_none() {
            return Err(doctor_fix!({
                reason: "ibus-daemon is not running",
                fix: || {
                    let output = Command::new("ibus-daemon").arg("-drxR").output()?;
                    if !output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eyre::bail!("ibus-daemon launch failed:\nstdout: {stdout}\nstderr: {stderr}\n");
                    }
                    Ok(())
            }}));
        }

        Ok(())
    }
}

pub struct SandboxCheck;

#[async_trait]
impl DoctorCheck<LinuxContext> for SandboxCheck {
    fn name(&self) -> Cow<'static, str> {
        "App is not running in a sandbox".into()
    }

    async fn check(&self, _: &LinuxContext) -> Result<(), DoctorError> {
        use fig_util::system_info::linux::SandboxKind;

        let kind = fig_util::system_info::linux::detect_sandbox();

        match kind {
            SandboxKind::None => Ok(()),
            SandboxKind::Flatpak => Err(doctor_error!("Running under Flatpak is not supported.")),
            SandboxKind::Snap => Err(doctor_error!("Running under Snap is not supported.")),
            SandboxKind::Docker => Err(doctor_warning!(
                "Support for Docker is in development. It may not work properly on your system."
            )),
            SandboxKind::Container(Some(engine)) => {
                Err(doctor_error!("Running under `{engine}` containers is not supported."))
            },
            SandboxKind::Container(None) => Err(doctor_error!("Running under non-docker containers is not supported.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use dbus::gnome_shell::GNOME_SHELL_PROCESS_NAME;
    use fig_os_shim::Env;

    use super::*;

    #[tokio::test]
    async fn test_ibus_env_check() {
        let ctx = Context::builder()
            .with_env(Env::from_slice(&[
                ("XDG_CURRENT_DESKTOP", "ubuntu:GNOME"),
                ("GTK_IM_MODULE", "ibus"),
                ("QT_IM_MODULE", "ibus"),
                ("XMODIFIERS", "@im=ibus"),
            ]))
            .build_fake();
        assert!(
            IBusEnvCheck.check(&ctx.into()).await.is_ok(),
            "should succeed with all env vars set"
        );

        let ctx = Context::builder()
            .with_env(Env::from_slice(&[
                ("XDG_CURRENT_DESKTOP", "ubuntu:GNOME"),
                ("QT_IM_MODULE", "ibus"),
                ("XMODIFIERS", "@im=ibus"),
            ]))
            .build_fake();
        assert!(
            IBusEnvCheck.check(&ctx.into()).await.is_ok(),
            "should succeed without GTK_IM_MODULE"
        );

        let ctx = Context::builder()
            .with_env(Env::from_slice(&[
                ("XDG_CURRENT_DESKTOP", "ubuntu:GNOME"),
                ("GTK_IM_MODULE", "gtk-im-context-simple"),
                ("QT_IM_MODULE", "simple"),
                ("XMODIFIERS", "@im=null"),
            ]))
            .build_fake();
        assert!(
            IBusEnvCheck.check(&ctx.into()).await.is_err(),
            "fail when input method is disabled"
        );

        let ctx = Context::builder()
            .with_env(Env::from_slice(&[
                ("XDG_CURRENT_DESKTOP", "fedora:KDE"),
                ("QT_IM_MODULE", "ibus"),
                ("XMODIFIERS", "@im=ibus"),
            ]))
            .build_fake();
        assert!(
            IBusEnvCheck.check(&ctx.into()).await.is_err(),
            "fail when missing GTK_IM_MODULE on non-gnome desktops"
        );

        let ctx = Context::builder()
            .with_env(Env::from_slice(&[("XDG_CURRENT_DESKTOP", "fedora:KDE")]))
            .build_fake();
        let err = IBusEnvCheck.check(&ctx.into()).await.unwrap_err();
        #[allow(clippy::match_wildcard_for_single_variants)]
        match err {
            DoctorError::Error { info, .. } => {
                let info = info.join("\n");
                for var in &["GTK_IM_MODULE", "QT_IM_MODULE", "XMODIFIERS"] {
                    assert!(
                        info.contains(var),
                        "error info should contain all env vars. Actual info: {}",
                        info
                    );
                }
            },
            _ => panic!("missing env vars should error"),
        }
    }

    #[tokio::test]
    async fn test_gnome_extension_check() {
        let ctx = Context::builder()
            .with_env(Env::from_slice(&[
                ("XDG_SESSION_TYPE", "x11"),
                ("XDG_CURRENT_DESKTOP", "ubuntu:GNOME"),
            ]))
            .build_fake();
        let check = GnomeExtensionCheck.check(&ctx.into()).await;
        assert!(
            check.is_ok(),
            "x11 on GNOME shouldn't require the extension. Error: {:?}",
            check
        );

        let ctx = Context::builder()
            .with_env(Env::from_slice(&[
                ("XDG_SESSION_TYPE", "wayland"),
                ("XDG_CURRENT_DESKTOP", "ubuntu:GNOME"),
            ]))
            .with_running_processes(&[GNOME_SHELL_PROCESS_NAME])
            .build_fake();
        let shell_extensions = ShellExtensions::new_fake(Arc::clone(&ctx));
        let check = GnomeExtensionCheck
            .check(&LinuxContext::new(ctx, shell_extensions.into()))
            .await;
        assert!(check.is_err(), "extension not installed should error");

        let ctx = Context::builder()
            .with_test_home()
            .await
            .unwrap()
            .with_env_var("XDG_SESSION_TYPE", "wayland")
            .with_env_var("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")
            .with_running_processes(&[GNOME_SHELL_PROCESS_NAME])
            .build_fake();
        let shell_extensions = ShellExtensions::new_fake(Arc::clone(&ctx));
        shell_extensions.install_for_fake(false, 1).await.unwrap();
        shell_extensions.enable_extension().await.unwrap();
        let check = GnomeExtensionCheck
            .check(&LinuxContext::new(ctx, shell_extensions.into()))
            .await;
        assert!(
            check.is_ok(),
            "extension installed, loaded, and enabled should not error. Error: {:?}",
            check
        );
    }
}
