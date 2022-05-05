use anyhow::Result;
use clap::{ArgEnum, Args, Subcommand};
use crossterm::style::Stylize;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::util::api::request;

use super::OutputFormat;

/*
# List members on a team
- fig team <team-name> members

# Remove user from a team
- fig team <team-name> remove <email>

# Add user to a team and optionally assign a role
fig team <team-name> add <email> [--role=admin|member]

# List all teams that the user is part of
fig teams

# Delete an existing team
fig teams delete <team>

# Create a new team
fig teams create <team>
*/

#[derive(Debug, Subcommand)]
pub enum TeamsSubcommand {
    /// Create a new team
    Create { team: String },
    /// Delete an existing team
    #[clap(hide = true)]
    Delete { team: String },
    /// List all teams that the user is part of
    List {
        #[clap(short, long, arg_enum, default_value_t)]
        format: OutputFormat,
    },
}

impl TeamsSubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            TeamsSubcommand::Create { team } => {
                let _val: Value =
                    request(Method::POST, "/teams", Some(&json!({ "name": team })), true).await?;
                println!("Created team {}", team);
                Ok(())
            }
            TeamsSubcommand::Delete { team: _ } => {
                todo!();
            }
            TeamsSubcommand::List { format } => {
                let teams: Value = request(Method::GET, "/teams", None, true).await?;
                match format {
                    OutputFormat::Plain => {
                        if let Some(teams) = teams.as_array() {
                            for team in teams {
                                println!(
                                    "{}",
                                    team["name"].as_str().unwrap_or_default(),
                                );
                            }
                        }
                    }
                    OutputFormat::Json => println!("{}", serde_json::to_string(&teams)?),
                    OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&teams)?),
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Args)]
pub struct TeamCommand {
    pub team: String,
    #[clap(subcommand)]
    pub subcommand: TeamSubcommand,
}

impl TeamCommand {
    pub async fn execute(&self) -> Result<()> {
        self.subcommand.execute(&self.team).await
    }
}

#[derive(ArgEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[clap(rename_all = "kebab-case")]
pub enum Role {
    Admin,
    Member,
}

#[derive(Debug, Subcommand)]
pub enum TeamSubcommand {
    /// List all members on a team
    Members {
        #[clap(long, short, arg_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Remove a member from a team
    Remove { email: String },
    /// Invite a member to a team
    Add {
        email: String,
        #[clap(long, arg_enum)]
        role: Option<Role>,
    },
}

impl TeamSubcommand {
    pub async fn execute(&self, team: &String) -> Result<()> {
        match self {
            TeamSubcommand::Members { format } => {
                let val: Value =
                    request(Method::GET, format!("/teams/{team}/users"), None, true).await?;
                match format {
                    OutputFormat::Plain => {
                        if let Some(users) = val.as_array() {
                            println!("{}", "Role     Email".bold());
                            for user in users {
                                println!(
                                    "{:<6}   {}",
                                    user["role"].as_str().unwrap_or_default(),
                                    user["email"].as_str().unwrap_or_default(),
                                );
                            }
                        }
                    }
                    OutputFormat::Json => println!("{}", serde_json::to_string(&val)?),
                    OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&val)?),
                }
                Ok(())
            }
            TeamSubcommand::Remove { email } => {
                let _val: Value = request(
                    Method::DELETE,
                    format!("/teams/{team}/users"),
                    Some(&json!({ "emailToRemove": email })),
                    true,
                )
                .await?;
                println!("Removed user {} from team {}", email, team);
                Ok(())
            }
            TeamSubcommand::Add { email, role } => {
                let _val: Value = request(
                    Method::POST,
                    format!("/teams/{team}/users"),
                    Some(&json!({
                        "emailToAdd": email,
                        "role": role.unwrap_or(Role::Member)
                    })),
                    true,
                )
                .await?;
                println!("Added user {} to team {}", email, team);
                Ok(())
            }
        }
    }
}

