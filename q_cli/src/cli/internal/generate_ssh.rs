use std::io::{
    stdout,
    Write,
};
use std::os::unix::net::UnixStream;

use clap::Args;
use crossterm::style::Stylize;
use eyre::Result;
use fig_util::{
    directories,
    CLI_BINARY_NAME,
    PRODUCT_NAME,
};
use indoc::formatdoc;
use uuid::Uuid;

const IGNORED_USERNAMES: &[&str] = &["git", "aur"];

#[derive(Debug, PartialEq, Eq, Args)]
pub struct GenerateSshArgs {
    /// The remote host
    #[arg(long)]
    remote_host: Option<String>,
    /// The remote port
    #[arg(long)]
    remote_port: Option<String>,
    /// The remote username
    #[arg(long)]
    remote_username: Option<String>,
}

impl GenerateSshArgs {
    pub fn execute(self) -> Result<()> {
        let GenerateSshArgs { remote_username, .. } = &self;

        let mut should_generate_config = true;

        if let Some(remote_username) = remote_username {
            for username in IGNORED_USERNAMES {
                if remote_username == username {
                    should_generate_config = false;
                }
            }
        }

        // check if remote socket is able to be connected to
        let remote_socket = directories::remote_socket_path_utf8()?;
        let stream = UnixStream::connect(&remote_socket);
        if stream.is_err() {
            should_generate_config = false;
        }
        drop(stream);

        let config_path = directories::fig_data_dir()?.join("ssh_inner");

        if should_generate_config {
            let uuid = uuid::Uuid::new_v4();
            let exe_path = std::env::current_exe()?;
            let exe_path = exe_path.to_string_lossy();

            let config = self.ssh_config(&uuid, &exe_path, remote_socket.as_str());

            std::fs::write(&config_path, config)?;
            let _ = writeln!(stdout(), "Wrote config at {}", config_path.display().to_string().bold());
        } else {
            std::fs::write(&config_path, self.ssh_config_header())?;
            let _ = writeln!(
                stdout(),
                "Cleared config at {}",
                config_path.display().to_string().bold()
            );
        }

        Ok(())
    }

    fn ssh_config_header(&self) -> String {
        let remote_username = self.remote_username.as_deref().unwrap_or_default();
        let remote_host = self.remote_host.as_deref().unwrap_or_default();
        let remote_port = self.remote_port.as_deref().unwrap_or_default();
        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();

        formatdoc! {"
            # This file is automatically @generated by {PRODUCT_NAME}.
            # It is not intended for manual editing.
            # 
            # This config was generated based on the following arguments:
            #
            # [args]
            # remote-host = {remote_host:?}
            # remote-port = {remote_port:?}
            # remote-username = {remote_username:?}
            # timestamp = {timestamp:?}
        "}
    }

    fn ssh_config(&self, uuid: &Uuid, exe_path: &str, remote_socket: &str) -> String {
        let header = self.ssh_config_header();
        let uuid = uuid.simple();
        let set_parent_socket_path = format!("/tmp/{CLI_BINARY_NAME}-parent-{uuid}.socket");

        formatdoc! {"
            {header}

            Match all
              RemoteForward '{set_parent_socket_path}' '{remote_socket}'
              SetEnv Q_SET_PARENT={set_parent_socket_path}
              StreamLocalBindMask 600
              StreamLocalBindUnlink yes
              PermitLocalCommand yes
              LocalCommand {exe_path} _ ssh-local-command '%r@%n' '{uuid}' 1>&2
        "}
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use fig_util::CLI_BINARY_NAME;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_ssh_config() {
        let args = GenerateSshArgs {
            remote_username: Some("root".into()),
            remote_host: Some("127.0.0.1".into()),
            remote_port: Some("22".into()),
        };

        let uuid = Uuid::new_v4();
        let exe_path = Path::new("/usr/bin").join(CLI_BINARY_NAME);
        let remote_socket = "/tmp/remote.socket";

        let config = args.ssh_config(&uuid, exe_path.to_str().unwrap(), remote_socket);
        println!("{config}");
    }
}
