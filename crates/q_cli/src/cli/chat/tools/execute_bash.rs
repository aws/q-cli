use std::fmt::Display;
use std::io::Write;
use std::process::Stdio;

use bstr::ByteSlice;
use crossterm::queue;
use crossterm::style::{
    self,
    Color,
};
use eyre::{
    Context as EyreContext,
    Result,
};
use fig_os_shim::Context;
use serde::Deserialize;

use super::{
    InvokeOutput,
    OutputKind,
};

#[derive(Debug, Deserialize)]
pub struct ExecuteBash {
    pub command: String,
}

impl ExecuteBash {
    pub fn display_name() -> String {
        "Execute bash command".to_owned()
    }

    pub async fn invoke(&self, mut updates: impl Write) -> Result<InvokeOutput> {
        queue!(
            updates,
            style::SetForegroundColor(Color::Green),
            style::Print(format!("Executing `{}`", &self.command)),
            style::ResetColor,
            style::Print("\n"),
        )?;

        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(&self.command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .wrap_err_with(|| format!("Unable to spawn command '{}'", &self.command))?
            .wait_with_output()
            .await
            .wrap_err_with(|| format!("Unable to wait on subprocess for command '{}'", &self.command))?;
        let status = output.status.code().unwrap_or(0).to_string();
        let stdout = output.stdout.to_str_lossy();
        let stderr = output.stderr.to_str_lossy();
        Ok(InvokeOutput {
            output: OutputKind::Json(serde_json::json!({
                "exit_status": status,
                "stdout": stdout,
                "stderr": stderr,
            })),
        })
    }

    pub fn show_readable_intention(&self, updates: &mut impl Write) -> Result<()> {
        Ok(queue!(
            updates,
            style::Print(format!("Executing bash command: {}\n", self.command))
        )?)
    }

    pub async fn validate(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}

impl Display for ExecuteBash {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        queue!(
            std::io::stdout(),
            style::Print(format!("Executing bash command: {}\n", self.command))
        )
        .map_err(|_| std::fmt::Error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_bash_tool() {
        let mut stdout = std::io::stdout();

        // Verifying stdout
        let v = serde_json::json!({
            "command": "echo Hello, world!"
        });
        let out = serde_json::from_value::<ExecuteBash>(v)
            .unwrap()
            .invoke(&mut stdout)
            .await
            .unwrap();

        if let OutputKind::Json(json) = out.output {
            assert_eq!(json.get("exit_status").unwrap(), &0.to_string());
            assert_eq!(json.get("stdout").unwrap(), "Hello, world!\n");
            assert_eq!(json.get("stderr").unwrap(), "");
        } else {
            panic!("Expected JSON output");
        }

        // Verifying stderr
        let v = serde_json::json!({
            "command": "echo Hello, world! 1>&2"
        });
        let out = serde_json::from_value::<ExecuteBash>(v)
            .unwrap()
            .invoke(&mut stdout)
            .await
            .unwrap();

        if let OutputKind::Json(json) = out.output {
            assert_eq!(json.get("exit_status").unwrap(), &0.to_string());
            assert_eq!(json.get("stdout").unwrap(), "");
            assert_eq!(json.get("stderr").unwrap(), "Hello, world!\n");
        } else {
            panic!("Expected JSON output");
        }

        // Verifying exit code
        let v = serde_json::json!({
            "command": "exit 1"
        });
        let out = serde_json::from_value::<ExecuteBash>(v)
            .unwrap()
            .invoke(&mut stdout)
            .await
            .unwrap();
        if let OutputKind::Json(json) = out.output {
            assert_eq!(json.get("exit_status").unwrap(), &1.to_string());
            assert_eq!(json.get("stdout").unwrap(), "");
            assert_eq!(json.get("stderr").unwrap(), "");
        } else {
            panic!("Expected JSON output");
        }
    }
}
