// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub enum IntentDataType {
    #[allow(missing_docs)] // documentation missing in model
    String(::std::string::String),
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
impl IntentDataType {
    #[allow(irrefutable_let_patterns)]
    /// Tries to convert the enum instance into [`String`](crate::types::IntentDataType::String),
    /// extracting the inner [`String`](::std::string::String). Returns `Err(&Self)` if it can't
    /// be converted.
    pub fn as_string(&self) -> ::std::result::Result<&::std::string::String, &Self> {
        if let IntentDataType::String(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }

    /// Returns true if this is a [`String`](crate::types::IntentDataType::String).
    pub fn is_string(&self) -> bool {
        self.as_string().is_ok()
    }

    /// Returns true if the enum instance is the `Unknown` variant.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}
