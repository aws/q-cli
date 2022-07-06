use std::ffi::OsStr;
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DefaultsError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("defaults read failed")]
    ReadFail,
    #[error("defaults write failed")]
    WriteFail,
}

pub fn get_default(key: impl AsRef<OsStr>) -> Result<String, DefaultsError> {
    let output = Command::new("defaults")
        .arg("read")
        .arg("com.mschrage.fig")
        .arg(key)
        .output()?;

    if !output.status.success() {
        Err(DefaultsError::ReadFail)
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).trim().into())
    }
}

pub fn set_default(key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Result<(), DefaultsError> {
    let output = Command::new("defaults")
        .arg("write")
        .arg("com.mschrage.fig")
        .arg(key)
        .arg(value)
        .output()?;

    if !output.status.success() {
        Err(DefaultsError::WriteFail)
    } else {
        Ok(())
    }
}

pub fn remove_default(key: impl AsRef<OsStr>) -> Result<(), DefaultsError> {
    let output = Command::new("defaults")
        .arg("delete")
        .arg("com.mschrage.fig")
        .arg(key)
        .output()?;

    if !output.status.success() {
        Err(DefaultsError::WriteFail)
    } else {
        Ok(())
    }
}
