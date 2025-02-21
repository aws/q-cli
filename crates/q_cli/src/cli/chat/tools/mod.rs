pub mod execute_bash;
pub mod fs_read;
pub mod fs_write;
pub mod use_aws;

use std::io::Write;
use std::path::Path;

use aws_smithy_types::{
    Document,
    Number as SmithyNumber,
};
use execute_bash::ExecuteBash;
use eyre::Result;
use fig_api_client::model::{
    ToolResult,
    ToolResultContentBlock,
    ToolResultStatus,
};
use fig_os_shim::Context;
use fs_read::FsRead;
use fs_write::FsWrite;
use serde::Deserialize;
use use_aws::UseAws;

use super::parser::ToolUse;

/// Represents an executable tool use.
#[derive(Debug)]
pub enum Tool {
    FsRead(FsRead),
    FsWrite(FsWrite),
    ExecuteBash(ExecuteBash),
    UseAws(UseAws),
}

impl Tool {
    pub fn from_tool_use(tool_use: ToolUse) -> Result<Self, ToolResult> {
        let map_err = |parse_error| ToolResult {
            tool_use_id: tool_use.id.clone(),
            content: vec![ToolResultContentBlock::Text(format!(
                "failed to deserialize with the following error: {parse_error}"
            ))],
            status: ToolResultStatus::Error,
        };

        Ok(match tool_use.name.as_str() {
            "fs_read" => Self::FsRead(serde_json::from_value::<FsRead>(tool_use.args).map_err(map_err)?),
            "fs_write" => Self::FsWrite(serde_json::from_value::<FsWrite>(tool_use.args).map_err(map_err)?),
            "execute_bash" => Self::ExecuteBash(serde_json::from_value::<ExecuteBash>(tool_use.args).map_err(map_err)?),
            "use_aws" => Self::UseAws(serde_json::from_value::<UseAws>(tool_use.args).map_err(map_err)?),
            unknown => {
                return Err(ToolResult {
                    tool_use_id: tool_use.id,
                    content: vec![ToolResultContentBlock::Text(format!(
                        "The tool, \"{unknown}\" is not supported by the client"
                    ))],
                    status: ToolResultStatus::Error,
                });
            },
        })
    }

    /// The display name of a tool
    pub fn display_name(&self) -> String {
        match self {
            Tool::FsRead(_) => FsRead::display_name(),
            Tool::FsWrite(_) => FsWrite::display_name(),
            Tool::ExecuteBash(_) => ExecuteBash::display_name(),
            Tool::UseAws(_) => UseAws::display_name(),
        }
    }

    /// Invokes the tool asynchronously
    pub async fn invoke(&self, context: &Context, updates: &mut impl Write) -> Result<InvokeOutput> {
        match self {
            Tool::FsRead(fs_read) => fs_read.invoke(context, updates).await,
            Tool::FsWrite(fs_write) => fs_write.invoke(context, updates).await,
            Tool::ExecuteBash(execute_bash) => execute_bash.invoke(updates).await,
            Tool::UseAws(use_aws) => use_aws.invoke(context, updates).await,
        }
    }

    /// Queues up a tool's intention in a human readable format
    pub fn show_readable_intention(&self, updates: &mut impl Write) -> Result<()> {
        match self {
            Tool::FsRead(fs_read) => fs_read.show_readable_intention(updates),
            Tool::FsWrite(fs_write) => fs_write.show_readable_intention(updates),
            Tool::ExecuteBash(execute_bash) => execute_bash.show_readable_intention(updates),
            Tool::UseAws(use_aws) => use_aws.show_readable_intention(updates),
        }
    }

    /// Validates the tool with the arguments supplied
    pub async fn validate(&mut self, ctx: &Context) -> Result<()> {
        match self {
            Tool::FsRead(fs_read) => fs_read.validate(ctx).await,
            Tool::FsWrite(fs_write) => fs_write.validate(ctx).await,
            Tool::ExecuteBash(execute_bash) => execute_bash.validate(ctx).await,
            Tool::UseAws(use_aws) => use_aws.validate(ctx).await,
        }
    }
}

/// A tool specification to be sent to the model as part of a conversation. Maps to
/// [BedrockToolSpecification].
#[derive(Debug, Clone, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: InputSchema,
}

/// The schema specification describing a tool's fields.
#[derive(Debug, Clone, Deserialize)]
pub struct InputSchema(pub serde_json::Value);

/// The output received from invoking a [Tool].
#[derive(Debug, Default)]
pub struct InvokeOutput {
    pub output: OutputKind,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum OutputKind {
    Text(String),
    Json(serde_json::Value),
}

impl Default for OutputKind {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

pub fn serde_value_to_document(value: serde_json::Value) -> Document {
    match value {
        serde_json::Value::Null => Document::Null,
        serde_json::Value::Bool(bool) => Document::Bool(bool),
        serde_json::Value::Number(number) => {
            if let Some(num) = number.as_u64() {
                Document::Number(SmithyNumber::PosInt(num))
            } else if number.as_i64().is_some_and(|n| n < 0) {
                Document::Number(SmithyNumber::NegInt(number.as_i64().unwrap()))
            } else {
                Document::Number(SmithyNumber::Float(number.as_f64().unwrap_or_default()))
            }
        },
        serde_json::Value::String(string) => Document::String(string),
        serde_json::Value::Array(vec) => {
            Document::Array(vec.clone().into_iter().map(serde_value_to_document).collect::<_>())
        },
        serde_json::Value::Object(map) => Document::Object(
            map.into_iter()
                .map(|(k, v)| (k, serde_value_to_document(v)))
                .collect::<_>(),
        ),
    }
}

/// Returns a display-friendly [String] of `path` relative to `cwd`, returning `path` if either
/// `cwd` or `path` is invalid UTF-8, or `path` is not prefixed by `cwd`.
fn relative_path(cwd: impl AsRef<Path>, path: impl AsRef<Path>) -> String {
    match (cwd.as_ref().to_str(), path.as_ref().to_str()) {
        (Some(cwd), Some(path)) => path.strip_prefix(cwd).unwrap_or_default().to_string(),
        _ => path.as_ref().to_string_lossy().to_string(),
    }
}
