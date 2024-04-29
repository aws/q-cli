use std::process::Command;

use assert_cmd::prelude::*;
use fig_util::CLI_CRATE_NAME;
use predicates::prelude::*;

// Integrations tests for the CLI
//
// This should be used to test interfaces that external code may rely on
// (exit codes, structured output, CLI flags)

fn cli() -> Command {
    Command::cargo_bin(CLI_CRATE_NAME).unwrap()
}

#[test]
fn version_flag_has_status_code_zero() -> Result<(), Box<dyn std::error::Error>> {
    cli()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
    Ok(())
}

#[test]
fn help_flag_has_status_code_zero() -> Result<(), Box<dyn std::error::Error>> {
    cli().arg("--help").assert().success();
    Ok(())
}

#[test]
fn help_all_flag_has_status_code_zero() -> Result<(), Box<dyn std::error::Error>> {
    cli().arg("--help-all").assert().success();
    Ok(())
}

#[test]
fn should_figterm_launch_code_success() -> Result<(), Box<dyn std::error::Error>> {
    cli()
        .args(["_", "should-figterm-launch"])
        .env("Q_FORCE_FIGTERM_LAUNCH", "1")
        .assert()
        .success();
    Ok(())
}

#[test]
fn should_figterm_launch_code_failure() -> Result<(), Box<dyn std::error::Error>> {
    cli().args(["_", "should-figterm-launch"]).assert().failure();
    Ok(())
}
