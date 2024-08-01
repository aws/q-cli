// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ExtensionProviderMetadata {
    #[allow(missing_docs)] // documentation missing in model
    pub extension_provider: ::std::string::String,
    #[allow(missing_docs)] // documentation missing in model
    pub description: ::std::option::Option<::std::string::String>,
}
impl ExtensionProviderMetadata {
    #[allow(missing_docs)] // documentation missing in model
    pub fn extension_provider(&self) -> &str {
        use std::ops::Deref;
        self.extension_provider.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn description(&self) -> ::std::option::Option<&str> {
        self.description.as_deref()
    }
}
impl ExtensionProviderMetadata {
    /// Creates a new builder-style object to manufacture
    /// [`ExtensionProviderMetadata`](crate::types::ExtensionProviderMetadata).
    pub fn builder() -> crate::types::builders::ExtensionProviderMetadataBuilder {
        crate::types::builders::ExtensionProviderMetadataBuilder::default()
    }
}

/// A builder for [`ExtensionProviderMetadata`](crate::types::ExtensionProviderMetadata).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct ExtensionProviderMetadataBuilder {
    pub(crate) extension_provider: ::std::option::Option<::std::string::String>,
    pub(crate) description: ::std::option::Option<::std::string::String>,
}
impl ExtensionProviderMetadataBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn extension_provider(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.extension_provider = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_extension_provider(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.extension_provider = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_extension_provider(&self) -> &::std::option::Option<::std::string::String> {
        &self.extension_provider
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn description(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.description = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_description(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.description = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_description(&self) -> &::std::option::Option<::std::string::String> {
        &self.description
    }

    /// Consumes the builder and constructs a
    /// [`ExtensionProviderMetadata`](crate::types::ExtensionProviderMetadata). This method will
    /// fail if any of the following fields are not set:
    /// - [`extension_provider`](crate::types::builders::ExtensionProviderMetadataBuilder::extension_provider)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::ExtensionProviderMetadata, ::aws_smithy_types::error::operation::BuildError>
    {
        ::std::result::Result::Ok(crate::types::ExtensionProviderMetadata {
            extension_provider: self.extension_provider.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "extension_provider",
                    "extension_provider was not specified but it is required when building ExtensionProviderMetadata",
                )
            })?,
            description: self.description,
        })
    }
}
