use std::os::windows::process::CommandExt;

use crate::index::UpdatePackage;
use crate::Error;

pub async fn update(package: UpdatePackage, _deprecated: bool) -> Result<(), Error> {
    let installer_path = fig_util::directories::fig_data_dir().unwrap().join("fig_installer.exe");

    if installer_path.exists() {
        std::fs::remove_file(&installer_path)?;
    }

    let detached = 0x8;
    std::process::Command::new("curl")
        .creation_flags(detached)
        .args(["-L", "-s", "-o"])
        .arg(&installer_path)
        .arg(&package.download)
        .status()?;

    std::process::Command::new(installer_path)
        .args(["/upgrade", "/quiet", "/norestart"])
        .spawn()?;

    Ok(())
}
