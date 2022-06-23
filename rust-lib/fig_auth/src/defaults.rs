use std::ffi::OsStr;
use std::process::Command;

use anyhow::Result;

pub fn get_default(key: impl AsRef<OsStr>) -> Result<String> {
    let output = Command::new("defaults")
        .arg("read")
        .arg("com.mschrage.fig")
        .arg(key)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("defaults read failed"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().into())
}

pub fn set_default(key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Result<()> {
    let output = Command::new("defaults")
        .arg("write")
        .arg("com.mschrage.fig")
        .arg(key)
        .arg(value)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("defaults write failed"));
    }

    Ok(())
}

pub fn remove_default(key: impl AsRef<OsStr>) -> Result<()> {
    let output = Command::new("defaults")
        .arg("delete")
        .arg("com.mschrage.fig")
        .arg(key)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("defaults write failed"));
    }

    Ok(())
}
