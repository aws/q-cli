use std::path::{
    Path,
    PathBuf,
};

use async_trait::async_trait;
use dbus::gnome_shell::{
    get_extension_status,
    ExtensionInstallationStatus,
    ShellExtensions,
};
use fig_os_shim::{
    EnvProvider,
    FsProvider,
    SysInfoProvider,
};

use crate::error::{
    Error,
    Result,
};
use crate::Integration;

#[derive(Debug, Clone)]
pub struct GnomeExtensionIntegration<'a, Ctx, ExtensionsCtx> {
    ctx: &'a Ctx,
    shell_extensions: &'a ShellExtensions<ExtensionsCtx>,
    bundle_path: PathBuf,
    version: u32,
}

impl<'a, Ctx, ExtensionsCtx> GnomeExtensionIntegration<'a, Ctx, ExtensionsCtx>
where
    Ctx: FsProvider + EnvProvider,
{
    pub fn new(
        ctx: &'a Ctx,
        shell_extensions: &'a ShellExtensions<ExtensionsCtx>,
        bundle_path: impl AsRef<Path>,
        version: u32,
    ) -> Self {
        Self {
            ctx,
            shell_extensions,
            bundle_path: bundle_path.as_ref().into(),
            version,
        }
    }
}

#[async_trait]
impl<Ctx, ExtensionsCtx> Integration for GnomeExtensionIntegration<'_, Ctx, ExtensionsCtx>
where
    Ctx: FsProvider + EnvProvider + SysInfoProvider + Sync,
    ExtensionsCtx: FsProvider + EnvProvider + SysInfoProvider + Send + Sync,
{
    fn describe(&self) -> String {
        "GNOME Extension Integration".to_owned()
    }

    async fn install(&self) -> Result<()> {
        if self.is_installed().await.is_ok() {
            return Ok(());
        }

        self.shell_extensions
            .install_bundled_extension(&self.bundle_path)
            .await?;

        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        self.shell_extensions.uninstall_extension().await?;
        Ok(())
    }

    async fn is_installed(&self) -> Result<()> {
        match get_extension_status(self.ctx, self.shell_extensions, Some(self.version)).await? {
            ExtensionInstallationStatus::GnomeShellNotRunning => Err(Error::Custom(
                "GNOME Shell is not running, cannot determine installation status".into(),
            )),
            ExtensionInstallationStatus::NotInstalled | ExtensionInstallationStatus::UnexpectedVersion { .. } => {
                Err(Error::NotInstalled(self.describe().into()))
            },
            ExtensionInstallationStatus::NotEnabled
            | ExtensionInstallationStatus::RequiresReboot
            | ExtensionInstallationStatus::Enabled => Ok(()),
        }
    }
}
