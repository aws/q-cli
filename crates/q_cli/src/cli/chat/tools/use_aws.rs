use std::collections::HashMap;
use std::io::Write;
use std::process::Stdio;

use bstr::ByteSlice;
use convert_case::{
    Case,
    Casing,
};
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

const READONLY_OPS: [&str; 6] = ["get", "describe", "list", "ls", "search", "batch_get"];

// TODO: we should perhaps composite this struct with an interface that we can use to mock the
// actual cli with. That will allow us to more thoroughly test it.
#[derive(Debug, Deserialize)]
pub struct UseAws {
    pub service_name: String,
    pub operation_name: String,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub region: String,
    pub profile_name: Option<String>,
    pub label: Option<String>,
}

impl UseAws {
    pub fn requires_consent(&self) -> bool {
        !READONLY_OPS.iter().any(|op| self.operation_name.starts_with(op))
    }

    pub async fn invoke(&self, _ctx: &Context, _updates: impl Write) -> Result<InvokeOutput> {
        let mut command = tokio::process::Command::new("aws");
        command.envs(std::env::vars()).arg("--region").arg(&self.region);
        if let Some(profile_name) = self.profile_name.as_deref() {
            command.arg("--profile").arg(profile_name);
        }
        command.arg(&self.service_name).arg(&self.operation_name);
        if let Some(parameters) = self.cli_parameters() {
            for (name, val) in parameters {
                command.arg(name).arg(val);
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
        )?;
        if let Some(parameters) = &self.parameters {
            queue!(updates, style::Print("Parameters: \n".to_string()))?;
            for (name, value) in parameters {
                queue!(updates, style::Print(format!("- {}: {}\n", name, value)))?;
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
        Ok(())
    }

    /// Returns the CLI arguments properly formatted as kebab case if parameters is
    /// [Option::Some], otherwise None
    fn cli_parameters(&self) -> Option<Vec<(String, String)>> {
        if let Some(parameters) = &self.parameters {
            let mut params = vec![];
            for (param_name, val) in parameters {
                let param_name = format!("--{}", param_name.trim_start_matches("--").to_case(Case::Kebab));
                let param_val = val.as_str().map(|s| s.to_string()).unwrap_or(val.to_string());
                params.push((param_name, param_val));
            }
            Some(params)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! use_aws {
        ($value:tt) => {
            serde_json::from_value::<UseAws>(serde_json::json!($value)).unwrap()
        };
    }

    #[test]
    fn test_requires_consent() {
        let cmd = use_aws! {{
            "service_name": "ecs",
            "operation_name": "list-task-definitions",
            "region": "us-west-2",
            "profile_name": "default",
            "label": ""
        }};
        assert!(!cmd.requires_consent());
        let cmd = use_aws! {{
            "service_name": "lambda",
            "operation_name": "list-functions",
            "region": "us-west-2",
            "profile_name": "default",
            "label": ""
        }};
        assert!(!cmd.requires_consent());
        let cmd = use_aws! {{
            "service_name": "s3",
            "operation_name": "put-object",
            "region": "us-west-2",
            "profile_name": "default",
            "label": ""
        }};
        assert!(cmd.requires_consent());
    }

    #[test]
    fn test_use_aws_deser() {
        let cmd = use_aws! {{
            "service_name": "s3",
            "operation_name": "put-object",
            "parameters": {
                "TableName": "table-name",
                "KeyConditionExpression": "PartitionKey = :pkValue"
            },
            "region": "us-west-2",
            "profile_name": "default",
            "label": ""
        }};
        let params = cmd.cli_parameters().unwrap();
        assert!(
            params.iter().any(|p| p.0 == "--table-name" && p.1 == "table-name"),
            "not found in {:?}",
            params
        );
        assert!(
            params
                .iter()
                .any(|p| p.0 == "--key-condition-expression" && p.1 == "PartitionKey = :pkValue"),
            "not found in {:?}",
            params
        );
    }

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
