use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{
    Path,
    PathBuf,
};
use std::str::FromStr;
use std::sync::{
    Arc,
    Weak,
};

use fig_os_shim::{
    EnvProvider,
    FsProvider,
    SysInfoProvider,
};
use fig_util::directories::{
    DirectoryError,
    home_dir_ctx,
};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::debug;
use zbus::proxy;
use zbus::zvariant::OwnedValue;

use super::session_bus;
use crate::CrateError;

pub const GNOME_SHELL_PROCESS_NAME: &str = "gnome-shell";

/// Extension uuid for GNOME Shell v44 and prior.
const LEGACY_EXTENSION_UUID: &str = "amazon-q-for-cli-legacy-gnome-integration@aws.amazon.com";

/// Extension uuid for GNOME Shell v45 and after.
const MODERN_EXTENSION_UUID: &str = "amazon-q-for-cli-gnome-integration@aws.amazon.com";

/// Represents the installation status for the Amazon Q CLI GNOME Shell extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionInstallationStatus {
    /// The GNOME Shell process is not running.
    ///
    /// This is a sort of error value where the installation status actually can't be checked since
    /// it requires communicating with the GNOME Shell dbus service.
    GnomeShellNotRunning,

    /// The extension is not installed.
    NotInstalled,

    /// The extension is installed, but not loaded into GNOME Shell's memory. The user must reboot
    /// their machine.
    RequiresReboot,

    /// The extension is installed but with an unexpected version.
    UnexpectedVersion { installed_version: u32 },

    /// The extension is installed but not enabled.
    NotEnabled,

    /// The extension is installed and enabled.
    Enabled,
}

async fn new_proxy() -> Result<ShellExtensionsProxy<'static>, CrateError> {
    Ok(ShellExtensionsProxy::new(session_bus().await?).await?)
}

/// Path to the directory containing the extension by the given `uuid`.
pub fn local_extension_directory<Ctx: FsProvider + EnvProvider>(
    ctx: &Ctx,
    uuid: &str,
) -> Result<PathBuf, ExtensionsError> {
    Ok(home_dir_ctx(ctx)?
        .join(".local/share/gnome-shell/extensions")
        .join(uuid))
}

/// Gets the installation status for the Amazon Q CLI GNOME Shell extension.
///
/// If `expected_version` is [Option::Some], then an additional version check is applied, in which
/// case [ExtensionInstallationStatus::UnexpectedVersion] may be returned.
pub async fn get_extension_status<Ctx, ExtensionsCtx>(
    ctx: &Ctx,
    shell_extensions: &ShellExtensions<ExtensionsCtx>,
    expected_version: Option<u32>,
) -> Result<ExtensionInstallationStatus, ExtensionsError>
where
    Ctx: FsProvider + EnvProvider + SysInfoProvider,
    ExtensionsCtx: FsProvider + EnvProvider + SysInfoProvider,
{
    if !shell_extensions.is_gnome_shell_running().await? {
        return Ok(ExtensionInstallationStatus::GnomeShellNotRunning);
    }

    // This could mean the extension is *technically* installed but just not loaded into
    // gnome shell's js jit, or the extension literally is not installed.
    //
    // As a check, see if the user's local directory contains the extension UUID.
    // If so, they need to reboot.
    if !shell_extensions.is_extension_loaded().await? {
        let uuid = shell_extensions.extension_uuid().await?;
        let local_extension_path = local_extension_directory(ctx, &uuid)?;
        if ctx.fs().exists(&local_extension_path) {
            // The user could still have an old extension installed, so parse the metadata.json to
            // check the version, returning "NotInstalled" if we run into any errors.
            let metadata_path = local_extension_path.join("metadata.json");
            debug!("checking: {}", &metadata_path.to_string_lossy());
            match ctx.fs().read_to_string(metadata_path).await {
                Ok(metadata) => {
                    let metadata: ExtensionMetadata = match serde_json::from_str(&metadata) {
                        Ok(metadata) => metadata,
                        Err(_) => return Ok(ExtensionInstallationStatus::NotInstalled),
                    };
                    if let Some(expected_version) = expected_version {
                        if metadata.version != expected_version {
                            return Ok(ExtensionInstallationStatus::UnexpectedVersion {
                                installed_version: metadata.version,
                            });
                        }
                    }
                },
                Err(_) => return Ok(ExtensionInstallationStatus::NotInstalled),
            }

            // All other cases means the extension is installed and we just have to reboot.
            return Ok(ExtensionInstallationStatus::RequiresReboot);
        }

        return Ok(ExtensionInstallationStatus::NotInstalled);
    }

    let mut info = shell_extensions.get_extension_info().await?;
    debug!("Found extension info: {:?}", info);
    if let Some(expected_version) = expected_version {
        let installed_version = f64::try_from(
            info.remove("version")
                .ok_or(ExtensionsError::Other("missing extension version".into()))?,
        )? as u32;
        if installed_version != expected_version {
            return Ok(ExtensionInstallationStatus::UnexpectedVersion { installed_version });
        }
    }

    if !shell_extensions.is_extension_enabled().await? {
        return Ok(ExtensionInstallationStatus::NotEnabled);
    }

    Ok(ExtensionInstallationStatus::Enabled)
}

/// Provides an accessible interface to retrieving info about the Amazon Q GNOME Shell Extension.
#[derive(Debug)]
pub struct ShellExtensions<Ctx> {
    inner: inner::Inner<Ctx>,
}

mod inner {
    use std::sync::{
        Arc,
        Weak,
    };

    use tokio::sync::Mutex;

    use super::*;

    #[derive(Debug)]
    pub enum Inner<Ctx> {
        Real(Weak<Ctx>),
        Fake(Arc<Mutex<Fake<Ctx>>>),
    }

    #[derive(Debug)]
    pub struct Fake<Ctx> {
        pub(super) ctx: Weak<Ctx>,
        pub version: GnomeShellVersion,
        pub extension_info: HashMap<String, OwnedValue>,
    }

    impl<Ctx> Fake<Ctx> {
        pub fn extension_info(&self) -> HashMap<String, OwnedValue> {
            self.extension_info
                .iter()
                .map(|(k, v)| (k.clone(), v.try_clone().unwrap()))
                .collect::<_>()
        }
    }
}

impl<Ctx> ShellExtensions<Ctx>
where
    Ctx: FsProvider + EnvProvider + SysInfoProvider,
{
    /// Creates a new real GNOME Shell extension client.
    ///
    /// Takes a [`Weak`] pointer since this enables [`ShellExtensions`] to be embedded with a
    /// [`fig_os_shim::Context`].
    pub fn new(ctx: Weak<Ctx>) -> Self {
        Self {
            inner: inner::Inner::Real(ctx),
        }
    }

    /// Creates a new fake shell extension client, returning GNOME Shell v45 and the extension as
    /// not installed.
    pub fn new_fake(ctx: Weak<Ctx>) -> Self {
        Self {
            inner: inner::Inner::Fake(Arc::new(Mutex::new(inner::Fake {
                ctx,
                version: GnomeShellVersion {
                    major: 45,
                    minor: "0".to_string(),
                },
                extension_info: HashMap::new(),
            }))),
        }
    }

    /// Returns the version of the system's GNOME Shell.
    pub async fn gnome_shell_version(&self) -> Result<GnomeShellVersion, ExtensionsError> {
        use inner::Inner;
        match &self.inner {
            Inner::Real(_) => new_proxy().await?.shell_version().await?.parse(),
            Inner::Fake(fake) => Ok(fake.lock().await.version.clone()),
        }
    }

    pub async fn get_extension_info(&self) -> Result<HashMap<String, OwnedValue>, ExtensionsError> {
        use inner::Inner;
        match &self.inner {
            Inner::Real(_) => Ok(new_proxy()
                .await?
                .get_extension_info(&self.extension_uuid().await?)
                .await?),
            Inner::Fake(fake) => Ok(fake.lock().await.extension_info()),
        }
    }

    /// Returns the UUID (ie, extension name) of the Amazon Q extension intended for the
    /// current system.
    pub async fn extension_uuid(&self) -> Result<String, ExtensionsError> {
        if self.gnome_shell_version().await?.major >= 45 {
            Ok(MODERN_EXTENSION_UUID.to_string())
        } else {
            Ok(LEGACY_EXTENSION_UUID.to_string())
        }
    }

    /// Uninstall the currently installed Amazon Q extension. Returns a bool indicating
    /// whether or not the extension was uninstalled.
    ///
    /// Note that this means Ok(false) is returned in the case where the extension was
    /// not installed.
    pub async fn uninstall_extension(&self) -> Result<bool, ExtensionsError> {
        use inner::Inner;
        match &self.inner {
            Inner::Real(ctx) => {
                let uuid = self.extension_uuid().await?;
                let mut was_uninstalled = new_proxy()
                    .await?
                    .uninstall_extension(&self.extension_uuid().await?)
                    .await?;

                // There might be an edgecase with the GNOME Shell dbus api where uninstalling an extension that's
                // not already loaded in GNOME Shell's js engine causes the extension to not
                // actually be uninstalled, so remove the extension directory if it still exists.
                let ctx = ctx.upgrade().ok_or(ExtensionsError::InvalidContext)?;
                let extension_path = local_extension_directory(ctx.as_ref(), &uuid)?;
                if ctx.fs().exists(&extension_path) {
                    ctx.fs().remove_dir_all(&extension_path).await?;
                    was_uninstalled = true;
                }

                Ok(was_uninstalled)
            },
            Inner::Fake(fake) => {
                // Attempt to mimic what the real implementation does:
                // Clear local extension directory if it exists.
                {
                    let ctx = self.ctx().await?;
                    let uuid = self.extension_uuid().await?;
                    let extension_path = local_extension_directory(ctx.as_ref(), &uuid)?;
                    if ctx.fs().exists(&extension_path) {
                        ctx.fs().remove_dir_all(&extension_path).await?;
                    }
                }
                // If keys were still present, then it means we hadn't uninstalled yet, in which
                // case return `Ok(true)`.
                let mut fake = fake.lock().await;
                if fake.extension_info.keys().count() > 0 {
                    fake.extension_info.clear();
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
        }
    }

    /// Installs an extension bundle from a zip file.
    ///
    /// The Fake implementation assumes that the provided path is just a text file with the
    /// extension version as its contents.
    #[allow(clippy::await_holding_lock)]
    pub async fn install_bundled_extension(&self, path: impl AsRef<Path>) -> Result<(), ExtensionsError> {
        use inner::Inner;
        match &self.inner {
            Inner::Real(_) => {
                let output = Command::new("gnome-extensions")
                    .arg("install")
                    .arg(path.as_ref())
                    .arg("--force")
                    .output()
                    .await?;

                if output.status.success() {
                    Ok(())
                } else {
                    Err(ExtensionsError::Other(
                        format!(
                            "Unable to install extension. gnome-extensions install stderr: {}",
                            String::from_utf8_lossy(&output.stderr)
                        )
                        .into(),
                    ))
                }
            },
            Inner::Fake(_) => {
                if self.ctx().await?.fs().exists(&path) {
                    let version: u32 = self
                        .ctx()
                        .await?
                        .fs()
                        .read_to_string(&path)
                        .await
                        .unwrap()
                        .parse()
                        .unwrap();
                    self.write_fake_extension_to_fs(version).await?;
                    Ok(())
                } else {
                    Err(ExtensionsError::Other(
                        format!("extension path does not exist: {}", &path.as_ref().to_string_lossy()).into(),
                    ))
                }
            },
        }
    }

    async fn is_gnome_shell_running(&self) -> Result<bool, ExtensionsError> {
        Ok(self.ctx().await?.sysinfo().is_process_running(GNOME_SHELL_PROCESS_NAME))
    }

    /// Whether or not the extension is loaded into GNOME Shell.
    ///
    /// Note that this is not an indicator of whether or not the extension is installed! The
    /// extension may be installed but not loaded into GNOME Shell, in which case GNOME Shell
    /// must be restarted.
    async fn is_extension_loaded(&self) -> Result<bool, ExtensionsError> {
        let info = self.get_extension_info().await?;
        if info.keys().count() == 0 { Ok(false) } else { Ok(true) }
    }

    /// Whether or not the extension is enabled.
    ///
    /// A prerequisite of being enabled is being installed *and* loaded into GNOME Shell.
    async fn is_extension_enabled(&self) -> Result<bool, ExtensionsError> {
        let mut info = self.get_extension_info().await?;
        let state = f64::try_from(
            info.remove("state")
                .ok_or(ExtensionsError::Other("missing extension state".into()))?,
        )? as u32;
        // Extension is enabled if "state" equals 1. If "state" equals 2, then it's
        // disabled.
        if state == 2 { Ok(false) } else { Ok(true) }
    }

    /// Returns a bool indicating whether or not the Amazon Q extension was successfully enabled.
    ///
    /// Return value behavior:
    /// - Extension not installed -> `Ok(false)`
    /// - Extension installed but not loaded -> `Ok(false)`
    /// - Extension installed and already enabled -> `Ok(false)`
    /// - Otherwise, `Ok(true)`
    pub async fn enable_extension(&self) -> Result<bool, ExtensionsError> {
        use inner::Inner;
        match &self.inner {
            Inner::Real(_) => Ok(new_proxy()
                .await?
                .enable_extension(&self.extension_uuid().await?)
                .await?),
            Inner::Fake(fake) => {
                if self.is_extension_loaded().await? {
                    fake.lock()
                        .await
                        .extension_info
                        .insert("state".to_string(), OwnedValue::from(1f64));
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
        }
    }

    async fn ctx(&self) -> Result<Arc<Ctx>, ExtensionsError> {
        use inner::Inner;
        match &self.inner {
            Inner::Real(ctx) => Ok(ctx.upgrade().ok_or(ExtensionsError::InvalidContext)?),
            Inner::Fake(fake) => Ok(fake.lock().await.ctx.upgrade().ok_or(ExtensionsError::InvalidContext)?),
        }
    }

    /// Test helper for the fake impl that installs an extension locally, optionally loading it if
    /// `requires_reboot` is false.
    ///
    /// It is not enabled.
    #[allow(dead_code)]
    pub async fn install_for_fake(&self, requires_reboot: bool, version: u32) -> Result<(), ExtensionsError> {
        if let inner::Inner::Fake(fake) = &self.inner {
            self.write_fake_extension_to_fs(version).await?;
            if !requires_reboot {
                fake.lock().await.extension_info = [
                    ("version".to_string(), OwnedValue::from(version as f64)),
                    ("state".to_string(), OwnedValue::from(2f64)),
                ]
                .into_iter()
                .collect();
            }
        }
        Ok(())
    }

    /// Test helper that creates the extension directory locally under [local_extension_directory],
    /// writing the metadata.json with the provided `version`.
    #[allow(dead_code)]
    async fn write_fake_extension_to_fs(&self, version: u32) -> Result<(), ExtensionsError> {
        if let inner::Inner::Fake(_) = &self.inner {
            let uuid = self.extension_uuid().await?;
            let ctx = self.ctx().await?;
            let extension_dir_path = local_extension_directory(ctx.as_ref(), &uuid)?;
            ctx.fs().create_dir_all(&extension_dir_path).await.ok();
            ctx.fs()
                .write(
                    extension_dir_path.join("metadata.json"),
                    json!({ "version": version }).to_string(),
                )
                .await
                .unwrap();
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ExtensionsError {
    #[error(transparent)]
    CrateError(#[from] CrateError),
    #[error(transparent)]
    StdIo(#[from] std::io::Error),
    #[error(transparent)]
    Zbus(#[from] zbus::Error),
    #[error(transparent)]
    ZVariant(#[from] zbus::zvariant::Error),
    #[error("Invalid major version: {0:?}")]
    InvalidMajorVersion(#[from] std::num::ParseIntError),
    #[error(transparent)]
    DirectoryError(#[from] DirectoryError),
    #[error("Invalid Context reference")]
    InvalidContext,
    #[error("Error: {0:?}")]
    Other(Cow<'static, str>),
}

#[proxy(
    default_service = "org.gnome.Shell.Extensions",
    interface = "org.gnome.Shell.Extensions",
    default_path = "/org/gnome/Shell/Extensions"
)]
trait ShellExtensions {
    /// ListExtensions method
    fn list_extensions(&self) -> zbus::Result<HashMap<String, HashMap<String, OwnedValue>>>;

    /// InstallRemoteExtension method
    fn install_remote_extension(&self, uuid: &str) -> zbus::Result<String>;

    #[zbus(property)]
    fn shell_version(&self) -> zbus::Result<String>;

    /// GetExtensionInfo method
    fn get_extension_info(&self, uuid: &str) -> zbus::Result<HashMap<String, OwnedValue>>;

    /// UninstallExtension method
    fn uninstall_extension(&self, uuid: &str) -> zbus::Result<bool>;

    /// EnableExtension method
    fn enable_extension(&self, uuid: &str) -> zbus::Result<bool>;
}

/// Represents a version of the system's GNOME Shell.
///
/// GNOME Shell versioning scheme taken from this post: <https://discourse.gnome.org/t/new-gnome-versioning-scheme/4235>
#[derive(Debug, Clone)]
pub struct GnomeShellVersion {
    pub major: u32,
    pub minor: String,
}

impl FromStr for GnomeShellVersion {
    type Err = ExtensionsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(".");
        let major = split.next().unwrap();
        let minor = split.next().unwrap();
        Ok(Self {
            major: major
                .to_string()
                .parse()
                .map_err(ExtensionsError::InvalidMajorVersion)?,
            minor: minor.to_string(),
        })
    }
}

/// The metadata.json file distributed with every GNOME Shell extension.
#[derive(Debug, Deserialize)]
struct ExtensionMetadata {
    version: u32,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fig_os_shim::Context;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_fake_impl_behavior_on_install_bundled_extension() {
        let ctx = Context::builder()
            .with_test_home()
            .await
            .unwrap()
            .with_running_processes(&[GNOME_SHELL_PROCESS_NAME])
            .build_fake();
        let shell_extensions = ShellExtensions::new_fake(Arc::downgrade(&ctx));

        // Default status is not installed
        let status = get_extension_status(&ctx, &shell_extensions, Some(1)).await.unwrap();
        assert_eq!(status, ExtensionInstallationStatus::NotInstalled);

        // Installing will require a reboot
        let extension_version = 1;
        let extension_bundle_path = PathBuf::from_str("extension.zip").unwrap();
        ctx.fs()
            .write(&extension_bundle_path, extension_version.to_string())
            .await
            .unwrap();
        shell_extensions
            .install_bundled_extension(&extension_bundle_path)
            .await
            .unwrap();
        let status = get_extension_status(&ctx, &shell_extensions, Some(extension_version))
            .await
            .unwrap();
        assert_eq!(status, ExtensionInstallationStatus::RequiresReboot);
    }

    mod extension_status_tests {
        use super::*;

        async fn make_ctx() -> Arc<Context> {
            Context::builder()
                .with_test_home()
                .await
                .unwrap()
                .with_running_processes(&[GNOME_SHELL_PROCESS_NAME])
                .build_fake()
        }

        #[test]
        fn test_extension_metadata_deser() {
            let metadata = r#"
            {
              "uuid": "amazon-q-for-cli-legacy-gnome-integration@aws.amazon.com",
              "name": "Amazon Q for CLI GNOME Integration",
              "url": "https://github.com/aws",
              "version": 1,
              "description": "Integrates Amazon Q for CLI with GNOME Shell prior to v45",
              "gettext-domain": "amazon-q-for-cli-legacy-gnome-integration",
              "settings-schema": "org.gnome.shell.extensions.amazon-q-for-cli-legacy-gnome-integration",
              "shell-version": ["41", "42", "43", "44"]
            }"#;
            let metadata: ExtensionMetadata = serde_json::from_str(metadata).unwrap();
            assert_eq!(metadata.version, 1);
        }

        #[tokio::test]
        async fn test_extension_status_when_gnome_shell_not_running() {
            let ctx = Context::builder().with_test_home().await.unwrap().build_fake();
            let shell_extensions = ShellExtensions::new_fake(Arc::downgrade(&ctx));

            // When
            let status = get_extension_status(&ctx, &shell_extensions, Some(1)).await.unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::GnomeShellNotRunning);
        }

        #[tokio::test]
        async fn test_extension_status_when_empty_info() {
            let ctx = make_ctx().await;
            let shell_extensions = ShellExtensions::new_fake(Arc::downgrade(&ctx));

            // When
            let status = get_extension_status(&ctx, &shell_extensions, Some(1)).await.unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::NotInstalled);
        }

        #[tokio::test]
        async fn test_extension_installed_but_not_loaded() {
            tracing_subscriber::fmt::try_init().ok();

            let ctx = make_ctx().await;
            let shell_extensions = ShellExtensions::new_fake(Arc::downgrade(&ctx));
            let expected_version = 1;
            shell_extensions.install_for_fake(true, expected_version).await.unwrap();

            // When
            let status = get_extension_status(&ctx, &shell_extensions, Some(expected_version))
                .await
                .unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::RequiresReboot);
        }

        #[tokio::test]
        async fn test_extension_installed_but_not_loaded_with_different_version() {
            tracing_subscriber::fmt::try_init().ok();

            let ctx = make_ctx().await;
            let shell_extensions = ShellExtensions::new_fake(Arc::downgrade(&ctx));
            let extension_dir_path = ctx
                .env()
                .home()
                .unwrap()
                .join(".local/share/gnome-shell/extensions")
                .join(shell_extensions.extension_uuid().await.unwrap());
            ctx.fs().create_dir_all(&extension_dir_path).await.unwrap();
            let expected_version: u32 = 2;
            ctx.fs()
                .write(
                    extension_dir_path.join("metadata.json"),
                    json!({ "version": 1 }).to_string(),
                )
                .await
                .unwrap();

            // When
            let status = get_extension_status(&ctx, &shell_extensions, Some(expected_version))
                .await
                .unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::UnexpectedVersion {
                installed_version: 1
            });
        }

        #[tokio::test]
        async fn test_extension_installed_with_different_version() {
            tracing_subscriber::fmt::try_init().ok();

            let ctx = make_ctx().await;
            let shell_extensions = ShellExtensions::new_fake(Arc::downgrade(&ctx));
            let expected_version = 2;
            let installed_version = 1;
            shell_extensions
                .install_for_fake(false, installed_version)
                .await
                .unwrap();

            // When
            let status = get_extension_status(&ctx, &shell_extensions, Some(expected_version))
                .await
                .unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::UnexpectedVersion {
                installed_version
            });
        }

        #[tokio::test]
        async fn test_extension_installed_but_not_enabled() {
            tracing_subscriber::fmt::try_init().ok();

            let ctx = make_ctx().await;
            let shell_extensions = ShellExtensions::new_fake(Arc::downgrade(&ctx));
            let expected_version = 2;
            shell_extensions
                .install_for_fake(false, expected_version)
                .await
                .unwrap();

            // When
            let status = get_extension_status(&ctx, &shell_extensions, Some(expected_version))
                .await
                .unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::NotEnabled);
        }

        #[tokio::test]
        async fn test_extension_installed_and_enabled() {
            tracing_subscriber::fmt::try_init().ok();

            let ctx = make_ctx().await;
            let shell_extensions = ShellExtensions::new_fake(Arc::downgrade(&ctx));
            let expected_version = 2;
            shell_extensions
                .install_for_fake(false, expected_version)
                .await
                .unwrap();
            shell_extensions.enable_extension().await.unwrap();

            // When
            let status = get_extension_status(&ctx, &shell_extensions, Some(expected_version))
                .await
                .unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::Enabled);
        }
    }

    mod e2e {
        use super::*;

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_gnome_shell_version() {
            let ctx = Context::new();
            println!(
                "{:?}",
                ShellExtensions::new(Arc::downgrade(&ctx))
                    .gnome_shell_version()
                    .await
                    .unwrap()
            );
        }

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_get_extension_info() {
            let ctx = Context::new();
            let uuid = ShellExtensions::new(Arc::downgrade(&ctx))
                .extension_uuid()
                .await
                .unwrap();
            let info = new_proxy().await.unwrap().get_extension_info(&uuid).await.unwrap();
            println!("{:?}", info);
        }

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_get_installed_extension_status() {
            let ctx = Context::new();
            let shell_extensions = ShellExtensions::new(Arc::downgrade(&ctx));
            let status = get_extension_status(&ctx, &shell_extensions, Some(1)).await.unwrap();
            println!("{:?}", status);
        }

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_uninstall_extension() {
            let ctx = Context::new();
            let res = ShellExtensions::new(Arc::downgrade(&ctx))
                .uninstall_extension()
                .await
                .unwrap();
            println!("{:?}", res);
        }

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_install_bundled_extension() {
            let ctx = Context::new();
            let path = PathBuf::from("");
            let res = ShellExtensions::new(Arc::downgrade(&ctx))
                .install_bundled_extension(path)
                .await;
            println!("{:?}", res);
        }

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_enable_extension() {
            let ctx = Context::new();
            let res = ShellExtensions::new(Arc::downgrade(&ctx)).enable_extension().await;
            println!("{:?}", res);
        }
    }
}
