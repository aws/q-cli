// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Programming Languages supported by CodeWhisperer
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct ProgrammingLanguage {
    #[allow(missing_docs)] // documentation missing in model
    pub language_name: ::std::string::String,
}
impl ProgrammingLanguage {
    #[allow(missing_docs)] // documentation missing in model
    pub fn language_name(&self) -> &str {
        use std::ops::Deref;
        self.language_name.deref()
    }
}
impl ProgrammingLanguage {
    /// Creates a new builder-style object to manufacture
    /// [`ProgrammingLanguage`](crate::types::ProgrammingLanguage).
    pub fn builder() -> crate::types::builders::ProgrammingLanguageBuilder {
        crate::types::builders::ProgrammingLanguageBuilder::default()
    }
}

/// A builder for [`ProgrammingLanguage`](crate::types::ProgrammingLanguage).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct ProgrammingLanguageBuilder {
    pub(crate) language_name: ::std::option::Option<::std::string::String>,
}
impl ProgrammingLanguageBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn language_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.language_name = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_language_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.language_name = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_language_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.language_name
    }

    /// Consumes the builder and constructs a
    /// [`ProgrammingLanguage`](crate::types::ProgrammingLanguage). This method will fail if any
    /// of the following fields are not set:
    /// - [`language_name`](crate::types::builders::ProgrammingLanguageBuilder::language_name)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::ProgrammingLanguage, ::aws_smithy_types::error::operation::BuildError>
    {
        ::std::result::Result::Ok(crate::types::ProgrammingLanguage {
            language_name: self.language_name.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "language_name",
                    "language_name was not specified but it is required when building ProgrammingLanguage",
                )
            })?,
        })
    }
}
