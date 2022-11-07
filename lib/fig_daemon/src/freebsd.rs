use camino::Utf8Path;

use crate::{
    Error,
    Result,
};

#[derive(Debug, Default)]
pub struct Daemon;

impl Daemon {
    pub async fn install(&self, _executable: &Utf8Path) -> Result<()> {
        Err(Error::Unimplemented)
    }

    pub async fn uninstall(&self) -> Result<()> {
        Err(Error::Unimplemented)
    }

    pub async fn start(&self) -> Result<()> {
        Err(Error::Unimplemented)
    }

    pub async fn stop(&self) -> Result<()> {
        Err(Error::Unimplemented)
    }

    pub async fn restart(&self) -> Result<()> {
        Err(Error::Unimplemented)
    }

    pub async fn status(&self) -> Result<Option<i32>> {
        Err(Error::Unimplemented)
    }
}
