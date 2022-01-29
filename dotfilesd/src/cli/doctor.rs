use std::{fs::File, io::Read};

use anyhow::Result;
use regex::Regex;

use crate::{auth::Credentials, util::shell::Shell};

// Doctor
pub fn doctor_cli() -> Result<()> {
    println!("Checking dotfiles...");
    println!();

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        println!("Checking {:?}...", shell);

        if let Ok(config_path) = shell.get_config_path() {
            if config_path.exists() {
                println!("✅ {} dotfiles exist at {}", shell, config_path.display());

                let mut config_file = File::open(config_path)?;
                let mut config_contents = String::new();
                config_file.read_to_string(&mut config_contents)?;

                let pre_eval_regex = match shell {
                    Shell::Bash => Regex::new(r#"eval "\$\(dotfiles shell bash pre\)""#),
                    Shell::Zsh => Regex::new(r#"eval "\$\(dotfiles shell zsh pre\)""#),
                    Shell::Fish => Regex::new(r#"eval \(dotfiles shell fish pre\)"#),
                }
                .unwrap();

                if pre_eval_regex.is_match(&config_contents) {
                    println!("✅ `dotfiles shell {} pre` exists", shell);
                } else {
                    println!("❌ `dotfiles shell {} pre` does not exist", shell);
                }

                let post_eval_regex = match shell {
                    Shell::Bash => Regex::new(r#"eval "\$\(dotfiles shell bash post\)""#),
                    Shell::Zsh => Regex::new(r#"eval "\$\(dotfiles shell zsh post\)""#),
                    Shell::Fish => Regex::new(r#"eval \(dotfiles shell fish post\)"#),
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
