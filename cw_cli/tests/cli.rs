use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

// Integrations tests for the CLI
// This should be used to test interfaces that external code may rely on (exit codes, structured
// output, CLI flags) List of external codebases that are tightly coupled to `cw_cli`. If you need
// to modify these tests, make sure that you audit external codebases (fig completion spec,
// figterm, shell integrations) as well.

#[test]
fn version_flag_has_status_code_zero() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cw_cli")?;

    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));

    Ok(())
}
