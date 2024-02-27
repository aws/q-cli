use std::collections::HashMap;

use zbus::proxy;
use zbus::zvariant::OwnedValue;

use super::session_bus;
use crate::CrateError;

const EXTENSION_NAME: &str = "fig-gnome-integration@fig.io";

#[proxy(
    interface = "org.gnome.Shell.Extensions",
    default_path = "/org/gnome/Shell/Extensions"
)]
trait ShellExtensions {
    /// ListExtensions method
    fn list_extensions(&self) -> zbus::Result<HashMap<String, HashMap<String, OwnedValue>>>;

    /// InstallRemoteExtension method
    fn install_remote_extension(&self, uuid: &str) -> zbus::Result<String>;
}

pub async fn has_extension() -> Result<bool, CrateError> {
    let proxy = ShellExtensionsProxy::new(session_bus().await?, "/org/gnome/Shell/Extensions").await?;
    let extensions = proxy.list_extensions().await?;

    Ok(extensions.contains_key(EXTENSION_NAME))
}

pub async fn install_extension() -> Result<(), CrateError> {
    let proxy = ShellExtensionsProxy::new(session_bus().await?, "/org/gnome/Shell/Extensions").await?;
    proxy.install_remote_extension("fig-gnome-integration@fig.io").await?;

    Ok(())
}
