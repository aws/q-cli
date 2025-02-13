// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq)]
pub enum ToolResultContentBlock {
    /// A tool result that is JSON format data.
    Json(::aws_smithy_types::Document),
    /// A tool result that is text.
    Text(::std::string::String),
    /// The `Unknown` variant represents cases where new union variant was received. Consider
    /// upgrading the SDK to the latest available version. An unknown enum variant
    ///
    /// _Note: If you encounter this error, consider upgrading your SDK to the latest version._
    /// The `Unknown` variant represents cases where the server sent a value that wasn't recognized
    /// by the client. This can happen when the server adds new functionality, but the client has
    /// not been updated. To investigate this, consider turning on debug logging to print the
    /// raw HTTP response.
    #[non_exhaustive]
    Unknown,
}
impl ToolResultContentBlock {
    /// Tries to convert the enum instance into
    /// [`Json`](crate::types::ToolResultContentBlock::Json), extracting the inner
    /// [`Document`](::aws_smithy_types::Document). Returns `Err(&Self)` if it can't be
    /// converted.
    pub fn as_json(&self) -> ::std::result::Result<&::aws_smithy_types::Document, &Self> {
        if let ToolResultContentBlock::Json(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }

    /// Returns true if this is a [`Json`](crate::types::ToolResultContentBlock::Json).
    pub fn is_json(&self) -> bool {
        self.as_json().is_ok()
    }

    /// Tries to convert the enum instance into
    /// [`Text`](crate::types::ToolResultContentBlock::Text), extracting the inner
    /// [`String`](::std::string::String). Returns `Err(&Self)` if it can't be converted.
    pub fn as_text(&self) -> ::std::result::Result<&::std::string::String, &Self> {
        if let ToolResultContentBlock::Text(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }

    /// Returns true if this is a [`Text`](crate::types::ToolResultContentBlock::Text).
    pub fn is_text(&self) -> bool {
        self.as_text().is_ok()
    }

    /// Returns true if the enum instance is the `Unknown` variant.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}
impl ::std::fmt::Debug for ToolResultContentBlock {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            ToolResultContentBlock::Json(_) => f.debug_tuple("*** Sensitive Data Redacted ***").finish(),
            ToolResultContentBlock::Text(_) => f.debug_tuple("*** Sensitive Data Redacted ***").finish(),
            ToolResultContentBlock::Unknown => f.debug_tuple("Unknown").finish(),
        }
    }
}
