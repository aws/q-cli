use once_cell::sync::OnceCell;
use thiserror::Error;
use tokio::sync::Mutex;
use zbus::Connection;

use self::ibus::AddressError;

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
