use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

use anyhow::{Context, Result};
use clap::{ArgEnum, Parser, Subcommand};
use crossterm::style::Stylize;
use dialoguer::theme::ColorfulTheme;
use dirs::{cache_dir, home_dir};
use regex::Regex;
use self_update::update::UpdateStatus;
use serde::{Deserialize, Serialize};
use tokio::try_join;
use url::Url;

use crate::auth::{
    get_client, Credentials, SignInConfirmError, SignInError, SignInInput, SignUpConfirmError,
    SignUpInput,
};
use crate::daemon::daemon;

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

        let sudo_prompt = match env::var("USER") {
            Ok(user) => format!("Please enter your password for user {}: ", user),
            Err(_) => "Please enter your password: ".to_string(),
        };

        match geteuid().is_root() {
            true => Ok(()),
            false => {
                let mut child = Command::new("sudo")
                    .arg("-E")
                    .arg("-p")
                    .arg(sudo_prompt)
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

fn dialoguer_theme() -> impl dialoguer::theme::Theme {
    let mut theme = ColorfulTheme::default();

    theme.prompt_prefix = dialoguer::console::style("?".to_string())
        .for_stderr()
        .magenta();

    theme
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
    Login {
        #[clap(long, short)]
        refresh: bool,
    },
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
                CliRootCommands::Login { refresh } => login(refresh).await,
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

    match dialoguer::Confirm::with_theme(&dialoguer_theme())
        .with_prompt("Do you want dotfiles to modify your shell config (you will have to manually do this otherwise)?")
        .interact()?
    {
        true => {
            install_dotfiles().context("Could not install dotfiles")?;
        }
        false => {
            println!();
            println!("To install dotfiles you will have to add the following to your rc files");
            println!();
            println!(
                "At the top of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:"
            );
            println!("bashrc:    eval \"$(dotfiles shell bash pre)\"");
            println!("zshrc:     eval \"$(dotfiles shell zsh pre)\"");
            println!("fish:      eval \"$(dotfiles shell fish pre)\"");
            println!();
            println!(
                "At the bottom of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:"
            );
            println!("bashrc:    eval \"$(dotfiles shell bash post)\"");
            println!("zshrc:     eval \"$(dotfiles shell zsh post)\"");
            println!("fish:      eval \"$(dotfiles shell fish post)\"");
            println!();
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
                        r#"(?:# Pre dotfiles eval\n)?eval "\$\(dotfiles shell bash pre\)"\n{0,2}"#,
                    ),
                    Shells::Zsh => Regex::new(
                        r#"(?:# Pre dotfiles eval\n)?eval "\$\(dotfiles shell zsh pre\)"\n{0,2}"#,
                    ),
                    Shells::Fish => Regex::new(
                        r#"(?:# Pre dotfiles eval\n)?eval \(dotfiles shell fish pre\)\n{0,2}"#,
                    ),
                }
                .unwrap();

                let contents = pre_eval.replace_all(&contents, "");

                let post_eval_regex = match shell {
                    Shells::Bash => Regex::new(
                        r#"(?:# Post dotfiles eval\n)?eval "\$\(dotfiles shell bash post\)"\n{0,2}"#,
                    ),
                    Shells::Zsh => Regex::new(
                        r#"(?:# Post dotfiles eval\n)?eval "\$\(dotfiles shell zsh post\)"\n{0,2}"#,
                    ),
                    Shells::Fish => Regex::new(
                        r#"(?:# Post dotfiles eval\n)?eval \(dotfiles shell fish post\)\n{0,2}"#,
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
    match dialoguer::Confirm::with_theme(&dialoguer_theme())
        .with_prompt("Do you want dotfiles to modify your shell config (you will have to manually do this otherwise)?")
        .interact()? {
            true => {
                uninstall_dotfiles().context("Could not uninstall dotfiles")?;
            },
            false => {
                println!();
                println!(
                    "To uninstall dotfiles you will have to remove the following from your rc files"
                );
                println!();
                println!(
                    "At the top of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:"
                );
                println!("bashrc:    eval \"$(dotfiles shell bash pre)\"");
                println!("zshrc:     eval \"$(dotfiles shell zsh pre)\"");
                println!("fish:      eval \"$(dotfiles shell fish pre)\"");
                println!();
                println!("At the bottom of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:");
                println!("bashrc:    eval \"$(dotfiles shell bash post)\"");
                println!("zshrc:     eval \"$(dotfiles shell zsh post)\"");
                println!("fish:      eval \"$(dotfiles shell fish post)\"");
                println!();
            },
    }

    // Delete the binary
    let binary_path = Path::new("/usr/local/bin/dotfiles");

    if binary_path.exists() {
        std::fs::remove_file(binary_path)
            .with_context(|| format!("Could not delete {}", binary_path.display()))?;
    }

    println!("\n{}\n", "Dotfiles has been uninstalled".bold());

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
        .arg("/usr/lib/systemd/system/dotfiles-daemon.service")
        .output()
        .with_context(|| "Could not disable dotfiles-daemon.service")?;

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
        "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\dotfiles-daemon.exe",
    );

    if service_path.exists() {
        std::fs::remove_file(service_path)
            .with_context(|| format!("Could not delete {}", service_path.display()))?;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum UpdateType {
    Confirm,
    NoConfirm,
    NoProgress,
}

/// Self-update the dotfiles binary
/// Update will exit the binary if the update was successful
pub fn update(update_type: UpdateType) -> Result<UpdateStatus> {
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
            .bin_name("dotfiles")
            .current_version(current_version)
            .no_confirm(true)
            .show_output(false)
            .show_download_progress(progress_output)
            .build()?;

        let latest_release = update.get_latest_release()?;

        if !self_update::version::bump_is_greater(current_version, &latest_release.version)? {
            println!("You are already on the latest version {}", current_version);

            return Ok(UpdateStatus::UpToDate);
        }

        if confirm {
            if !dialoguer::Confirm::with_theme(&dialoguer_theme())
                .with_prompt(format!(
                    "Do you want to update {} from {} to {}?",
                    env!("CARGO_PKG_NAME"),
                    update.current_version(),
                    latest_release.version
                ))
                .default(true)
                .interact()?
            {
                return Err(anyhow::anyhow!("Update cancelled"));
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfilesSourceRequest {
    email: String,
}

async fn sync_file(shell: &Shells) -> Result<()> {
    // Get the access token from defaults
    let token = Command::new("defaults")
        .arg("read")
        .arg("com.mschrage.fig")
        .arg("access_token")
        .output()
        .with_context(|| "Could not read access_token")?;

    let email = Credentials::load_credentials()
        .map(|creds| creds.email)
        .or_else(|_| {
            let out = Command::new("defaults")
                .arg("read")
                .arg("com.mschrage.fig")
                .arg("userEmail")
                .output()?;

            let email = String::from_utf8(out.stdout)?;

            anyhow::Ok(Some(email))
        })?;

    // Constuct the request body
    let body = serde_json::to_string(&DotfilesSourceRequest {
        email: email.unwrap_or_else(|| "".to_string()),
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
pub async fn sync() -> Result<()> {
    try_join!(
        sync_file(&Shells::Bash),
        sync_file(&Shells::Zsh),
        sync_file(&Shells::Fish),
    )?;

    println!("Dotfiles synced!");

    Ok(())
}

/// Login to the dotfiles server
async fn login(refresh: bool) -> Result<()> {
    let client_id = "hkinciohdp1i7h0imdk63a4bv";
    let client = get_client("dotfiles")?;

    if refresh {
        let mut creds = Credentials::load_credentials()?;
        creds.refresh_credentials(&client, client_id).await?;
        creds.save_credentials()?;
        return Ok(());
    }

    println!("{}", "Login to Fig dotfiles".bold().magenta());

    let theme = dialoguer_theme();

    let email: String = dialoguer::Input::with_theme(&theme)
        .with_prompt("Email")
        .validate_with(|input: &String| -> Result<(), &str> {
            if validator::validate_email(input.trim()) {
                Ok(())
            } else {
                Err("This is not a valid email")
            }
        })
        .interact_text()?;

    let trimmed_email = email.trim();

    let sign_in_input = SignInInput::new(&client, client_id, trimmed_email);

    println!("Sending login code to {}...", trimmed_email);
    println!("Please check your email for the code");

    match sign_in_input.sign_in().await {
        Ok(mut sign_in_output) => {
            loop {
                let login_code: String = dialoguer::Input::with_theme(&theme)
                    .with_prompt("Login code")
                    .interact_text()?;

                match sign_in_output.confirm(login_code.trim()).await {
                    Ok(creds) => {
                        creds.save_credentials()?;
                        println!("Login successful!");
                        return Ok(());
                    }
                    Err(err) => match err {
                        SignInConfirmError::ErrorCodeMismatch => {
                            println!("Code mismatch, try again...");
                            continue;
                        }
                        SignInConfirmError::NotAuthorized => {
                            return Err(anyhow::anyhow!("Not authorized, you may have entered the wrong code too many times."));
                        }
                        err => return Err(err.into()),
                    },
                };
            }
        }
        Err(err) => match err {
            SignInError::UserNotFound(_) => {
                let mut sign_up_output = SignUpInput::new(&client, client_id, email)
                    .sign_up()
                    .await?;

                loop {
                    let login_code: String = dialoguer::Input::with_theme(&theme)
                        .with_prompt("Login code")
                        .interact_text()?;

                    match sign_up_output.confirm(login_code.trim()).await {
                        Ok(creds) => {
                            creds.save_credentials()?;
                            println!("Login successful!");
                            return Ok(());
                        }
                        Err(err) => match err {
                            SignUpConfirmError::CodeMismatch(_) => {
                                println!("Code mismatch, try again...");
                                continue;
                            }
                            err => return Err(err.into()),
                        },
                    };
                }
            }
            err => Err(err.into()),
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
                    Shells::Bash => Regex::new(r#"eval "\$\(dotfiles shell bash pre\)""#),
                    Shells::Zsh => Regex::new(r#"eval "\$\(dotfiles shell zsh pre\)""#),
                    Shells::Fish => Regex::new(r#"eval \(dotfiles shell fish pre\)"#),
                }
                .unwrap();

                if pre_eval_regex.is_match(&config_contents) {
                    println!("✅ `dotfiles shell {} pre` exists", shell);
                } else {
                    println!("❌ `dotfiles shell {} pre` does not exist", shell);
                }

                let post_eval_regex = match shell {
                    Shells::Bash => Regex::new(r#"eval "\$\(dotfiles shell bash post\)""#),
                    Shells::Zsh => Regex::new(r#"eval "\$\(dotfiles shell zsh post\)""#),
                    Shells::Fish => Regex::new(r#"eval \(dotfiles shell fish post\)"#),
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

    // Check credentials to see if they are logged in
    println!("Checking login status...");
    if let Ok(creds) = Credentials::load_credentials() {
        if creds.get_access_token().is_some()
            && creds.get_id_token().is_some()
            && creds.get_refresh_token().is_some()
        {
            println!("✅ You are logged in");
        } else {
            println!("❌ You are not logged in");
            println!("   You can login with `dotfiles login`");
        }
    } else {
        println!("❌ You are not logged in");
        println!("   You can login with `dotfiles login`");
    }

    // Check if daemon is running
    // Send a ping to the daemon to see if it's running

    println!();
    println!("dotfiles appears to be installed correctly");
    println!("If you have any issues, please report them to");
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
