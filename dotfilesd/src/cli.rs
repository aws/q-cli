use std::fmt::Display;
use std::fs::File;
use std::io::{stdout, Read, Write, stdin};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::time::Duration;
use std::{any, env};

use anyhow::{Context, Result};
use aws_sdk_cognitoidentityprovider::{Client, Config};
use clap::{ArgEnum, Parser, Subcommand};
use crossterm::style::Stylize;
use dirs::home_dir;
use regex::Regex;
use self_update::cargo_crate_version;
use self_update::update::UpdateStatus;

use crate::auth::{get_client, SignInInput, SignUpInput};
use crate::cli;

/// Ensure the command is being run with root privileges.
/// If not, rexecute the command with sudo.
fn permission_guard() -> Result<()> {
    #[cfg(unix)]
    {
        use nix::unistd::geteuid;

        // Hack to persist the ZDOTDIR environment variable to the new process.
        if let Some(val) = env::var_os("ZDOTDIR") {
            if env::var_os("FIG_ZDOTDIR").is_none() {
                env::set_var("FIG_ZDOTDIR", val);
            }
        }

        match geteuid().is_root() {
            true => Ok(()),
            false => {
                let mut child = Command::new("sudo")
                    .arg("-E")
                    .args(env::args_os())
                    .spawn()?;

                let status = child.wait()?;

                exit(status.code().unwrap_or(1));
            }
        }
    }

    #[cfg(windows)]
    {
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ArgEnum)]
pub enum Shells {
    Bash,
    Zsh,
    Fish,
}

impl Display for Shells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shells::Bash => write!(f, "bash"),
            Shells::Zsh => write!(f, "zsh"),
            Shells::Fish => write!(f, "fish"),
        }
    }
}

impl Shells {
    pub fn get_config_path(&self) -> Result<PathBuf> {
        let home_dir = home_dir().context("Could not get home directory")?;

        let path = match self {
            Shells::Bash => home_dir.join(".bashrc"),
            Shells::Zsh => match env::var("ZDOTDIR")
                .or_else(|_| env::var("FIG_ZDOTDIR"))
                .map(PathBuf::from)
            {
                Ok(zdotdir) => {
                    let zdot_path = zdotdir.join(".zshrc");
                    if zdot_path.exists() {
                        zdot_path
                    } else {
                        home_dir.join(".zshrc")
                    }
                }
                Err(_) => home_dir.join(".zshrc"),
            },
            Shells::Fish => home_dir.join(".config/fish/config.fish"),
        };

        Ok(path)
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
        #[clap(long, short = 'y')]
        no_confirm: bool,
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
    /// Login to dotfiles
    Login,
    /// Doctor
    Doctor,
}

#[derive(Debug, Parser)]
#[clap(version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<CliRootCommands>,
}

impl Cli {
    pub async fn execute(self) {
        let result = match self.subcommand {
            Some(subcommand) => match subcommand {
                CliRootCommands::Install => install(),
                CliRootCommands::Uninstall => uninstall(),
                CliRootCommands::Update { no_confirm } => update(if no_confirm {
                    UpdateType::NoConfirm
                } else {
                    UpdateType::Confirm
                })
                .map(|_| ()),
                CliRootCommands::Daemon => daemon().await,
                CliRootCommands::Shell { shell, when } => {
                    println!("# {:?} for {:?}", when, shell);
                    println!("echo 'hello from the dotfiles {:?}'", when);

                    Ok(())
                }
                CliRootCommands::Sync => sync().await,
                CliRootCommands::Login => login().await,
                CliRootCommands::Doctor => doctor(),
            },
            // Root command
            None => {
                // Open the default browser to the homepage
                let url = "https://dotfiles.com/";
                if open_url(url).is_err() {
                    println!("{}", url);
                }

                Ok(())
            }
        };

        if let Err(e) = result {
            eprintln!("{:?}", e);
            exit(1);
        }
    }
}

fn install() -> Result<()> {
    permission_guard()?;

    // Install daemons
    #[cfg(target_os = "macos")]
    install_daemon_macos().context("Could not install macOS daemon")?;
    #[cfg(target_os = "linux")]
    install_daemon_linux().context("Could not install systemd daemon")?;
    #[cfg(target_os = "windows")]
    install_daemon_windows().context("Could not install Windows daemon")?;
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    unimplemented!();

    loop {
        print!("Do you want dotfiles to modify your shell config (you will have to manually do this otherwise)? [Y/n] ");
        stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "Y" | "y" | "" => {
                // Install dotfiles
                install_dotfiles().context("Could not install dotfiles")?;
                break;
            }
            "N" | "n" => {
                println!();
                println!("To install dotfiles you will have to add the following to your rc files");
                println!();
                println!(
                    "At the top of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:"
                );
                println!("bashrc:    eval \"$(dotfilesd shell bash pre)\"");
                println!("zshrc:     eval \"$(dotfilesd shell zsh pre)\"");
                println!("fish:      eval \"$(dotfilesd shell fish pre)\"");
                println!();
                println!("At the bottom of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:");
                println!("bashrc:    eval \"$(dotfilesd shell bash post)\"");
                println!("zshrc:     eval \"$(dotfilesd shell zsh post)\"");
                println!("fish:      eval \"$(dotfilesd shell fish post)\"");
                println!();

                break;
            }
            _ => {
                println!("Please enter y, n, or nothing");
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn install_daemon_macos() -> Result<()> {
    // Put the daemon plist in /Library/LaunchDaemons
    let plist = include_str!("daemon_files/io.fig.dotfiles-daemon.plist");
    let plist_path = "/Library/LaunchDaemons/io.fig.dotfiles-daemon.plist";
    std::fs::write(plist_path, plist)
        .with_context(|| format!("Could not write to {}", plist_path))?;

    // Start the daemon using launchctl
    Command::new("launchctl")
        .arg("load")
        .arg(plist_path)
        .output()
        .with_context(|| format!("Could not load {}", plist_path))?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn install_daemon_linux() -> Result<()> {
    // Put the daemon service in /usr/lib/systemd/system
    let service = include_str!("daemon_files/dotfiles-daemon.service");
    let service_path = "/usr/lib/systemd/system/dotfiles-daemon.service";
    std::fs::write(service_path, service)
        .with_context(|| format!("Could not write to {}", service_path))?;

    // Enable the daemon using systemctl
    Command::new("systemctl")
        .arg("--now")
        .arg("enable")
        .arg(service_path)
        .output()
        .with_context(|| format!("Could not enable {}", service_path))?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn install_daemon_windows() -> Result<()> {
    // Put the daemon service in %APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
    // let service = include_str!("daemon_files/dotfiles-daemon.bat");
    // let service_path = r"%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\dotfiles-daemon.bat";
    // std::fs::write(service_path, service)
    //     .with_context(|| format!("Could not write to {}", service_path))?;

    Ok(())
}

fn install_dotfiles() -> Result<()> {
    for shell in [Shells::Bash, Shells::Zsh, Shells::Fish] {
        if let Ok(path) = shell.get_config_path() {
            if path.exists() {
                // Prepend and append the dotfiles
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let mut modified = false;
                let mut lines = vec![];

                let pre_eval = match shell {
                    Shells::Bash => "eval \"$(dotfilesd shell bash pre)\"",
                    Shells::Zsh => "eval \"$(dotfilesd shell zsh pre)\"",
                    Shells::Fish => "eval (dotfilesd shell fish pre)",
                };

                if !contents.contains(pre_eval) {
                    lines.push("# Pre dotfilesd eval");
                    lines.push(pre_eval);
                    lines.push("");

                    modified = true;
                }

                lines.extend(contents.lines());

                let post_eval = match shell {
                    Shells::Bash => "eval \"$(dotfilesd shell bash post)\"",
                    Shells::Zsh => "eval \"$(dotfilesd shell zsh post)\"",
                    Shells::Fish => "eval (dotfilesd shell fish post)",
                };

                if !contents.contains(post_eval) {
                    lines.push("");
                    lines.push("# Post dotfilesd eval");
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

fn uninstall_dotfiles() -> Result<()> {
    for shell in [Shells::Bash, Shells::Zsh, Shells::Fish] {
        if let Ok(path) = shell.get_config_path() {
            if path.exists() {
                // Prepend and append the dotfiles
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let pre_eval = match shell {
                    Shells::Bash => Regex::new(
                        r#"(?:# Pre dotfilesd eval\n)?eval "\$\(dotfilesd shell bash pre\)"\n{0,2}"#,
                    ),
                    Shells::Zsh => Regex::new(
                        r#"(?:# Pre dotfilesd eval\n)?eval "\$\(dotfilesd shell zsh pre\)"\n{0,2}"#,
                    ),
                    Shells::Fish => Regex::new(
                        r#"(?:# Pre dotfilesd eval\n)?eval \(dotfilesd shell fish pre\)\n{0,2}"#,
                    ),
                }
                .unwrap();

                let contents = pre_eval.replace_all(&contents, "");

                let post_eval_regex = match shell {
                    Shells::Bash => Regex::new(
                        r#"(?:# Post dotfilesd eval\n)?eval "\$\(dotfilesd shell bash post\)"\n{0,2}"#,
                    ),
                    Shells::Zsh => Regex::new(
                        r#"(?:# Post dotfilesd eval\n)?eval "\$\(dotfilesd shell zsh post\)"\n{0,2}"#,
                    ),
                    Shells::Fish => Regex::new(
                        r#"(?:# Post dotfilesd eval\n)?eval \(dotfilesd shell fish post\)\n{0,2}"#,
                    ),
                }
                .unwrap();

                let contents = post_eval_regex.replace_all(&contents, "");

                let mut file = File::create(&path)?;
                file.write_all(contents.as_bytes())?;
            }
        }
    }

    Ok(())
}

/// Uninstall dotfiles
fn uninstall() -> Result<()> {
    permission_guard()?;

    // Uninstall daemons
    #[cfg(target_os = "macos")]
    uninstall_daemon_macos()?;
    #[cfg(target_os = "linux")]
    uninstall_daemon_linux()?;
    #[cfg(target_os = "windows")]
    uninstall_daemon_windows()?;
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    unimplemented!();

    // Uninstall dotfiles
    loop {
        print!("Do you want dotfiles to modify your shell config (you will have to manually do this otherwise)? [Y/n] ");
        stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim() {
            "Y" | "y" | "" => {
                uninstall_dotfiles().context("Could not uninstall dotfiles")?;
                break;
            }
            "N" | "n" => {
                println!();
                println!(
                    "To uninstall dotfiles you will have to remove the following from your rc files"
                );
                println!();
                println!(
                    "At the top of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:"
                );
                println!("bashrc:    eval \"$(dotfilesd shell bash pre)\"");
                println!("zshrc:     eval \"$(dotfilesd shell zsh pre)\"");
                println!("fish:      eval \"$(dotfilesd shell fish pre)\"");
                println!();
                println!("At the bottom of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:");
                println!("bashrc:    eval \"$(dotfilesd shell bash post)\"");
                println!("zshrc:     eval \"$(dotfilesd shell zsh post)\"");
                println!("fish:      eval \"$(dotfilesd shell fish post)\"");
                println!();

                break;
            }
            _ => {
                println!("Please enter y, n, or nothing");
            }
        }
    }

    // Delete the binary
    let binary_path = Path::new("/usr/local/bin/dotfilesd");

    if binary_path.exists() {
        std::fs::remove_file(binary_path)
            .with_context(|| format!("Could not delete {}", binary_path.display()))?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_daemon_macos() -> Result<()> {
    // Stop the daemon using launchctl
    Command::new("launchctl")
        .arg("unload")
        .arg("/Library/LaunchDaemons/io.fig.dotfiles-daemon.plist")
        .output()
        .with_context(|| "Could not unload io.fig.dotfiles-daemon.plist")?;

    // Delete the daemon plist
    let plist_path = Path::new("/Library/LaunchDaemons/io.fig.dotfiles-daemon.plist");

    if plist_path.exists() {
        std::fs::remove_file(plist_path)
            .with_context(|| format!("Could not delete {}", plist_path.display()))?;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_daemon_linux() -> Result<()> {
    // Disable the daemon using systemctl
    Command::new("systemctl")
        .arg("disable")
        .arg("/usr/lib/systemd/system/dotfilesd-daemon.service")
        .output()
        .with_context(|| "Could not disable dotfilesd-daemon.service")?;

    // Delete the daemon service
    let service_path = Path::new("/etc/systemd/system/dotfiles-daemon.service");

    if service_path.exists() {
        std::fs::remove_file(service_path)
            .with_context(|| format!("Could not delete {}", service_path.display()))?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn uninstall_daemon_windows() -> Result<()> {
    // Delete the daemon service
    let service_path = Path::new(
        "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\dotfilesd-daemon.exe",
    );

    if service_path.exists() {
        std::fs::remove_file(service_path)
            .with_context(|| format!("Could not delete {}", service_path.display()))?;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum UpdateType {
    Confirm,
    NoConfirm,
    NoProgress,
}

/// Self-update the dotfiles binary
/// Update will exit the binary if the update was successful
fn update(update_type: UpdateType) -> Result<UpdateStatus> {
    permission_guard()?;

    let confirm = match update_type {
        UpdateType::Confirm => true,
        UpdateType::NoConfirm => false,
        UpdateType::NoProgress => false,
    };

    let progress_output = match update_type {
        UpdateType::Confirm => true,
        UpdateType::NoConfirm => true,
        UpdateType::NoProgress => false,
    };

    tokio::task::block_in_place(move || {
        let current_version = env!("CARGO_PKG_VERSION");

        let update = self_update::backends::s3::Update::configure()
            .bucket_name("get-fig-io")
            .asset_prefix("bin")
            .region("us-west-1")
            .bin_name("dotfilesd")
            .current_version(current_version)
            .no_confirm(true)
            .show_output(false)
            .show_download_progress(progress_output)
            .build()?;

        let latest_release = update.get_latest_release()?;

        if !self_update::version::bump_is_greater(current_version, &latest_release.version)? {
            println!("You are already on the latest version");

            return Ok(UpdateStatus::UpToDate);
        }

        if confirm {
            loop {
                print!(
                    "Do you want to update {} from {} to {}? [Y/n] ",
                    env!("CARGO_PKG_NAME"),
                    update.current_version(),
                    latest_release.version
                );
                stdout().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                match input.trim() {
                    "Y" | "y" | "" => break,
                    "N" | "n" => {
                        println!();
                        println!("Update cancelled");
                        return Err(anyhow::anyhow!("Update cancelled"));
                    }
                    _ => {
                        println!("Please enter y, n, or nothing");
                    }
                }
            }
        } else {
            println!(
                "Updating {} from {} to {}",
                env!("CARGO_PKG_NAME"),
                update.current_version(),
                latest_release.version
            );
        }

        Ok(update.update_extended()?)
    })
}

/// Spawn the daemon to listen for updates and dotfiles changes
async fn daemon() -> Result<()> {
    // Connect to the web socket

    loop {
        // Check for updates
        match update(UpdateType::NoProgress)? {
            UpdateStatus::UpToDate => {},
            UpdateStatus::Updated(release) => {
                println!("Updated to {}", release.version);
                println!("Quitting...");
                return Ok(());
            },
        }

        // Sleep
        tokio::time::sleep(Duration::from_secs(60 * 60)).await;
    }
}

/// Download the lastest dotfiles
async fn sync() -> Result<()> {
    let dotfiles = reqwest::get("https://dotfiles.com/").await?.text().await?;
    println!("{}", dotfiles);
    todo!();
}

/// Login to the dotfiles server
async fn login() -> Result<()> {
    let client = get_client("dotfilesd")?;

    // 	fmt.Print("Client ID: ")
	// fmt.Scanln(&clientId)
	// fmt.Print("Username or email: ")
	// fmt.Scanln(&usernameOrEmail)

    print!("Client ID: ");
    stdout().flush()?;

    let mut client_id = String::new();
    stdin().read_line(&mut client_id)?;

    print!("Username or email: ");
    stdout().flush()?;

    let mut username_or_email = String::new();
    stdin().read_line(&mut username_or_email)?;

    let client_id = client_id.trim();
    let username_or_email = username_or_email.trim();

    let signup = SignUpInput::new(client, client_id, username_or_email);

    signup.sign_up().await?;

    Ok(())
}

// Doctor
fn doctor() -> Result<()> {
    println!("Checking dotfiles...");
    println!();

    for shell in [Shells::Bash, Shells::Zsh, Shells::Fish] {
        println!("Checking {:?}...", shell);

        if let Ok(config_path) = shell.get_config_path() {
            if config_path.exists() {
                println!("✅ {} dotfiles exist at {}", shell, config_path.display());

                let mut config_file = File::open(config_path)?;
                let mut config_contents = String::new();
                config_file.read_to_string(&mut config_contents)?;

                let pre_eval_regex = match shell {
                    Shells::Bash => Regex::new(r#"eval "\$\(dotfilesd shell bash pre\)""#),
                    Shells::Zsh => Regex::new(r#"eval "\$\(dotfilesd shell zsh pre\)""#),
                    Shells::Fish => Regex::new(r#"eval \(dotfilesd shell fish pre\)"#),
                }
                .unwrap();

                if pre_eval_regex.is_match(&config_contents) {
                    println!("✅ `dotfiles shell {} pre` exists", shell);
                } else {
                    println!("❌ `dotfiles shell {} pre` does not exist", shell);
                }

                let post_eval_regex = match shell {
                    Shells::Bash => Regex::new(r#"eval "\$\(dotfilesd shell bash post\)""#),
                    Shells::Zsh => Regex::new(r#"eval "\$\(dotfilesd shell zsh post\)""#),
                    Shells::Fish => Regex::new(r#"eval \(dotfilesd shell fish post\)"#),
                }
                .unwrap();

                if post_eval_regex.is_match(&config_contents) {
                    println!("✅ `dotfiles shell {} post` exists", shell);
                } else {
                    println!("❌ `dotfiles shell {} post` does not exist", shell);
                }
            } else {
                println!("{} does not exist", config_path.display());
            }
        }
        println!();
    }

    println!();
    println!("dotfilesd appears to be installed correctly");
    println!("If you have any issues, please report them at");
    println!("hello@fig.io or https://github.com/withfig/fig");
    println!();

    Ok(())
}

fn open_url(url: impl AsRef<str>) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url.as_ref())
            .output()
            .with_context(|| "Could not open url")?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url.as_ref())
            .output()
            .with_context(|| "Could not open url")?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg(url.as_ref())
            .output()
            .with_context(|| "Could not open url")?;
        
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    unimplemented!();
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
