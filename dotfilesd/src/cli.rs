use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use clap::{ArgEnum, Parser, Subcommand};
use dirs::home_dir;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ArgEnum)]
pub enum Shells {
    Bash,
    Zsh,
    Fish,
}

impl Shells {
    pub fn get_config_path(&self) -> Result<PathBuf> {
        let home_dir = home_dir().context("Could not get home directory")?;

        Ok(match self {
            Shells::Bash => home_dir.join(".bashrc"),
            Shells::Zsh => home_dir.join(".zshrc"),
            Shells::Fish => home_dir.join(".config/fish/config.fish"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ArgEnum)]
pub enum When {
    Pre,
    Post,
}

#[derive(Debug, Subcommand)]
pub enum CliRootCommands {
    /// Install dotfiles
    Install,
    /// Uninstall dotfiles
    Uninstall,
    /// Update dotfiles
    Update {
        /// Force update
        #[clap(long, short)]
        force: bool,
    },
    /// Run the daemon
    Daemon,
    /// Generate the dotfiles for the given shell
    Shell {
        /// The shell to generate the dotfiles for
        #[clap(arg_enum)]
        shell: Shells,
        /// When to generate the dotfiles for
        #[clap(arg_enum)]
        when: When,
    },
    /// Sync your latest dotfiles
    Sync,
}

#[derive(Debug, Parser)]
#[clap(
    name = "dotfiles",
    about = "A tool for managing dotfiles",
    version = "0.1.0"
)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<CliRootCommands>,
}

impl Cli {
    pub fn execute(self) {
        match self.subcommand {
            Some(subcommand) => match subcommand {
                CliRootCommands::Install => {
                    if let Err(e) = install() {
                        eprintln!("{:?}", e);
                    }
                }
                CliRootCommands::Uninstall => {
                    if let Err(e) = uninstall() {
                        eprintln!("{:?}", e);
                    }
                }
                CliRootCommands::Update { .. } => update(),
                CliRootCommands::Daemon => daemon(),
                CliRootCommands::Shell { shell, when } => {
                    println!("# {:?} for {:?}", when, shell);
                }
                CliRootCommands::Sync => sync(),
            },
            // Root command
            None => {
                // Open the default browser to the homepage
                const URL: &str = "https://dotfiles.com/";
                Command::new("open").arg(URL).output().unwrap();
            }
        }
    }
}

fn install() -> Result<()> {
    // Install dotfiles
    install_dotfiles().context("Could not install dotfiles")?;

    // Install daemons
    #[cfg(target_os = "macos")]
    install_daemon_macos().context("Could not install macOS daemon")?;
    #[cfg(target_os = "linux")]
    install_daemon_linux().context("Could not install systemd daemon")?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn install_daemon_macos() -> Result<()> {
    let plist = include_str!("daemon_files/com.dotfiles.Daemon.plist");
    let plist_path = "/Library/LaunchDaemons/com.dotfiles.Daemon.plist";
    std::fs::write(plist_path, plist)
        .with_context(|| format!("Could not write to {}", plist_path))?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn install_daemon_linux() -> Result<()> {
    let service = include_str!("daemon_files/dotfiles-Daemon.service");
    let service_path = "/etc/systemd/system/dotfiles-Daemon.service";
    std::fs::write(service_path, service)
        .with_context(|| format!("Could not write to {}", service_path))?;

    Ok(())
}

fn install_dotfiles() -> Result<()> {
    for shell in [Shells::Bash, Shells::Zsh, Shells::Fish].into_iter() {
        if let Ok(path) = shell.get_config_path() {
            if path.exists() {
                // Prepend and append the dotfiles
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let mut modified = false;
                let mut lines = vec![];

                let pre_eval = match shell {
                    Shells::Bash => "eval \"$(dotfiles shell bash pre)\"",
                    Shells::Zsh => "eval \"$(dotfiles shell zsh pre)\"",
                    Shells::Fish => "eval (dotfiles shell fish pre)",
                };

                if !contents.contains(pre_eval) {
                    lines.push("# Pre dotfiles eval");
                    lines.push(pre_eval);
                    lines.push("");

                    modified = true;
                }
                
                lines.extend(contents.lines());

                let post_eval = match shell {
                    Shells::Bash => "eval \"$(dotfiles shell bash post)\"",
                    Shells::Zsh => "eval \"$(dotfiles shell zsh post)\"",
                    Shells::Fish => "eval (dotfiles shell fish post)",
                };

                if !contents.contains(post_eval) {
                    lines.push("");
                    lines.push("# Post dotfiles eval");
                    lines.push(post_eval);
                    lines.push("");

                    modified = true;
                }

                if modified {
                    let mut file = File::create(&path)?;
                    file.write_all(lines.join("\n").as_bytes())?;
                }
            }
        }
    }

    Ok(())
}

fn uninstall() -> Result<()> {
    // Uninstall daemons
    #[cfg(target_os = "macos")]
    uninstall_daemon_macos()?;
    #[cfg(target_os = "linux")]
    uninstall_daemon_linux()?;

    // Delete the binary
    let binary_path = Path::new("/usr/local/bin/dotfiles");

    if binary_path.exists() {
        std::fs::remove_file(binary_path)
            .with_context(|| format!("Could not delete {}", binary_path.display()))?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_daemon_macos() -> Result<()> {
    let plist_path = Path::new("/Library/LaunchDaemons/com.dotfiles.Daemon.plist");

    if plist_path.exists() {
        std::fs::remove_file(plist_path)
            .with_context(|| format!("Could not delete {}", plist_path.display()))?;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_daemon_linux() -> Result<()> {
    let service_path = Path::new("/etc/systemd/system/dotfiles-Daemon.service");

    if service_path.exists() {
        std::fs::remove_file(service_path)
            .with_context(|| format!("Could not delete {}", service_path.display()))?;
    }
}

fn update() {
    todo!();
}

fn daemon() {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
        println!("# Running daemon");
    }
}

fn sync() {
    todo!();
}

#[cfg(test)]
mod test {
    use clap::IntoApp;

    use super::*;

    #[test]
    fn debug_assert() {
        Cli::into_app().debug_assert();
    }
}
