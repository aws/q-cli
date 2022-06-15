use std::path::Path;
use std::process::Command;

use anyhow::{
    anyhow,
    Result,
};

use crate::error::InstallationError;
use crate::Integration;

fn ibus_engine_path() -> &'static Path {
    Path::new("/usr/share/ibus/component/engine.xml")
}

pub struct IbusIntegration {}

impl Integration for IbusIntegration {
    fn install(&self, _: Option<&Path>) -> Result<()> {
        let output = Command::new("ibus").args(&["engine", "fig"]).output()?;
        output
            .status
            .success()
            .then(|| ())
            .ok_or_else(|| anyhow!("Failed set IBus engine to Fig, IBus may not be running."))
    }

    fn uninstall(&self) -> Result<()> {
        Err(anyhow!("IBus integration cannot be uninstalled"))
    }

    fn is_installed(&self) -> Result<(), InstallationError> {
        if !ibus_engine_path().exists() {
            return Err(InstallationError::FileDoesNotExist(ibus_engine_path().into()));
        }

        let ibus_engine_output = Command::new("ibus")
            .arg("engine")
            .output()
            .map_err(|err| InstallationError::NotInstalled(err.to_string().into()))?;

        let stdout = String::from_utf8_lossy(&ibus_engine_output.stdout);

        if ibus_engine_output.status.success() && "fig" == stdout.trim() {
            Ok(())
        } else {
            Err(InstallationError::NotInstalled("".into()))
        }
    }
}
