use std::process::Command;

use assert_cmd::prelude::*;
use fig_util::CLI_CRATE_NAME;
use predicates::prelude::*;

// Integrations tests for the CLI
// This should be used to test interfaces that external code may rely on (exit codes, structured
// output, CLI flags) List of external codebases that are tightly coupled to `q_cli`. If you need
// to modify these tests, make sure that you audit external codebases (fig completion spec,
// figterm, shell integrations) as well.

#[test]
fn version_flag_has_status_code_zero() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin(CLI_CRATE_NAME)?
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
    Ok(())
}

#[test]
fn help_flag_has_status_code_zero() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin(CLI_CRATE_NAME)?.arg("--help").assert().success();
    Ok(())
}

#[test]
fn help_all_flag_has_status_code_zero() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin(CLI_CRATE_NAME)?.arg("--help-all").assert().success();
    Ok(())
}
