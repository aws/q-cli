use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::error::{
    Error,
    Result,
};
use crate::Integration;

pub fn ibus_engine_path() -> &'static Path {
    Path::new("/usr/share/ibus/component/engine.xml")
}

pub struct IbusIntegration {}

impl Integration for IbusIntegration {
    fn describe(&self) -> String {
        "IBus Engine".to_owned()
    }

    fn install(&self, _: Option<&Path>) -> Result<()> {
        match Command::new("ibus").arg("write-cache").output() {
            Ok(std::process::Output { status, stderr, .. }) if !status.success() => {
                return Err(Error::Custom(
                    format!(
                        "Failed to run 'ibus write-cache':\n{}",
                        String::from_utf8_lossy(&stderr)
                    )
                    .into(),
                ));
            },
            Err(err) => return Err(Error::Custom(format!("Failed to run 'ibus write-cache': {err}").into())),
            Ok(_) => {},
        };

        std::thread::sleep(Duration::from_millis(250));

        match Command::new("ibus").arg("engine").arg("fig").output() {
            Ok(std::process::Output { status, stderr, .. }) if !status.success() => {
                return Err(Error::Custom(
                    format!("Failed to run 'ibus engine fig':\n{}", String::from_utf8_lossy(&stderr)).into(),
                ));
            },
            Err(err) => return Err(Error::Custom(format!("Failed to run 'ibus engine fig': {err}").into())),
            Ok(_) => {},
        };

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        Err(Error::Custom("IBus integration cannot be uninstalled".into()))
    }

    fn is_installed(&self) -> Result<()> {
        let ibus_engine_output = Command::new("ibus")
            .arg("engine")
            .output()
            .map_err(|err| Error::NotInstalled(err.to_string().into()))?;

        let stdout = String::from_utf8_lossy(&ibus_engine_output.stdout);

        if ibus_engine_output.status.success() && "fig" == stdout.trim() {
            Ok(())
        } else {
            Err(Error::Custom(stdout.to_string().into()))
        }
    }
}
