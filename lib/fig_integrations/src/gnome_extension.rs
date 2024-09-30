use std::path::{
    Path,
    PathBuf,
};

use async_trait::async_trait;
use dbus::gnome_shell::{
    ExtensionInstallationStatus,
    ShellExtensions,
    get_extension_status,
};
use fig_os_shim::{
    EnvProvider,
    FsProvider,
    SysInfoProvider,
};

use crate::Integration;
use crate::error::{
    Error,
    Result,
};

#[derive(Debug, Clone)]
pub struct GnomeExtensionIntegration<'a, Ctx, ExtensionsCtx> {
    ctx: &'a Ctx,
    shell_extensions: &'a ShellExtensions<ExtensionsCtx>,

    /// Path to a local extension zip. Required for installation.
    bundle_path: Option<PathBuf>,

    /// Version of the extension. When [Option::Some], performs an additional version check when
    /// running [Self::is_installed].
    version: Option<u32>,
}

impl<'a, Ctx, ExtensionsCtx> GnomeExtensionIntegration<'a, Ctx, ExtensionsCtx>
where
    Ctx: FsProvider + EnvProvider,
{
    pub fn new(
        ctx: &'a Ctx,
        shell_extensions: &'a ShellExtensions<ExtensionsCtx>,
        bundle_path: Option<impl AsRef<Path>>,
        version: Option<u32>,
    ) -> Self {
        Self {
            ctx,
            shell_extensions,
            bundle_path: bundle_path.map(|p| PathBuf::from(p.as_ref())),
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

        match &self.bundle_path {
            Some(bundle_path) => self.shell_extensions.install_bundled_extension(bundle_path).await?,
            None => {
                return Err(Error::Custom(
                    "Extension bundle path is required for installation.".into(),
                ));
            },
        };

        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        self.shell_extensions.uninstall_extension().await?;
        Ok(())
    }

    async fn is_installed(&self) -> Result<()> {
        match get_extension_status(self.ctx, self.shell_extensions, self.version).await? {
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
