use std::path::Path;
use std::process::Command;

use crate::error::{
    Error,
    Result,
};
use crate::Integration;

fn ibus_engine_path() -> &'static Path {
    Path::new("/usr/share/ibus/component/engine.xml")
}

pub struct IbusIntegration {}

impl Integration for IbusIntegration {
    fn describe(&self) -> String {
        "IBus Engine".to_owned()
    }

    fn install(&self, _: Option<&Path>) -> Result<()> {
        let output = Command::new("ibus").args(&["engine", "fig"]).output()?;
        output
            .status
            .success()
            .then(|| ())
            .ok_or_else(|| Error::Custom("Failed set IBus engine to Fig, IBus may not be running".into()))
    }

    fn uninstall(&self) -> Result<()> {
        Err(Error::Custom("IBus integration cannot be uninstalled".into()))
    }

    fn is_installed(&self) -> Result<()> {
        if !ibus_engine_path().exists() {
            return Err(Error::FileDoesNotExist(ibus_engine_path().into()));
        }

        let ibus_engine_output = Command::new("ibus")
            .arg("engine")
            .output()
            .map_err(|err| Error::NotInstalled(err.to_string().into()))?;

        let stdout = String::from_utf8_lossy(&ibus_engine_output.stdout);

        if ibus_engine_output.status.success() && "fig" == stdout.trim() {
            Ok(())
        } else {
            Err(Error::NotInstalled("".into()))
        }
    }
}
