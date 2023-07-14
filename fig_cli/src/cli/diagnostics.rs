use std::io::{
    stdout,
    IsTerminal,
};

use clap::Args;
use color_eyre::Result;
use crossterm::terminal::{
    Clear,
    ClearType,
};
use crossterm::{
    cursor,
    execute,
};
use eyre::ContextCompat;
use fig_diagnostic::Diagnostics;
use fig_ipc::local::send_recv_command_to_socket;
use fig_proto::local::command::Command;
use fig_proto::local::command_response::Response;
use fig_proto::local::{
    DiagnosticsCommand,
    DiagnosticsResponse,
    IntegrationAction,
    TerminalIntegrationCommand,
};
use spinners::{
    Spinner,
    Spinners,
};

use super::OutputFormat;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct DiagnosticArgs {
    /// The format of the output
    #[arg(long, short, value_enum, default_value_t)]
    format: OutputFormat,
    /// Force limited diagnostic output
    #[arg(long)]
    force: bool,
}

impl DiagnosticArgs {
    pub async fn execute(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        if !self.force && !fig_util::is_fig_desktop_running() {
            use owo_colors::OwoColorize;

            println!(
                "\n→ Fig is not running.\n  Please launch Fig with {} or run {} to get limited diagnostics.",
                "fig launch".magenta(),
                "fig diagnostic --force".magenta()
            );
            return Ok(());
        }

        let spinner = if stdout().is_terminal() {
            Some(Spinner::new(Spinners::Dots, "Generating...".into()))
        } else {
            None
        };

        if spinner.is_some() {
            execute!(std::io::stdout(), cursor::Hide)?;

            ctrlc::set_handler(move || {
                execute!(std::io::stdout(), cursor::Show).ok();
                std::process::exit(1);
            })?;
        }

        let diagnostics = Diagnostics::new();

        if let Some(mut sp) = spinner {
            sp.stop();
            execute!(std::io::stdout(), Clear(ClearType::CurrentLine), cursor::Show)?;
            println!();
        }

        match self.format {
            OutputFormat::Plain => println!("{}", diagnostics.user_readable().join("\n")),
            OutputFormat::Json => println!("{}", serde_json::to_string(&diagnostics)?),
            OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&diagnostics)?),
        }

        Ok(())
    }
}

pub async fn get_diagnostics() -> Result<DiagnosticsResponse> {
    let response = send_recv_command_to_socket(Command::Diagnostics(DiagnosticsCommand {}))
        .await?
        .context("Received EOF while reading diagnostics")?;

    match response.response {
        Some(Response::Diagnostics(diagnostics)) => Ok(diagnostics),
        _ => eyre::bail!("Invalid response"),
    }
}

pub async fn verify_integration(integration: impl Into<String>) -> Result<String> {
    let response = send_recv_command_to_socket(Command::TerminalIntegration(TerminalIntegrationCommand {
        identifier: integration.into(),
        action: IntegrationAction::VerifyInstall as i32,
    }))
    .await?
    .context("Received EOF while getting terminal integration")?;

    let message = match response.response {
        Some(Response::Success(success)) => success.message,
        Some(Response::Error(error)) => error.message,
        _ => eyre::bail!("Invalid response"),
    };

    message.context("No message found")
}
