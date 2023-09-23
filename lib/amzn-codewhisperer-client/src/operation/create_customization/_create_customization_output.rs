// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct CreateCustomizationOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub customization_arn: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl CreateCustomizationOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn customization_arn(&self) -> ::std::option::Option<&str> {
        self.customization_arn.as_deref()
    }
}
impl ::aws_http::request_id::RequestId for CreateCustomizationOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl CreateCustomizationOutput {
    /// Creates a new builder-style object to manufacture [`CreateCustomizationOutput`](crate::operation::create_customization::CreateCustomizationOutput).
    pub fn builder() -> crate::operation::create_customization::builders::CreateCustomizationOutputBuilder {
        crate::operation::create_customization::builders::CreateCustomizationOutputBuilder::default()
    }
}

/// A builder for [`CreateCustomizationOutput`](crate::operation::create_customization::CreateCustomizationOutput).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct CreateCustomizationOutputBuilder {
    pub(crate) customization_arn: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl CreateCustomizationOutputBuilder {
    #[allow(missing_docs)] // documentation missing in model
    pub fn customization_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.customization_arn = ::std::option::Option::Some(input.into());
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_customization_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.customization_arn = input;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_customization_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.customization_arn
    }
    pub(crate) fn _request_id(mut self, request_id: impl Into<String>) -> Self {
        self._request_id = Some(request_id.into());
        self
    }

    pub(crate) fn _set_request_id(&mut self, request_id: Option<String>) -> &mut Self {
        self._request_id = request_id;
        self
    }
    /// Consumes the builder and constructs a [`CreateCustomizationOutput`](crate::operation::create_customization::CreateCustomizationOutput).
    pub fn build(self) -> crate::operation::create_customization::CreateCustomizationOutput {
        crate::operation::create_customization::CreateCustomizationOutput {
            customization_arn: self.customization_arn,
            _request_id: self._request_id,
        }
    }
}
