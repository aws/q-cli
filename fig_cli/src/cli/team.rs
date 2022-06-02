use anyhow::{
    bail,
    Result,
};
use clap::{
    ArgEnum,
    Args,
    Subcommand,
};
use crossterm::style::Stylize;
use reqwest::Method;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    json,
    Value,
};

use super::OutputFormat;
use crate::cli::dialoguer_theme;
use crate::util::api::request;

#[derive(Debug, Args)]
pub struct TeamsArgs {
    // List all teams that the user is part of
    #[clap(long, conflicts_with_all = &["new", "delete"])]
    list: bool,
    // Create a new team
    #[clap(long, conflicts_with_all = &["list", "delete"])]
    new: bool,
    // Delete an existing team
    #[clap(long, conflicts_with_all = &["list", "new"])]
    delete: bool,
    // Format of output
    #[clap(short, long, arg_enum, default_value_t)]
    format: OutputFormat,
}

#[derive(Debug, Args)]
pub struct TeamCommand {
    pub team: Option<String>,
    #[clap(subcommand)]
    pub subcommand: Option<TeamSubcommand>,
    #[clap(flatten)]
    pub args: TeamsArgs,
}

impl TeamCommand {
    pub async fn execute(&self) -> Result<()> {
        if self.args.list {
            let teams: Value = request(Method::GET, "/teams", None, true).await?;
            match self.args.format {
                OutputFormat::Plain => {
                    if let Some(teams) = teams.as_array() {
                        if teams.is_empty() {
                            eprintln!("You are not part of any teams.");
                        } else {
                            for team in teams {
                                println!("{}", team["name"].as_str().unwrap_or_default());
                            }
                        }
                    } else {
                        bail!("Unexpected response from server");
                    }
                },
                OutputFormat::Json => println!("{}", serde_json::to_string(&teams)?),
                OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&teams)?),
            }
            Ok(())
        } else if let Some(team) = &self.team {
            if self.args.new {
                request::<Value, _, _>(Method::POST, "/teams", Some(&json!({ "name": team })), true).await?;
                println!("Created team {}", team);
                Ok(())
            } else if self.args.delete {
                println!("Type the team name again to confirm: ");
                let confirmation = dialoguer::Input::<String>::with_theme(&dialoguer_theme()).interact()?;
                if &confirmation == team {
                    request::<Value, _, _>(Method::DELETE, &format!("/teams/{}", team), None, true).await?;
                    println!("Deleted team {}", team);
                    Ok(())
                } else {
                    bail!("Team name does not match");
                }
            } else {
                match &self.subcommand {
                    Some(subcommand) => subcommand.execute(team, &self.args.format).await,
                    None => bail!("No subcommand specified, run --help for usage"),
                }
            }
        } else {
            bail!("No team specified, run --help for usage");
        }
    }
}

#[derive(ArgEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    #[clap(hide = true)]
    Owner,
    Admin,
    Member,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Owner => f.write_str("owner"),
            Role::Admin => f.write_str("admin"),
            Role::Member => f.write_str("member"),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum TeamSubcommand {
    /// List all members on a team
    Members,
    /// Remove a member from a team
    Remove { email: String },
    /// Invite a member to a team
    Add {
        email: String,
        #[clap(long, arg_enum)]
        role: Option<Role>,
    },
    /// List pending invitations to a team
    Invitations,
    /// Revoke an invitation to a team
    Revoke { email: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    email: String,
    role: Role,
}

impl TeamSubcommand {
    pub async fn execute(&self, team: &String, format: &OutputFormat) -> Result<()> {
        match self {
            TeamSubcommand::Members => {
                let users: Vec<User> = request(Method::GET, format!("/teams/{team}/users"), None, true).await?;
                match format {
                    OutputFormat::Plain => {
                        println!("{}", "Role     Email".bold());
                        for user in users {
                            println!("{:<6}   {}", user.role, user.email);
                        }
                    },
                    OutputFormat::Json => println!("{}", serde_json::to_string(&users)?),
                    OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&users)?),
                }
                Ok(())
            },
            TeamSubcommand::Remove { email } => {
                request::<Value, _, _>(
                    Method::DELETE,
                    format!("/teams/{team}/users"),
                    Some(&json!({ "emailToRemove": email })),
                    true,
                )
                .await?;
                println!(
                    "Removed user {} from team {}",
                    email.clone().bold(),
                    team.clone().bold()
                );
                Ok(())
            },
            TeamSubcommand::Add { email, role } => {
                request::<Value, _, _>(
                    Method::POST,
                    format!("/teams/{team}/users"),
                    Some(&json!({
                        "emailToAdd": email,
                        "role": role.unwrap_or(Role::Member)
                    })),
                    true,
                )
                .await?;
                println!("Invited {} to team {}", email.clone().bold(), team.clone().bold());
                Ok(())
            },
            TeamSubcommand::Invitations => {
                let invitations: Vec<User> =
                    request(Method::GET, format!("/teams/{team}/invitations"), None, true).await?;
                match format {
                    OutputFormat::Plain => {
                        for invitation in invitations {
                            println!("{}", invitation.email);
                        }
                    },
                    OutputFormat::Json => println!("{}", serde_json::to_string(&invitations)?),
                    OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&invitations)?),
                }
                Ok(())
            },
            TeamSubcommand::Revoke { email } => {
                request::<Value, _, _>(
                    Method::DELETE,
                    format!("/teams/{team}/invitations"),
                    Some(&json!({ "emailToRevoke": email })),
                    true,
                )
                .await?;
                println!("Revoked invitation for {}", email.clone().bold());
                Ok(())
            },
        }
    }
}
