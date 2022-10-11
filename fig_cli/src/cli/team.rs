use clap::{
    Args,
    Subcommand,
    ValueEnum,
};
use crossterm::style::Stylize;
use eyre::{
    bail,
    Result,
};
use fig_request::Request;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;

use super::OutputFormat;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct TeamsArgs {
    // List all teams that the user is part of
    #[arg(long, conflicts_with_all = &["new", "delete"])]
    list: bool,
    // Create a new team
    #[arg(long, conflicts_with_all = &["list", "delete"])]
    new: bool,
    // Delete an existing team
    #[arg(long, conflicts_with_all = &["list", "new"])]
    delete: bool,
    // Format of output
    #[arg(long, short, value_enum, default_value_t)]
    format: OutputFormat,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct TeamCommand {
    pub team: Option<String>,
    #[command(subcommand)]
    pub subcommand: Option<TeamSubcommand>,
    #[command(flatten)]
    pub args: TeamsArgs,
}

impl TeamCommand {
    pub async fn execute(&self) -> Result<()> {
        if self.args.list {
            let teams = Request::get("/teams").auth().json().await?;
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
                Request::post("/teams")
                    .auth()
                    .body(json!({ "name": team }))
                    .send()
                    .await?;
                println!("Created team {team}");
                Ok(())
            } else if self.args.delete {
                println!("Type the team name again to confirm: ");
                let confirmation =
                    dialoguer::Input::<String>::with_theme(&crate::util::dialoguer_theme()).interact()?;
                if &confirmation == team {
                    Request::delete(format!("/teams/{team}")).auth().send().await?;
                    println!("Deleted team {team}");
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

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    #[value(hide = true)]
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

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum TeamSubcommand {
    /// List all members on a team
    Members,
    /// Remove a member from a team
    Remove { email: String },
    /// Invite a member to a team
    Add {
        email: String,
        #[arg(long, value_enum)]
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
                let users: Vec<User> = Request::get(format!("/teams/{team}/users")).auth().deser_json().await?;
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
                Request::delete(format!("/teams/{team}/users"))
                    .body(json!({ "emailToRemove": email }))
                    .auth()
                    .send()
                    .await?;
                println!(
                    "Removed user {} from team {}",
                    email.clone().bold(),
                    team.clone().bold()
                );
                Ok(())
            },
            TeamSubcommand::Add { email, role } => {
                Request::post(format!("/teams/{team}/users"))
                    .body(json!({
                        "emailToAdd": email,
                        "role": role.unwrap_or(Role::Member)
                    }))
                    .auth()
                    .send()
                    .await?;
                println!("Invited {} to team {}", email.clone().bold(), team.clone().bold());
                Ok(())
            },
            TeamSubcommand::Invitations => {
                let invitations: Vec<User> = Request::get(format!("/teams/{team}/invitations"))
                    .auth()
                    .deser_json()
                    .await?;
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
                Request::delete(format!("/teams/{team}/invitations"))
                    .body(json!({ "emailToRevoke": email }))
                    .auth()
                    .send()
                    .await?;
                println!("Revoked invitation for {}", email.clone().bold());
                Ok(())
            },
        }
    }
}
