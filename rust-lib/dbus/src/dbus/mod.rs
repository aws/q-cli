use once_cell::sync::OnceCell;
use thiserror::Error;
use tokio::sync::Mutex;
use zbus::{
    Connection,
    ConnectionBuilder,
};

use self::ibus::{
    ibus_address,
    AddressError,
};

pub mod gnome_shell;
pub mod ibus;

#[derive(Debug, Error)]
pub enum CrateError {
    #[error(transparent)]
    Address(#[from] AddressError),
    #[error(transparent)]
    ZBus(#[from] zbus::Error),
    #[error(transparent)]
    ZVariant(#[from] zbus::zvariant::Error),
    #[error("Invalid GNOME shell version {0}")]
    InvalidVersion(String),
    #[error(transparent)]
    Fdo(#[from] zbus::fdo::Error),
}

static SESSION_BUS: OnceCell<Connection> = OnceCell::new();
static SESSION_BUS_INIT: Mutex<()> = Mutex::const_new(());

async fn session_bus() -> Result<&'static Connection, CrateError> {
    if let Some(connection) = SESSION_BUS.get() {
        return Ok(connection);
    }

    let _guard = SESSION_BUS_INIT.lock().await;

    if let Some(connection) = SESSION_BUS.get() {
        return Ok(connection);
    }

    let connection = Connection::session().await?;

    let _ = SESSION_BUS.set(connection);

    Ok(SESSION_BUS.get().unwrap())
}

static IBUS_BUS: OnceCell<Connection> = OnceCell::new();
static IBUS_BUS_INIT: Mutex<()> = Mutex::const_new(());

pub async fn ibus_bus_new() -> Result<Connection, CrateError> {
    Ok(ConnectionBuilder::address(&*ibus_address().await?)?.build().await?)
}

pub async fn ibus_bus() -> Result<&'static Connection, CrateError> {
    if let Some(connection) = IBUS_BUS.get() {
        return Ok(connection);
    }

    let _guard = IBUS_BUS_INIT.lock().await;

    if let Some(connection) = IBUS_BUS.get() {
        return Ok(connection);
    }

    let connection = ibus_bus_new().await?;

    let _ = IBUS_BUS.set(connection);

    Ok(IBUS_BUS.get().unwrap())
}
