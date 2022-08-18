use std::path::Path;
use std::process::Command;

use sysinfo::{
    ProcessRefreshKind,
    RefreshKind,
    System,
    SystemExt,
};

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
        let system = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
        if system.processes_by_name("ibus-daemon").next().is_none() {
            return Err(Error::Custom(
                "IBus daemon is not running, either run `ibus-setup` or log out and log back in.".into(),
            ));
        }

        match Command::new("ibus").arg("engine").arg("fig").output() {
            Ok(std::process::Output { status, stderr, .. }) if !status.success() => {
                if self.is_installed().is_ok() {
                    Ok(())
                } else {
                    Err(Error::Custom(
                        format!(
                            "Failed to set IBus engine, you may need to log out and log back in.\n\nDetails: Failed to run 'ibus engine fig':\n{}", 
                            String::from_utf8_lossy(&stderr)
                        ).into()
                    ))
                }
            },
            Err(err) => Err(Error::Custom(
                format!(
                    "Failed to run 'ibus', it may not be installed.\n\nDetails: Failed to run 'ibus engine fig': {err}"
                )
                .into(),
            )),
            Ok(_) => Ok(()),
        }
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
            Err(Error::Custom(
                String::from_utf8_lossy(&ibus_engine_output.stderr).to_string().into(),
            ))
        }
    }
}
