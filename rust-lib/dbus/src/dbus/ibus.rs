//! # DBus interface proxy for: `org.freedesktop.IBus`

use std::process::Output;

use thiserror::Error;
use tokio::process::Command;
use zbus::zvariant::{
    OwnedObjectPath,
    OwnedValue,
};
use zbus::{
    dbus_proxy,
    Connection,
    ConnectionBuilder,
};

use super::CrateError;

#[derive(Debug, Error)]
pub enum AddressError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Null address")]
    Null,
    #[error("Command failed: {0:?}")]
    FailedOutput(Output),
}

pub async fn ibus_address() -> Result<String, AddressError> {
    // TODO(grant): get address directly
    match Command::new("ibus").arg("address").output().await {
        Ok(output) if !output.status.success() => Err(AddressError::FailedOutput(output)),
        Ok(Output { stdout, .. }) => {
            let address_output = String::from_utf8(stdout)?.trim().to_owned();
            match address_output.as_str() {
                "(null)" => Err(AddressError::Null),
                _ => Ok(address_output),
            }
        },
        Err(err) => Err(AddressError::Io(err)),
    }
}

pub async fn ibus_connect() -> Result<Connection, CrateError> {
    let address = ibus_address().await?;
    Ok(ConnectionBuilder::address(&*address)?.build().await?)
}

pub async fn ibus_proxy(connection: &Connection) -> Result<IBusProxy, CrateError> {
    Ok(IBusProxy::new(connection).await?)
}

#[dbus_proxy(interface = "org.freedesktop.IBus")]
pub trait IBus {
    /// CreateInputContext method
    fn create_input_context(&self, client_name: &str) -> zbus::Result<OwnedObjectPath>;

    /// Exit method
    fn exit(&self, restart: bool) -> zbus::Result<()>;

    /// GetEnginesByNames method
    fn get_engines_by_names(&self, names: &[&str]) -> zbus::Result<Vec<OwnedValue>>;

    /// GetUseGlobalEngine method
    fn get_use_global_engine(&self) -> zbus::Result<bool>;

    /// Ping method
    fn ping(&self, data: &zbus::zvariant::Value<'_>) -> zbus::Result<OwnedValue>;

    /// RegisterComponent method
    fn register_component(&self, component: &zbus::zvariant::Value<'_>) -> zbus::Result<()>;

    /// SetGlobalEngine method
    fn set_global_engine(&self, engine_name: &str) -> zbus::Result<()>;

    /// RegistryChanged signal
    #[dbus_proxy(signal)]
    fn registry_changed(&self) -> zbus::Result<()>;

    /// ActiveEngines property
    #[dbus_proxy(property)]
    fn active_engines(&self) -> zbus::Result<Vec<OwnedValue>>;

    /// Address property
    #[dbus_proxy(property)]
    fn address(&self) -> zbus::Result<String>;

    /// CurrentInputContext property
    #[dbus_proxy(property)]
    fn current_input_context(&self) -> zbus::Result<OwnedObjectPath>;

    /// EmbedPreeditText property
    #[dbus_proxy(property)]
    fn embed_preedit_text(&self) -> zbus::Result<bool>;

    #[dbus_proxy(property)]
    fn set_embed_preedit_text(&self, value: bool) -> zbus::Result<()>;

    /// Engines property
    #[dbus_proxy(property)]
    fn engines(&self) -> zbus::Result<Vec<OwnedValue>>;

    /// GlobalEngine property
    #[dbus_proxy(property)]
    fn global_engine(&self) -> zbus::Result<OwnedValue>;

    /// PreloadEngines property
    #[dbus_proxy(property)]
    fn set_preload_engines(&self, value: &[&str]) -> zbus::Result<()>;
}
