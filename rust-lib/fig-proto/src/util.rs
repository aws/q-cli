use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetShellError {
    #[error("failed to get shell")]
    Io(#[from] std::io::Error),
    #[error("failed to parse shell")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("not yet implemented for windows")]
    WindowsError,
}

pub fn get_shell() -> Result<String, GetShellError> {
    #[cfg(target_os = "windows")]
    return Err(GetShellError::WindowsError);

    #[cfg(not(target_os = "windows"))]
    {
        let ppid = nix::unistd::getppid();

        let result = Command::new("ps")
            .arg("-p")
            .arg(format!("{}", ppid))
            .arg("-o")
            .arg("comm=")
            .output()?;

        Ok(std::str::from_utf8(&result.stdout)?.trim().into())
    }
}
