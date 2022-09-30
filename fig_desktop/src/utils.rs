use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect<U, V> {
    pub x: U,
    pub y: U,
    pub width: V,
    pub height: V,
}

#[cfg(target_os = "windows")]
pub async fn update_check() {
    use std::os::windows::process::CommandExt;

    use tracing::{
        error,
        info,
    };

    let installer = fig_util::directories::fig_data_dir().unwrap().join("fig_installer.exe");

    if installer.exists() {
        if let Err(e) = std::fs::remove_file(&installer) {
            error!("Failed to remove previous installer version: {e}");
            return;
        }
    }

    info!("Checking for updates...");

    match fig_install::check_for_updates(Some(env!("CARGO_PKG_VERSION").to_string())).await {
        Ok(Some(package)) => {
            info!("Updating Fig...");

            let detached = 0x8;
            if let Err(e) = std::process::Command::new("curl")
                .creation_flags(detached)
                .args(["-L", "-s", "-o", &installer.to_string_lossy(), &package.download])
                .status()
            {
                error!("Failed to download the newest version of Fig: {e}");
                return;
            }

            match std::process::Command::new(installer.as_os_str())
                .args(["/upgrade", "/quiet", "/norestart"])
                .spawn()
            {
                Ok(_) => std::process::exit(0),
                Err(e) => error!("Failed to update Fig: {e}"),
            }
        },
        Ok(None) => {
            info!("no updates available");
        },
        Err(err) => error!("failed checking for updates: {err:?}"),
    }
}
