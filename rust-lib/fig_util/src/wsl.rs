use once_cell::sync::Lazy;

#[cfg(target_os = "linux")]
static IS_WLS: Lazy<bool> = Lazy::new(|| {
    if let Ok(b) = std::fs::read("/proc/sys/kernel/osrelease") {
        if let Ok(s) = std::str::from_utf8(&b) {
            let a = s.to_ascii_lowercase();
            return a.contains("microsoft") || a.contains("wsl");
        }
    }
    false
});

/// Test if the program is running under WSL
#[cfg(target_os = "linux")]
pub fn is_wsl() -> bool {
    *IS_WLS
}

/// Test if the program is running under WSL
#[cfg(not(target_os = "linux"))]
pub fn is_wsl() -> bool {
    false
}
