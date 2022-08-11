use cfg_if::cfg_if;
use sha2::{
    Digest,
    Sha256,
};

use crate::Error;

pub fn get_system_id() -> Result<String, Error> {
    #[allow(unused_assignments)]
    let mut hwid = None;

    cfg_if!(
        if #[cfg(target_os = "macos")] {
            let output = std::process::Command::new("ioreg")
                .args(&["-rd1", "-c", "IOPlatformExpertDevice"])
                .output()?;

            let output = String::from_utf8_lossy(&output.stdout);

            let machine_id: String = output
                .lines()
                .find(|line| line.contains("IOPlatformUUID"))
                .ok_or(Error::HwidNotFound)?
                .split('=')
                .nth(1)
                .ok_or(Error::HwidNotFound)?
                .trim()
                .trim_start_matches('"')
                .trim_end_matches('"')
                .into();

            hwid = Some(machine_id);
        } else if #[cfg(target_os = "linux")] {
            for path in ["/var/lib/dbus/machine-id", "/etc/machine-id"] {
                use std::io::Read;

                if std::path::Path::new(path).exists() {
                    let content = {
                        let mut file = std::fs::File::open(path)?;
                        let mut content = String::new();
                        file.read_to_string(&mut content)?;
                        content
                    };
                    hwid = Some(content);
                    break;
                }
            }
        } else if #[cfg(windows)] {
            use winreg::enums::HKEY_LOCAL_MACHINE;
            use winreg::RegKey;

            let rkey = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\Microsoft\\Cryptography")?;
            let id: String = rkey.get_value("MachineGuid")?;

            hwid = Some(id);
        }
    );

    let mut hasher = Sha256::new();
    hasher.update(hwid.ok_or(Error::HwidNotFound)?);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn get_platform() -> Result<&'static str, Error> {
    if let Some(over_ride) = option_env!("FIG_OVERRIDE_PLATFORM") {
        return Ok(over_ride);
    }

    cfg_if! {
        if #[cfg(windows)] {
            return Ok("windows");
        } else if #[cfg(target_os = "linux")] {
            return Ok("linux");
        } else if #[cfg(target_os = "macos")] {
            return Ok("macos");
        } else {
            return Err(Error::UnsupportedPlatform);
        }
    }
}

pub fn get_arch() -> Result<&'static str, Error> {
    if let Some(over_ride) = option_env!("FIG_OVERRIDE_ARCH") {
        return Ok(over_ride);
    }

    cfg_if! {
        if #[cfg(target_arch = "x86_64")] {
            return Ok("x86_64");
        } else if #[cfg(target_arch = "aarch64")] {
            return Ok("aarch64");
        } else {
            return Err(Error::UnsupportedArch);
        }
    }
}
