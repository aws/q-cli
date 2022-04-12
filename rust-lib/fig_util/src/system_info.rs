use sha2::{Sha256, Digest};

use crate::Error;

#[cfg(target_os = "linux")]
const MACHINE_ID_FILES:[&str; 2] = [
    "/var/lib/dbus/machine-id",
    "/etc/machine-id"
];

pub fn get_system_id() -> Result<String, Error> {
    #[allow(unused_assignments)]
    let mut hwid = None;
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("ioreg")
            .args(&["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()?;

        let output = String::from_utf8_lossy(&output.stdout);

        let machine_id: String = output
            .lines()
            .find(|line| line.contains("IOPlatformUUID")).ok_or(Error::HwidNotFound)?
            .split('=')
            .nth(1).ok_or(Error::HwidNotFound)?
            .trim()
            .trim_start_matches('"')
            .trim_end_matches('"')
            .into();

        hwid = Some(machine_id);
    };
    #[cfg(target_os = "linux")]
    for path in MACHINE_ID_FILES.iter() {
        use std::io::Read;

        if std::path::Path::new(path).exists(){
            let content = {
                let mut file = std::fs::File::open(path)?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                content
            };
            hwid = Some(content);
            break;
        }
    };
    #[cfg(windows)]
    {
        use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

        let rkey = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\Microsoft\\Cryptography")?;
        let id: String = rkey.get_value("MachineGuid")?;

        hwid = Some(id);
    }

    let mut hasher = Sha256::new();
    hasher.update(hwid.ok_or(Error::HwidNotFound)?);
    Ok(format!("{:x}", hasher.finalize()))
}
