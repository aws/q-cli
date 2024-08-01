// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct UpdateProfileOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub profile_arn: ::std::string::String,
    _request_id: Option<String>,
}
impl UpdateProfileOutput {
    #[allow(missing_docs)] // documentation missing in model
    pub fn profile_arn(&self) -> &str {
        use std::ops::Deref;
        self.profile_arn.deref()
    }
}
impl ::aws_types::request_id::RequestId for UpdateProfileOutput {
    fn request_id(&self) -> Option<&str> {
        self._request_id.as_deref()
    }
}
impl UpdateProfileOutput {
    /// Creates a new builder-style object to manufacture
    /// [`UpdateProfileOutput`](crate::operation::update_profile::UpdateProfileOutput).
    pub fn builder() -> crate::operation::update_profile::builders::UpdateProfileOutputBuilder {
        crate::operation::update_profile::builders::UpdateProfileOutputBuilder::default()
    }
}

/// A builder for [`UpdateProfileOutput`](crate::operation::update_profile::UpdateProfileOutput).
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct UpdateProfileOutputBuilder {
    pub(crate) profile_arn: ::std::option::Option<::std::string::String>,
    _request_id: Option<String>,
}
impl UpdateProfileOutputBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn profile_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.profile_arn = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_profile_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.profile_arn = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_profile_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.profile_arn
    }

    pub(crate) fn _request_id(mut self, request_id: impl Into<String>) -> Self {
        self._request_id = Some(request_id.into());
        self
    }

    pub(crate) fn _set_request_id(&mut self, request_id: Option<String>) -> &mut Self {
        self._request_id = request_id;
        self
    }

    /// Consumes the builder and constructs a
    /// [`UpdateProfileOutput`](crate::operation::update_profile::UpdateProfileOutput).
    /// This method will fail if any of the following fields are not set:
    /// - [`profile_arn`](crate::operation::update_profile::builders::UpdateProfileOutputBuilder::profile_arn)
    pub fn build(
        self,
    ) -> ::std::result::Result<
        crate::operation::update_profile::UpdateProfileOutput,
        ::aws_smithy_types::error::operation::BuildError,
    > {
        ::std::result::Result::Ok(crate::operation::update_profile::UpdateProfileOutput {
            profile_arn: self.profile_arn.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "profile_arn",
                    "profile_arn was not specified but it is required when building UpdateProfileOutput",
                )
            })?,
            _request_id: self._request_id,
        })
    }
}
