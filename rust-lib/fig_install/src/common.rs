use std::convert::TryInto;
use std::path::Path;

use fig_daemon::Daemon;
use fig_integrations::shell::ShellExt;
use fig_integrations::ssh::SshIntegration;
use fig_integrations::Integration;
use fig_util::{
    directories,
    Shell,
};

use crate::Error;

bitflags::bitflags! {
    /// The different components that can be installed.
    pub struct InstallComponents: usize {
        const DAEMON             = 0b00000001;
        const SHELL_INTEGRATIONS = 0b00000010;
        const BINARY             = 0b00000100;
        const SSH                = 0b00001000;
        const DESKTOP_APP        = 0b00010000;
        const INPUT_METHOD       = 0b00100000;
    }
}

pub async fn uninstall(components: InstallComponents) -> Result<(), Error> {
    let ssh_result = if components.contains(InstallComponents::SSH) {
        SshIntegration::default()?.uninstall()
    } else {
        Ok(())
    };

    let shell_integration_result = {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            for integration in shell.get_shell_integrations()? {
                integration.uninstall()?
            }
        }
        Ok(())
    };

    if components.contains(InstallComponents::BINARY) {
        let local_path = directories::home_dir()?.join(".local").join("bin").join("fig");
        let binary_paths = [Path::new("/usr/local/bin/fig"), local_path.as_path()];

        for path in binary_paths {
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }
    }

    let daemon_result = if components.contains(InstallComponents::DAEMON) {
        Daemon::default().uninstall().await?;
        Ok(())
    } else {
        Ok(())
    };

    #[cfg(target_os = "macos")]
    if components.contains(InstallComponents::DESKTOP_APP) {
        super::os::uninstall_desktop().await?;
    }

    #[cfg(target_os = "macos")]
    if components.contains(InstallComponents::INPUT_METHOD) {
        let fig_input_method_app = directories::home_dir()?
            .join("Library")
            .join("Input Methods")
            .join("FigInputMethod.app");

        if fig_input_method_app.exists() {
            std::fs::remove_dir_all(fig_input_method_app)?;
        }
    }

    daemon_result
        .and(shell_integration_result)
        .and(ssh_result.map_err(|e| e.into()))
}

pub async fn install(components: InstallComponents) -> Result<(), Error> {
    if components.contains(InstallComponents::SHELL_INTEGRATIONS) {
        let backup_dir = directories::utc_backup_dir()?;

        let mut errs: Vec<Error> = vec![];
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            match shell.get_shell_integrations() {
                Ok(integrations) => {
                    for integration in integrations {
                        if let Err(e) = integration.install(Some(&backup_dir)) {
                            errs.push(e.into());
                        }
                    }
                },
                Err(e) => {
                    errs.push(e.into());
                },
            }
        }

        if let Some(err) = errs.pop() {
            return Err(err);
        }
    }

    if components.contains(InstallComponents::SSH) {
        SshIntegration::default()?.install(None)?;
    }

    if components.contains(InstallComponents::DAEMON) {
        let path: camino::Utf8PathBuf = std::env::current_exe()?.try_into()?;
        Daemon::default().install(&path).await?;
    }

    Ok(())
}
