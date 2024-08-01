// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct DocumentSymbol {
    /// Name of the Document Symbol
    pub name: ::std::string::String,
    /// Symbol type - DECLARATION / USAGE
    pub r#type: crate::types::SymbolType,
    /// Symbol package / source for FullyQualified names
    pub source: ::std::option::Option<::std::string::String>,
}
impl DocumentSymbol {
    /// Name of the Document Symbol
    pub fn name(&self) -> &str {
        use std::ops::Deref;
        self.name.deref()
    }

    /// Symbol type - DECLARATION / USAGE
    pub fn r#type(&self) -> &crate::types::SymbolType {
        &self.r#type
    }

    /// Symbol package / source for FullyQualified names
    pub fn source(&self) -> ::std::option::Option<&str> {
        self.source.as_deref()
    }
}
impl DocumentSymbol {
    /// Creates a new builder-style object to manufacture
    /// [`DocumentSymbol`](crate::types::DocumentSymbol).
    pub fn builder() -> crate::types::builders::DocumentSymbolBuilder {
        crate::types::builders::DocumentSymbolBuilder::default()
    }
}

/// A builder for [`DocumentSymbol`](crate::types::DocumentSymbol).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct DocumentSymbolBuilder {
    pub(crate) name: ::std::option::Option<::std::string::String>,
    pub(crate) r#type: ::std::option::Option<crate::types::SymbolType>,
    pub(crate) source: ::std::option::Option<::std::string::String>,
}
impl DocumentSymbolBuilder {
    /// Name of the Document Symbol
    /// This field is required.
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.name = ::std::option::Option::Some(input.into());
        self
    }

    /// Name of the Document Symbol
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.name = input;
        self
    }

    /// Name of the Document Symbol
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.name
    }

    /// Symbol type - DECLARATION / USAGE
    /// This field is required.
    pub fn r#type(mut self, input: crate::types::SymbolType) -> Self {
        self.r#type = ::std::option::Option::Some(input);
        self
    }

    /// Symbol type - DECLARATION / USAGE
    pub fn set_type(mut self, input: ::std::option::Option<crate::types::SymbolType>) -> Self {
        self.r#type = input;
        self
    }

    /// Symbol type - DECLARATION / USAGE
    pub fn get_type(&self) -> &::std::option::Option<crate::types::SymbolType> {
        &self.r#type
    }

    /// Symbol package / source for FullyQualified names
    pub fn source(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.source = ::std::option::Option::Some(input.into());
        self
    }

    /// Symbol package / source for FullyQualified names
    pub fn set_source(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.source = input;
        self
    }

    /// Symbol package / source for FullyQualified names
    pub fn get_source(&self) -> &::std::option::Option<::std::string::String> {
        &self.source
    }

    /// Consumes the builder and constructs a [`DocumentSymbol`](crate::types::DocumentSymbol).
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](crate::types::builders::DocumentSymbolBuilder::name)
    /// - [`r#type`](crate::types::builders::DocumentSymbolBuilder::r#type)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::DocumentSymbol, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::DocumentSymbol {
            name: self.name.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "name",
                    "name was not specified but it is required when building DocumentSymbol",
                )
            })?,
            r#type: self.r#type.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "r#type",
                    "r#type was not specified but it is required when building DocumentSymbol",
                )
            })?,
            source: self.source,
        })
    }
}
