pub mod execute_bash;
pub mod fs_read;
pub mod fs_write;
pub mod use_aws;

use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

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
    /// The display name of a tool
    pub fn display_name(&self) -> &'static str {
        match self {
            Tool::FsRead(_) => "Read from filesystem",
            Tool::FsWrite(_) => "Write to filesystem",
            Tool::ExecuteBash(_) => "Execute shell command",
            Tool::UseAws(_) => "Read AWS resources",
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
    pub fn queue_description(&self, ctx: &Context, updates: &mut impl Write) -> Result<()> {
        match self {
            Tool::FsRead(fs_read) => fs_read.queue_description(updates),
            Tool::FsWrite(fs_write) => fs_write.queue_description(ctx, updates),
            Tool::ExecuteBash(execute_bash) => execute_bash.queue_description(updates),
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

impl TryFrom<ToolUse> for Tool {
    type Error = ToolResult;

    fn try_from(value: ToolUse) -> std::result::Result<Self, Self::Error> {
        let map_err = |parse_error| ToolResult {
            tool_use_id: value.id.clone(),
            content: vec![ToolResultContentBlock::Text(format!(
                "Failed to validate tool parameters: {parse_error}. The model has either suggested tool parameters which are incompatible with the existing tools, or has suggested one or more tool that does not exist in the list of known tools."
            ))],
            status: ToolResultStatus::Error,
        };

        Ok(match value.name.as_str() {
            "fs_read" => Self::FsRead(serde_json::from_value::<FsRead>(value.args).map_err(map_err)?),
            "fs_write" => Self::FsWrite(serde_json::from_value::<FsWrite>(value.args).map_err(map_err)?),
            "execute_bash" => Self::ExecuteBash(serde_json::from_value::<ExecuteBash>(value.args).map_err(map_err)?),
            "use_aws" => Self::UseAws(serde_json::from_value::<UseAws>(value.args).map_err(map_err)?),
            unknown => {
                return Err(ToolResult {
                    tool_use_id: value.id,
                    content: vec![ToolResultContentBlock::Text(format!(
                        "The tool, \"{unknown}\" is not supported by the client"
                    ))],
                    status: ToolResultStatus::Error,
                });
            },
        })
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

/// Converts `path` to a relative path according to the current working directory `cwd`.
fn absolute_to_relative(cwd: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<PathBuf> {
    let cwd = cwd.as_ref().canonicalize()?;
    let path = path.as_ref().canonicalize()?;
    let mut cwd_parts = cwd.components().peekable();
    let mut path_parts = path.components().peekable();

    // Skip common prefix
    while let (Some(a), Some(b)) = (cwd_parts.peek(), path_parts.peek()) {
        if a == b {
            cwd_parts.next();
            path_parts.next();
        }
    }

    // ".." for any uncommon parts, then just append the rest of the path.
    let mut relative = PathBuf::new();
    for _ in cwd_parts {
        relative.push("..");
    }
    for part in path_parts {
        relative.push(part);
    }

    Ok(relative)
}
