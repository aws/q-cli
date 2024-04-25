use std::path::Path;

use fig_integrations::shell::ShellExt;
use fig_integrations::ssh::SshIntegration;
use fig_integrations::Integration;
use fig_util::{
    directories,
    Shell,
    CLI_BINARY_NAME,
    OLD_CLI_BINARY_NAME,
};

use crate::Error;

bitflags::bitflags! {
    /// The different components that can be installed.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InstallComponents: u64 {
        const SHELL_INTEGRATIONS = 0b00000001;
        const BINARY             = 0b00000010;
        const SSH                = 0b00000100;
        const DESKTOP_APP        = 0b00001000;
        const INPUT_METHOD       = 0b00010000;
    }
}

pub async fn uninstall(components: InstallComponents) -> Result<(), Error> {
    let ssh_result = if components.contains(InstallComponents::SSH) {
        SshIntegration::new()?.uninstall().await
    } else {
        Ok(())
    };

    let shell_integration_result = {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            for integration in shell.get_shell_integrations()? {
                integration.uninstall().await?;
            }
        }
        Ok(())
    };

    if components.contains(InstallComponents::BINARY) {
        let local_bin_path = directories::home_local_bin()?;
        let binary_paths = [
            local_bin_path.join(CLI_BINARY_NAME),
            local_bin_path.join(OLD_CLI_BINARY_NAME),
            Path::new("/usr/local/bin").join(CLI_BINARY_NAME),
            Path::new("/usr/local/bin").join(OLD_CLI_BINARY_NAME),
        ];

        for path in binary_paths {
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }
    }

    let daemon_result = Ok(());

    #[cfg(target_os = "macos")]
    if components.contains(InstallComponents::INPUT_METHOD) {
        use fig_integrations::input_method::{
            InputMethod,
            InputMethodError,
        };
        use fig_integrations::Error;

        match InputMethod::default().uninstall().await {
            Ok(_) | Err(Error::InputMethod(InputMethodError::CouldNotListInputSources)) => {},
            Err(err) => return Err(err.into()),
        }
    }

    #[cfg(target_os = "macos")]
    if components.contains(InstallComponents::DESKTOP_APP) {
        super::os::uninstall_desktop().await?;
        // Must be last -- this will kill the running desktop process if this is
        // called from the desktop app.
        let quit_res = tokio::process::Command::new("killall")
            .args([fig_util::consts::APP_PROCESS_NAME])
            .output()
            .await;
        if let Err(err) = quit_res {
            tracing::warn!("Failed to quit running Fig app: {err}");
        }
    }

    daemon_result
        .and(shell_integration_result)
        .and(ssh_result.map_err(|e| e.into()))
}

pub async fn install(components: InstallComponents) -> Result<(), Error> {
    if components.contains(InstallComponents::SHELL_INTEGRATIONS) {
        let mut errs: Vec<Error> = vec![];
        for shell in Shell::all() {
            match shell.get_shell_integrations() {
                Ok(integrations) => {
                    for integration in integrations {
                        if let Err(e) = integration.install().await {
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
        SshIntegration::new()?.install().await?;
    }

    #[cfg(target_os = "macos")]
    if components.contains(InstallComponents::INPUT_METHOD) {
        use fig_integrations::input_method::InputMethod;
        InputMethod::default().install().await?;
    }

    Ok(())
}
