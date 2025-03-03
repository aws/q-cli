use std::io::Write;
use std::process::Stdio;

use bstr::ByteSlice;
use crossterm::style::{
    self,
    Color,
};
use crossterm::{
    execute,
    queue,
};
use eyre::{
    Context as EyreContext,
    Result,
};
use fig_os_shim::Context;
use serde::Deserialize;

use super::{
    InvokeOutput,
    MAX_TOOL_RESPONSE_SIZE,
    OutputKind,
};

#[derive(Debug, Clone, Deserialize)]
pub struct ExecuteBash {
    pub command: String,
    pub interactive: Option<bool>,
}

impl ExecuteBash {
    pub async fn invoke(&self, mut updates: impl Write) -> Result<InvokeOutput> {
        queue!(
            updates,
            style::SetForegroundColor(Color::Green),
            style::Print(format!("Executing `{}`", &self.command)),
            style::ResetColor,
            style::Print("\n"),
        )?;

        let (stdout, stderr) = match self.interactive {
            Some(true) => (Stdio::inherit(), Stdio::inherit()),
            _ => (Stdio::piped(), Stdio::piped()),
        };

        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(&self.command)
            .stdin(Stdio::inherit())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .wrap_err_with(|| format!("Unable to spawn command '{}'", &self.command))?
            .wait_with_output()
            .await
            .wrap_err_with(|| format!("Unable to wait on subprocess for command '{}'", &self.command))?;
        let status = output.status.code().unwrap_or(0).to_string();
        let stdout = output.stdout.to_str_lossy();
        let stderr = output.stderr.to_str_lossy();

        if let Some(false) = self.interactive {
            execute!(updates, style::Print(&stdout))?;
        }

        let stdout = format!(
            "{}{}",
            &stdout[0..stdout.len().min(MAX_TOOL_RESPONSE_SIZE / 3)],
            if stdout.len() > MAX_TOOL_RESPONSE_SIZE / 3 {
                " ... truncated"
            } else {
                ""
            }
        );

        let stderr = format!(
            "{}{}",
            &stderr[0..stderr.len().min(MAX_TOOL_RESPONSE_SIZE / 3)],
            if stderr.len() > MAX_TOOL_RESPONSE_SIZE / 3 {
                " ... truncated"
            } else {
                ""
            }
        );

        let output = serde_json::json!({
            "exit_status": status,
            "stdout": stdout,
            "stderr": stderr,
        });

        Ok(InvokeOutput {
            output: OutputKind::Json(output),
        })
    }

    pub fn queue_description(&self, updates: &mut impl Write) -> Result<()> {
        queue!(updates, style::Print("I will run the following shell command: "),)?;

        // TODO: Could use graphemes for a better heuristic
        if self.command.len() > 20 {
            queue!(updates, style::Print("\n"),)?;
        }

        Ok(queue!(
            updates,
            style::SetForegroundColor(Color::Green),
            style::Print(&self.command),
            style::ResetColor
        )?)
    }

    pub async fn validate(&mut self, _ctx: &Context) -> Result<()> {
        // TODO: probably some small amount of PATH checking
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
            "command": "echo Hello, world!",
            "interactive": false
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
            "command": "echo Hello, world! 1>&2",
            "interactive": false
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
            "command": "exit 1",
            "interactive": false
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
