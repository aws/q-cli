// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct GetCustomizationInput {
    #[allow(missing_docs)] // documentation missing in model
    pub identifier: ::std::option::Option<::std::string::String>,
}
impl GetCustomizationInput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn identifier(&self) -> ::std::option::Option<&str> {
        self.identifier.as_deref()
    }
}
impl GetCustomizationInput {
    /// Creates a new builder-style object to manufacture
    /// [`GetCustomizationInput`](crate::operation::get_customization::GetCustomizationInput).
    pub fn builder() -> crate::operation::get_customization::builders::GetCustomizationInputBuilder {
        crate::operation::get_customization::builders::GetCustomizationInputBuilder::default()
    }
}

/// A builder for
/// [`GetCustomizationInput`](crate::operation::get_customization::GetCustomizationInput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct GetCustomizationInputBuilder {
    pub(crate) identifier: ::std::option::Option<::std::string::String>,
}
impl GetCustomizationInputBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn identifier(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.identifier = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_identifier(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.identifier = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_identifier(&self) -> &::std::option::Option<::std::string::String> {
        &self.identifier
    }

    /// Consumes the builder and constructs a
    /// [`GetCustomizationInput`](crate::operation::get_customization::GetCustomizationInput).
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::get_customization::GetCustomizationInput,
        ::aws_smithy_http::operation::error::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::get_customization::GetCustomizationInput {
            identifier: self.identifier,
        })
    }
}
