// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.

/// Code Reference / Repository details
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct Reference {
    /// License name
    pub license_name: ::std::option::Option<::std::string::String>,
    /// Code Repsitory for the associated reference
    pub repository: ::std::option::Option<::std::string::String>,
    /// Respository URL
    pub url: ::std::option::Option<::std::string::String>,
    /// Span / Range for the Reference
    pub recommendation_content_span: ::std::option::Option<crate::types::Span>,
}
impl Reference {
    /// License name
    pub fn license_name(&self) -> ::std::option::Option<&str> {
        self.license_name.as_deref()
    }
    /// Code Repsitory for the associated reference
    pub fn repository(&self) -> ::std::option::Option<&str> {
        self.repository.as_deref()
    }
    /// Respository URL
    pub fn url(&self) -> ::std::option::Option<&str> {
        self.url.as_deref()
    }
    /// Span / Range for the Reference
    pub fn recommendation_content_span(&self) -> ::std::option::Option<&crate::types::Span> {
        self.recommendation_content_span.as_ref()
    }
}
impl Reference {
    /// Creates a new builder-style object to manufacture [`Reference`](crate::types::Reference).
    pub fn builder() -> crate::types::builders::ReferenceBuilder {
        crate::types::builders::ReferenceBuilder::default()
    }
}

/// A builder for [`Reference`](crate::types::Reference).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct ReferenceBuilder {
    pub(crate) license_name: ::std::option::Option<::std::string::String>,
    pub(crate) repository: ::std::option::Option<::std::string::String>,
    pub(crate) url: ::std::option::Option<::std::string::String>,
    pub(crate) recommendation_content_span: ::std::option::Option<crate::types::Span>,
}
impl ReferenceBuilder {
    /// License name
    pub fn license_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.license_name = ::std::option::Option::Some(input.into());
        self
    }
    /// License name
    pub fn set_license_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.license_name = input;
        self
    }
    /// License name
    pub fn get_license_name(&self) -> &::std::option::Option<::std::string::String> {
        &self.license_name
    }
    /// Code Repsitory for the associated reference
    pub fn repository(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.repository = ::std::option::Option::Some(input.into());
        self
    }
    /// Code Repsitory for the associated reference
    pub fn set_repository(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.repository = input;
        self
    }
    /// Code Repsitory for the associated reference
    pub fn get_repository(&self) -> &::std::option::Option<::std::string::String> {
        &self.repository
    }
    /// Respository URL
    pub fn url(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.url = ::std::option::Option::Some(input.into());
        self
    }
    /// Respository URL
    pub fn set_url(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.url = input;
        self
    }
    /// Respository URL
    pub fn get_url(&self) -> &::std::option::Option<::std::string::String> {
        &self.url
    }
    /// Span / Range for the Reference
    pub fn recommendation_content_span(mut self, input: crate::types::Span) -> Self {
        self.recommendation_content_span = ::std::option::Option::Some(input);
        self
    }
    /// Span / Range for the Reference
    pub fn set_recommendation_content_span(mut self, input: ::std::option::Option<crate::types::Span>) -> Self {
        self.recommendation_content_span = input;
        self
    }
    /// Span / Range for the Reference
    pub fn get_recommendation_content_span(&self) -> &::std::option::Option<crate::types::Span> {
        &self.recommendation_content_span
    }
    /// Consumes the builder and constructs a [`Reference`](crate::types::Reference).
    pub fn build(self) -> crate::types::Reference {
        crate::types::Reference {
            license_name: self.license_name,
            repository: self.repository,
            url: self.url,
            recommendation_content_span: self.recommendation_content_span,
        }
    }
}
