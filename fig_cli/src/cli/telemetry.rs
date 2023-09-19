use clap::Subcommand;
use eyre::Result;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum TelemetrySubcommand {
    Enable,
    Disable,
    Status,
}

impl TelemetrySubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            TelemetrySubcommand::Enable => {
                if fig_request::auth::is_logged_in() {
                    fig_api_client::settings::update("telemetry.disabled", json!(false)).await?;
                } else {
                    fig_settings::settings::set_value("telemetry.disabled", json!(false))?;
                }
                Ok(())
            },
            TelemetrySubcommand::Disable => {
                if fig_request::auth::is_logged_in() {
                    fig_api_client::settings::update("telemetry.disabled", json!(true)).await?;
                } else {
                    fig_settings::settings::set_value("telemetry.disabled", json!(true))?;
                }
                Ok(())
            },
            TelemetrySubcommand::Status => {
                let status = fig_settings::settings::get_bool_or("telemetry.disabled", false);
                println!("Telemetry is {}", if status { "disabled" } else { "enabled" });
                Ok(())
            },
        }
    }
}
