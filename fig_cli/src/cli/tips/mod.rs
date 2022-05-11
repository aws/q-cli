use std::fs::{
    self,
    File,
};

use anyhow::{
    Context,
    Result,
};
use clap::Subcommand;
use crossterm::style::Stylize;
use semver::Version;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;

#[derive(Debug, Subcommand)]
pub enum TipsSubcommand {
    /// Enable fig tips
    Enable,
    /// Disable fig tips
    Disable,
    /// Reset the tips to the default
    #[clap(hide = true)]
    Reset,
    /// Show the tips
    #[clap(hide = true)]
    Prompt,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Changelog {
    version: String,
    notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tip {
    id: String,
    text: String,
    sent: bool,
    priority: i64,
    wait_time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tips {
    last_changelog: String,
    time_last_sent: i64,
    queue: Vec<Tip>,
}

fn get_all_tips() -> Vec<Tip> {
    vec![
        Tip {
            id: "tip-1".into(),
            text: format!(
                "{} 🚀 Customize keybindings\n\nFig lets you customize keybindings for:\n  • inserting text (like \
                 tab/enter)\n  • navigating (like {} & {} arrow keys)\n  • toggling the description pop out (like \
                 ⌘+i)\n  • and more\n\nJust run {} and then select {}",
                "Fig tips (1/5):".bold(),
                "↑".bold(),
                "↓".bold(),
                "fig settings".bold().magenta(),
                "keybindings".underlined()
            ),
            priority: 10,
            wait_time: 60 * 10,
            sent: false,
        },
        Tip {
            id: "tip-2".into(),
            text: format!(
                "{} ⚙️  Adjust settings\n\nCustomize autocomplete's look and feel for things like:\n  • Width & \
                 height\n  • Font family, font size, theme\n  • Auto-execute functionality (e.g. allowing \
                 auto-execute after space)\n\nJust run {}",
                "Fig Tips (2/5)".bold(),
                "fig settings".bold().magenta()
            ),
            priority: 9,
            wait_time: 60 * 60 * 12,
            sent: false,
        },
        Tip {
            id: "tip-3".into(),
            text: format!(
                "{} 😎 Private autocomplete\n\nDid you know Fig lets you create private completions for your own \
                 personal\n shortcuts or even your team's internal CLI tool?\n\nBuild private completions in less \
                 than 2 minutes:\n  1. {} {}\n2. {} {}",
                "Fig Tips (3/5)".bold(),
                "Personal:".bold(),
                "fig.io/shortcuts".underlined(),
                "Team:".bold(),
                "fig.io/teams".underlined(),
            ),
            priority: 8,
            wait_time: 60 * 60 * 12,
            sent: false,
        },
        Tip {
            id: "tip-4".into(),
            text: format!(
                "{} 🎉 Share Fig with friends\n\nEnjoying Fig and think your friends & teammates would too?\n\nShare \
                 Fig with friends!\n\nClaim your custom invite link by running: {}",
                "Fig Tips (4/5)".bold(),
                "fig invite".bold().magenta(),
            ),
            priority: 7,
            wait_time: 60 * 60 * 12,
            sent: false,
        },
        Tip {
            id: "tip-5".into(),
            text: format!(
                "\n{} 🤗 Contribute to autocomplete for public CLIs\n\nMissing completions for a CLI? Finding some \
                 errors in completions\nfor an existing CLI?\n\nAll of Fig's completions for public CLI tools like \
                 cd, git, docker,\n kubectl are open source and community driven!\n\nStart contributing at: {}",
                "Fig Tips (5/5)".bold(),
                "github.com/withfig/autocomplete".underlined(),
            ),
            priority: 6,
            wait_time: 60 * 60 * 12,
            sent: false,
        },
    ]
}

impl Tips {
    fn save(&self) -> anyhow::Result<()> {
        let data_dir = fig_directories::fig_data_dir().context("Could not get fig data dir")?;
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }
        let mut file = File::create(data_dir.join("tips.json"))?;

        serde_json::to_writer(&mut file, self)?;

        Ok(())
    }

    fn load() -> anyhow::Result<Tips> {
        let path = fig_directories::fig_data_dir()
            .context("Could not get fig data dir")?
            .join("tips.json");

        if !path.exists() {
            return Err(anyhow::anyhow!("Could not find tips file"));
        }
        let file = File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }
}

impl TipsSubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            TipsSubcommand::Enable => {
                fig_settings::settings::set_value("cli.tips.disabled", json!(false)).await?;
                println!("\n→ Fig Tips enabled...\n");
            },
            TipsSubcommand::Disable => {
                fig_settings::settings::set_value("cli.tips.disabled", json!(true)).await?;
                println!("\n→ Fig Tips disabled...\n");
            },
            TipsSubcommand::Reset => {
                let mut tips = Tips::load()?;
                for tip in get_all_tips() {
                    if tips.queue.iter().any(|x| x.id == tip.id) {
                        println!("Error adding {}: already exists.", tip.id);
                    } else {
                        tips.queue.push(tip);
                    }
                }
                tips.save()?;
            },
            TipsSubcommand::Prompt => match fig_settings::settings::get_value("cli.tips.disabled")? {
                Some(json!(false)) => {},
                _ => {
                    let mut tips = Tips::load()?;
                    let unsent = tips
                        .queue
                        .iter_mut()
                        .filter(|x| !x.sent)
                        .max_by(|a, b| a.priority.cmp(&b.priority));
                    let now = time::OffsetDateTime::now_utc().unix_timestamp();
                    if let Some(tip) = unsent {
                        if now - tips.time_last_sent > tip.wait_time {
                            println!(
                                "\n{}\n\n{} fig tips disable\n{} fig issue\n",
                                tip.text,
                                "Disable Getting Started Tips:".underlined(),
                                "Report a bug:".underlined(),
                            );
                            tip.sent = true;
                            tips.time_last_sent = now;
                        }
                    } else {
                        let changelog: Changelog = serde_json::from_str(include_str!("../../../../changelog.json"))?;
                        if Version::parse(&tips.last_changelog)? < Version::parse(&changelog.version)? {
                            println!("{}", changelog.notes);
                            tips.last_changelog = changelog.version;
                            tips.time_last_sent = now;
                        }
                    }
                    tips.save()?;
                },
            },
        }
        Ok(())
    }
}
