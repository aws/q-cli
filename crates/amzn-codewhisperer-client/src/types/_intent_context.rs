// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub enum IntentContext {
    #[allow(missing_docs)] // documentation missing in model
    Documentation(crate::types::DocumentationIntentContext),
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
impl IntentContext {
    #[allow(irrefutable_let_patterns)]
    /// Tries to convert the enum instance into
    /// [`Documentation`](crate::types::IntentContext::Documentation), extracting the inner
    /// [`DocumentationIntentContext`](crate::types::DocumentationIntentContext). Returns `Err(&
    /// Self)` if it can't be converted.
    pub fn as_documentation(&self) -> ::std::result::Result<&crate::types::DocumentationIntentContext, &Self> {
        if let IntentContext::Documentation(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }

    /// Returns true if this is a [`Documentation`](crate::types::IntentContext::Documentation).
    pub fn is_documentation(&self) -> bool {
        self.as_documentation().is_ok()
    }

    /// Returns true if the enum instance is the `Unknown` variant.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}
