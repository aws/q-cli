// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub enum IdentitySource {
    #[allow(missing_docs)] // documentation missing in model
    SsoIdentitySource(crate::types::SsoIdentitySource),
    /// The `Unknown` variant represents cases where new union variant was received. Consider upgrading the SDK to the latest available version.
    /// An unknown enum variant
    ///
    /// _Note: If you encounter this error, consider upgrading your SDK to the latest version._
    /// The `Unknown` variant represents cases where the server sent a value that wasn't recognized
    /// by the client. This can happen when the server adds new functionality, but the client has not been updated.
    /// To investigate this, consider turning on debug logging to print the raw HTTP response.
    #[non_exhaustive]
    Unknown,
}
impl IdentitySource {
    #[allow(irrefutable_let_patterns)]
    /// Tries to convert the enum instance into [`SsoIdentitySource`](crate::types::IdentitySource::SsoIdentitySource), extracting the inner [`SsoIdentitySource`](crate::types::SsoIdentitySource).
    /// Returns `Err(&Self)` if it can't be converted.
    pub fn as_sso_identity_source(&self) -> ::std::result::Result<&crate::types::SsoIdentitySource, &Self> {
        if let IdentitySource::SsoIdentitySource(val) = &self {
            ::std::result::Result::Ok(val)
        } else {
            ::std::result::Result::Err(self)
        }
    }
    /// Returns true if this is a [`SsoIdentitySource`](crate::types::IdentitySource::SsoIdentitySource).
    pub fn is_sso_identity_source(&self) -> bool {
        self.as_sso_identity_source().is_ok()
    }
    /// Returns true if the enum instance is the `Unknown` variant.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}
