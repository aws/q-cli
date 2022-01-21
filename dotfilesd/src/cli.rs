use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{ArgEnum, Parser, Subcommand};
use dirs::{cache_dir, home_dir};
use regex::Regex;
use self_update::update::UpdateStatus;
use serde::{Deserialize, Serialize};
use tokio::{select, try_join};
use url::Url;

use crate::auth::{
    get_client, SignInConfirmError, SignInError, SignInInput, SignUpConfirmError, SignUpInput,
};

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

    pub fn get_cache_path(&self) -> Result<PathBuf> {
        Ok(cache_dir()
            .context("Could not get cache directory")?
            .join("fig")
            .join("dotfiles")
            .join(format!("{}.json", self)))
    }

    pub fn get_remote_source(&self) -> Result<Url> {
        Ok(format!("https://api.fig.io/dotfiles/source/{}", self).parse()?)
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
                    match shell_source(&shell, &when) {
                        Ok(source) => println!("{}", source),
                        Err(err) => println!("# Could not load source: {}", err),
                    }

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
    let mut update_interval = tokio::time::interval(Duration::from_secs(60 * 60));

    loop {
        select! {
            _ = update_interval.tick() => {
                // Check for updates
                match update(UpdateType::NoProgress)? {
                    UpdateStatus::UpToDate => {}
                    UpdateStatus::Updated(release) => {
                        println!("Updated to {}", release.version);
                        println!("Quitting...");
                        return Ok(());
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfilesSourceRequest {
    email: String,
}

async fn sync_file(shell: &Shells) -> Result<()> {
    // Run defaults read com.mschrage.fig access_token
    let token = Command::new("defaults")
        .arg("read")
        .arg("com.mschrage.fig")
        .arg("access_token")
        .output()
        .with_context(|| "Could not read access_token")?;

    // Constuct the request body
    let body = serde_json::to_string(&DotfilesSourceRequest {
        email: "sean@fig.io".into(),
    })?;

    let download = reqwest::Client::new()
        .get(shell.get_remote_source()?)
        .header(
            "Authorization",
            format!("Bearer {}", String::from_utf8_lossy(&token.stdout).trim()),
        )
        .body(body)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    // Create path to dotfiles
    let cache_file = shell.get_cache_path()?;
    let cache_folder = cache_file.parent().unwrap();

    // Create cache folder if it doesn't exist
    if !cache_folder.exists() {
        std::fs::create_dir_all(cache_folder)?;
    }

    let mut dest_file = std::fs::File::create(cache_file)?;
    dest_file.write_all(download.as_bytes())?;

    Ok(())
}

/// Download the lastest dotfiles
async fn sync() -> Result<()> {
    try_join!(
        sync_file(&Shells::Bash),
        sync_file(&Shells::Zsh),
        sync_file(&Shells::Fish),
    )?;

    Ok(())
}

/// Login to the dotfiles server
async fn login() -> Result<()> {
    let client_id = "hkinciohdp1i7h0imdk63a4bv";
    let client = get_client("dotfilesd")?;

    print!("Email: ");
    stdout().flush()?;

    let mut email = String::new();
    stdin().read_line(&mut email)?;

    let email = email.trim();

    let sign_in_input = SignInInput::new(&client, client_id, email);

    match sign_in_input.sign_in().await {
        Ok(mut sign_in_output) => loop {
            print!("Login Code: ");
            stdout().flush()?;

            let mut login_code = String::new();
            stdin().read_line(&mut login_code)?;

            match sign_in_output.confirm(login_code.trim()).await {
                Ok(creds) => {
                    creds.save_credentials()?;
                    println!("Logged in!");
                    return Ok(());
                }
                Err(err) => match err {
                    SignInConfirmError::ErrorCodeMismatch => {
                        println!("Code mismatch, try again");
                        continue;
                    }
                    err => {
                        return Err(err.into());
                    }
                },
            }
        },
        Err(err) => match err {
            SignInError::UserNotFound(_) => {
                let mut sign_up_output = SignUpInput::new(&client, client_id, email)
                    .sign_up()
                    .await?;

                loop {
                    print!("Login Code: ");
                    stdout().flush()?;

                    let mut login_code = String::new();
                    stdin().read_line(&mut login_code)?;

                    match sign_up_output.confirm(login_code.trim()).await {
                        Ok(creds) => {
                            creds.save_credentials()?;
                            println!("Logged in!");
                            return Ok(());
                        }
                        Err(err) => match err {
                            SignUpConfirmError::CodeMismatch(_) => {
                                println!("Code mismatch, try again");
                                continue;
                            }
                            err => {
                                return Err(err.into());
                            }
                        },
                    }
                }
            }
            err => return Err(err.into()),
        },
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfileData {
    dotfile: String,
}

fn shell_source(shell: &Shells, when: &When) -> Result<String> {
    let raw = std::fs::read_to_string(shell.get_cache_path()?)?;
    let source: DotfileData = serde_json::from_str(&raw)?;

    match when {
        When::Pre => Ok(String::new()),
        When::Post => Ok(source.dotfile),
    }
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
