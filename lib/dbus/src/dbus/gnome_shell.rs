use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use fig_os_shim::Context;
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

const GNOME_SHELL_PROCESS_NAME: &str = "gnome-shell";

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

/// Gets the installation status for the Amazon Q CLI GNOME Shell extension.
pub async fn get_extension_status(
    ctx: &Context,
    shell_extensions: &ShellExtensions,
    expected_version: u32,
) -> Result<ExtensionInstallationStatus, ExtensionsError> {
    if !shell_extensions.is_gnome_shell_running().await? {
        return Ok(ExtensionInstallationStatus::GnomeShellNotRunning);
    }

    let mut info = shell_extensions.get_extension_info().await?;
    debug!("Found extension info: {:?}", info);
    let info_key_count = info.keys().count();
    // This could mean the extension is *technically* installed but just not loaded into
    // gnome shell's js jit, or the extension literally is not installed.
    //
    // As a check, see if the user's local directory contains the extension UUID.
    // If so, they need to reboot.
    if info_key_count == 0 {
        let uuid = shell_extensions.extension_uuid().await?;
        let local_extension_path = ctx
            .env()
            .home()
            .ok_or(ExtensionsError::Other("no home directory".into()))?
            .join(".local/share/gnome-shell/extensions")
            .join(uuid);
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
                    if metadata.version != expected_version {
                        return Ok(ExtensionInstallationStatus::UnexpectedVersion {
                            installed_version: metadata.version,
                        });
                    }
                },
                Err(_) => return Ok(ExtensionInstallationStatus::NotInstalled),
            }

            // All other cases means the extension is installed and we just have to reboot.
            return Ok(ExtensionInstallationStatus::RequiresReboot);
        }

        return Ok(ExtensionInstallationStatus::NotInstalled);
    }

    let installed_version = f64::try_from(
        info.remove("version")
            .ok_or(ExtensionsError::Other("missing extension version".into()))?,
    )? as u32;
    if installed_version != expected_version {
        return Ok(ExtensionInstallationStatus::UnexpectedVersion { installed_version });
    }

    // Extension is enabled if "state" equals 1. If "state" equals 2, then it's
    // disabled.
    let state = f64::try_from(
        info.remove("state")
            .ok_or(ExtensionsError::Other("missing extension state".into()))?,
    )? as u32;
    if state == 2 {
        return Ok(ExtensionInstallationStatus::NotEnabled);
    }

    Ok(ExtensionInstallationStatus::Enabled)
}

/// Provides an accessible interface to retrieving info about the Amazon Q GNOME Shell Extension.
#[derive(Debug)]
pub struct ShellExtensions(inner::Inner);

mod inner {
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use super::*;

    #[derive(Debug)]
    pub enum Inner {
        Real(Arc<Context>),
        Fake(Arc<Mutex<Fake>>),
    }

    #[derive(Debug)]
    pub struct Fake {
        pub(super) ctx: Arc<Context>,
        pub version: GnomeShellVersion,
        pub installed_version: Option<u32>,
        pub extension_info: HashMap<String, OwnedValue>,
        pub extension_enabled: bool,
    }

    impl Default for Fake {
        fn default() -> Self {
            Self {
                ctx: Context::new_fake(),
                version: GnomeShellVersion {
                    major: 45,
                    minor: "0".to_string(),
                },
                extension_info: HashMap::new(),
                installed_version: None,
                extension_enabled: false,
            }
        }
    }

    impl Fake {
        pub fn extension_info(&self) -> HashMap<String, OwnedValue> {
            self.extension_info
                .iter()
                .map(|(k, v)| (k.clone(), v.try_clone().unwrap()))
                .collect::<_>()
        }
    }
}

impl ShellExtensions {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self(inner::Inner::Real(ctx))
    }

    /// Creates a new fake shell extension client, returning GNOME Shell v45 and the extension as
    /// not installed.
    pub fn new_fake(ctx: Arc<Context>) -> Self {
        Self(inner::Inner::Fake(Arc::new(Mutex::new(inner::Fake {
            ctx,
            ..Default::default()
        }))))
    }

    /// Returns the version of the system's GNOME Shell.
    pub async fn gnome_shell_version(&self) -> Result<GnomeShellVersion, ExtensionsError> {
        use inner::Inner;
        match &self.0 {
            Inner::Real(_) => new_proxy().await?.shell_version().await?.parse(),
            Inner::Fake(fake) => Ok(fake.lock().await.version.clone()),
        }
    }

    pub async fn get_extension_info(&self) -> Result<HashMap<String, OwnedValue>, ExtensionsError> {
        use inner::Inner;
        match &self.0 {
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
        match &self.0 {
            Inner::Real(_) => {
                let r = new_proxy()
                    .await?
                    .uninstall_extension(&self.extension_uuid().await?)
                    .await?;

                Ok(r)
            },
            Inner::Fake(fake) => match fake.lock().await.installed_version {
                Some(_) => Ok(true),
                None => Ok(false),
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
        match &self.0 {
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
            Inner::Fake(fake) => {
                if fake.lock().await.ctx.fs().exists(&path) {
                    let uuid = self.extension_uuid().await.unwrap();
                    let mut fake = fake.lock().await;
                    let extension_dir_path = fake
                        .ctx
                        .env()
                        .home()
                        .unwrap()
                        .join(".local/share/gnome-shell/extensions")
                        .join(uuid);
                    fake.ctx.fs().create_dir_all(&extension_dir_path).await.ok();
                    let version: u32 = fake.ctx.fs().read_to_string(&path).await.unwrap().parse().unwrap();
                    fake.installed_version = Some(version);
                    fake.ctx
                        .fs()
                        .write(
                            extension_dir_path.join("metadata.json"),
                            json!({ "version": version }).to_string(),
                        )
                        .await
                        .unwrap();
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
        use inner::Inner;
        match &self.0 {
            Inner::Real(ctx) => Ok(ctx.sysinfo().is_process_running(GNOME_SHELL_PROCESS_NAME)),
            Inner::Fake(fake) => Ok(fake
                .lock()
                .await
                .ctx
                .sysinfo()
                .is_process_running(GNOME_SHELL_PROCESS_NAME)),
        }
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
        match &self.0 {
            Inner::Real(_) => Ok(new_proxy()
                .await?
                .enable_extension(&self.extension_uuid().await?)
                .await?),
            Inner::Fake(fake) => Ok(fake.lock().await.extension_enabled),
        }
    }

    #[allow(dead_code)]
    async fn set_fake_extension_info(&self, extension_info: HashMap<String, OwnedValue>) {
        if let inner::Inner::Fake(fake) = &self.0 {
            fake.lock().await.extension_info = extension_info;
        }
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

    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_fake_impl_behavior_on_install() {
        let ctx = Context::builder()
            .with_test_home()
            .await
            .unwrap()
            .with_running_processes(&[GNOME_SHELL_PROCESS_NAME])
            .build_fake();
        let shell_extensions = ShellExtensions::new_fake(Arc::clone(&ctx));

        // Default status is not installed
        let status = get_extension_status(&ctx, &shell_extensions, 1).await.unwrap();
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
        let status = get_extension_status(&ctx, &shell_extensions, extension_version)
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
            let shell_extensions = ShellExtensions::new_fake(Arc::clone(&ctx));

            // When
            let status = get_extension_status(&ctx, &shell_extensions, 1).await.unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::GnomeShellNotRunning);
        }

        #[tokio::test]
        async fn test_extension_status_when_empty_info() {
            let ctx = make_ctx().await;
            let shell_extensions = ShellExtensions::new_fake(Arc::clone(&ctx));

            // When
            let status = get_extension_status(&ctx, &shell_extensions, 1).await.unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::NotInstalled);
        }

        #[tokio::test]
        async fn test_extension_installed_but_not_loaded() {
            tracing_subscriber::fmt::try_init().ok();

            let ctx = make_ctx().await;
            let shell_extensions = ShellExtensions::new_fake(Arc::clone(&ctx));
            let extension_dir_path = ctx
                .env()
                .home()
                .unwrap()
                .join(".local/share/gnome-shell/extensions")
                .join(shell_extensions.extension_uuid().await.unwrap());
            ctx.fs().create_dir_all(&extension_dir_path).await.unwrap();
            let expected_version: u32 = 1;
            ctx.fs()
                .write(
                    extension_dir_path.join("metadata.json"),
                    json!({ "version": expected_version }).to_string(),
                )
                .await
                .unwrap();

            // When
            let status = get_extension_status(&ctx, &shell_extensions, expected_version)
                .await
                .unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::RequiresReboot);
        }

        #[tokio::test]
        async fn test_extension_installed_but_not_loaded_with_different_version() {
            tracing_subscriber::fmt::try_init().ok();

            let ctx = make_ctx().await;
            let shell_extensions = ShellExtensions::new_fake(Arc::clone(&ctx));
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
            let status = get_extension_status(&ctx, &shell_extensions, expected_version)
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
            let shell_extensions = ShellExtensions::new_fake(Arc::clone(&ctx));
            let expected_version = 2;
            let installed_version = 1;
            shell_extensions
                .set_fake_extension_info(
                    [("version".to_string(), OwnedValue::from(installed_version as f64))]
                        .into_iter()
                        .collect(),
                )
                .await;

            // When
            let status = get_extension_status(&ctx, &shell_extensions, expected_version)
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
            let shell_extensions = ShellExtensions::new_fake(Arc::clone(&ctx));
            let expected_version = 2;
            shell_extensions
                .set_fake_extension_info(
                    [
                        ("version".to_string(), OwnedValue::from(expected_version as f64)),
                        ("state".to_string(), OwnedValue::from(2_f64)),
                    ]
                    .into_iter()
                    .collect(),
                )
                .await;

            // When
            let status = get_extension_status(&ctx, &shell_extensions, expected_version)
                .await
                .unwrap();

            // Then
            assert_eq!(status, ExtensionInstallationStatus::NotEnabled);
        }

        #[tokio::test]
        async fn test_extension_installed_and_enabled() {
            tracing_subscriber::fmt::try_init().ok();

            let ctx = make_ctx().await;
            let shell_extensions = ShellExtensions::new_fake(Arc::clone(&ctx));
            let expected_version = 2;
            shell_extensions
                .set_fake_extension_info(
                    [
                        ("version".to_string(), OwnedValue::from(expected_version as f64)),
                        ("state".to_string(), OwnedValue::from(1_f64)),
                    ]
                    .into_iter()
                    .collect(),
                )
                .await;

            // When
            let status = get_extension_status(&ctx, &shell_extensions, expected_version)
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
            println!(
                "{:?}",
                ShellExtensions::new(Context::new())
                    .gnome_shell_version()
                    .await
                    .unwrap()
            );
        }

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_get_extension_info() {
            let uuid = ShellExtensions::new(Context::new()).extension_uuid().await.unwrap();
            let info = new_proxy().await.unwrap().get_extension_info(&uuid).await.unwrap();
            // let info = new_proxy().await.unwrap().get_extension_info("jdsifosdjif").await.unwrap();
            println!("{:?}", info);
        }

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_get_installed_extension_status() {
            let ctx = Context::new();
            let shell_extensions = ShellExtensions::new(Arc::clone(&ctx));
            let status = get_extension_status(&ctx, &shell_extensions, 1).await.unwrap();
            println!("{:?}", status);
        }

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_uninstall_extension() {
            let res = ShellExtensions::new(Context::new())
                .uninstall_extension()
                .await
                .unwrap();
            println!("{:?}", res);
        }

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_install_bundled_extension() {
            let path = PathBuf::from("");
            let res = ShellExtensions::new(Context::new())
                .install_bundled_extension(path)
                .await;
            println!("{:?}", res);
        }

        #[tokio::test]
        #[ignore = "not in ci"]
        async fn test_enable_extension() {
            let res = ShellExtensions::new(Context::new()).enable_extension().await;
            println!("{:?}", res);
        }
    }
}
