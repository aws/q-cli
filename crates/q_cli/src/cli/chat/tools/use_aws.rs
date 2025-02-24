use std::collections::HashMap;
use std::io::Write;
use std::process::Stdio;

use bstr::ByteSlice;
use crossterm::{
    queue,
    style,
};
use eyre::{
    Result,
    WrapErr,
};
use fig_os_shim::Context;
use serde::Deserialize;

use super::{
    InvokeOutput,
    OutputKind,
};

const ALLOWED_OPS: [&str; 6] = ["get", "describe", "list", "ls", "search", "batch_get"];

#[derive(Debug, thiserror::Error)]
enum AwsToolError {
    ForbiddenOperation(String),
}

impl std::fmt::Display for AwsToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AwsToolError::ForbiddenOperation(op) => Ok(writeln!(f, "Forbidden operation encountered: {}", op)?),
        }
    }
}

// TODO: we should perhaps composite this struct with an interface that we can use to mock the
// actual cli with. That will allow us to more thoroughly test it.
#[derive(Debug, Deserialize)]
pub struct UseAws {
    pub service_name: String,
    pub operation_name: String,
    pub parameters: Option<HashMap<String, String>>,
    pub region: String,
    pub profile_name: Option<String>,
    pub label: Option<String>,
}

impl UseAws {
    fn validate_operation(&self) -> Result<(), AwsToolError> {
        let operation_name = &self.operation_name;
        for op in ALLOWED_OPS {
            if self.operation_name.starts_with(op) {
                return Ok(());
            }
        }
        Err(AwsToolError::ForbiddenOperation(operation_name.clone()))
    }

    pub async fn invoke(&self, _ctx: &Context, _updates: impl Write) -> Result<InvokeOutput> {
        let mut command = tokio::process::Command::new("aws");
        command.envs(std::env::vars()).arg("--region").arg(&self.region);
        if let Some(profile_name) = self.profile_name.as_deref() {
            command.arg("--profile").arg(profile_name);
        }
        command.arg(&self.service_name).arg(&self.operation_name);
        if let Some(parameters) = &self.parameters {
            for (param_name, val) in parameters {
                if param_name.starts_with("--") {
                    command.arg(param_name).arg(val);
                } else {
                    command.arg(format!("--{}", param_name)).arg(val);
                }
            }
        }
        let output = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .wrap_err_with(|| format!("Unable to spawn command '{:?}'", self))?
            .wait_with_output()
            .await
            .wrap_err_with(|| format!("Unable to spawn command '{:?}'", self))?;
        let status = output.status.code().unwrap_or(0).to_string();
        let stdout = output.stdout.to_str_lossy();
        let stderr = output.stderr.to_str_lossy();

        if status.eq("0") {
            Ok(InvokeOutput {
                output: OutputKind::Json(serde_json::json!({
                    "exit_status": status,
                    "stdout": stdout,
                    "stderr": stderr
                })),
            })
        } else {
            Err(eyre::eyre!(stderr.to_string()))
        }
    }

    pub fn queue_description(&self, updates: &mut impl Write) -> Result<()> {
        queue!(
            updates,
            style::Print("Running aws cli command:\n"),
            style::Print(format!("Service name: {}\n", self.service_name)),
            style::Print(format!("Operation name: {}\n", self.operation_name)),
            style::Print("Parameters: \n".to_string()),
        )?;
        if let Some(parameters) = &self.parameters {
            for (name, value) in parameters {
                queue!(updates, style::Print(format!("{}: {}\n", name, value)))?;
            }
        }

        if let Some(ref profile_name) = self.profile_name {
            queue!(updates, style::Print(format!("Profile name: {}\n", profile_name)))?;
        } else {
            queue!(updates, style::Print("Profile name: default\n".to_string()))?;
        }

        queue!(updates, style::Print(format!("Region: {}", self.region)))?;

        if let Some(ref label) = self.label {
            queue!(updates, style::Print(format!("\nLabel: {}", label)))?;
        }
        Ok(())
    }

    pub async fn validate(&mut self, _ctx: &Context) -> Result<()> {
        self.validate_operation()
            .wrap_err_with(|| format!("Unable to spawn command '{:?}'", &self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "not in ci"]
    async fn test_aws_read_only() {
        let ctx = Context::new_fake();

        let v = serde_json::json!({
            "service_name": "s3",
            "operation_name": "put-object",
            // technically this wouldn't be a valid request with an empty parameter set but it's
            // okay for this test
            "parameters": {},
            "region": "us-west-2",
            "profile_name": "default",
            "label": ""
        });

        assert!(
            serde_json::from_value::<UseAws>(v)
                .unwrap()
                .invoke(&ctx, &mut std::io::stdout())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    #[ignore = "not in ci"]
    async fn test_aws_output() {
        let ctx = Context::new_fake();

        let v = serde_json::json!({
            "service_name": "s3",
            "operation_name": "ls",
            "parameters": {},
            "region": "us-west-2",
            "profile_name": "default",
            "label": ""
        });
        let out = serde_json::from_value::<UseAws>(v)
            .unwrap()
            .invoke(&ctx, &mut std::io::stdout())
            .await
            .unwrap();

        if let OutputKind::Json(json) = out.output {
            // depending on where the test is ran we might get different outcome here but it does
            // not mean the tool is not working
            let exit_status = json.get("exit_status").unwrap();
            if exit_status == 0 {
                assert_eq!(json.get("stderr").unwrap(), "");
            } else {
                assert_ne!(json.get("stderr").unwrap(), "");
            }
        } else {
            panic!("Expected JSON output");
        }
    }
}
