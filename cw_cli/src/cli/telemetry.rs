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
                fig_settings::settings::set_value("telemetry.enabled", json!(true))?;
                Ok(())
            },
            TelemetrySubcommand::Disable => {
                fig_settings::settings::set_value("telemetry.enabled", json!(false))?;
                Ok(())
            },
            TelemetrySubcommand::Status => {
                let status = fig_settings::settings::get_bool_or("telemetry.enabled", true);
                println!("Telemetry is {}", if status { "disabled" } else { "enabled" });
                Ok(())
            },
        }
    }
}
