// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(missing_docs)] // documentation missing in model
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)]
pub struct SsoIdentityDetails {
    #[allow(missing_docs)] // documentation missing in model
    pub instance_arn: ::std::string::String,
    #[allow(missing_docs)] // documentation missing in model
    pub oidc_client_id: ::std::string::String,
    #[allow(missing_docs)] // documentation missing in model
    pub sso_region: ::std::option::Option<::std::string::String>,
}
impl SsoIdentityDetails {
    #[allow(missing_docs)] // documentation missing in model
    pub fn instance_arn(&self) -> &str {
        use std::ops::Deref;
        self.instance_arn.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn oidc_client_id(&self) -> &str {
        use std::ops::Deref;
        self.oidc_client_id.deref()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn sso_region(&self) -> ::std::option::Option<&str> {
        self.sso_region.as_deref()
    }
}
impl SsoIdentityDetails {
    /// Creates a new builder-style object to manufacture
    /// [`SsoIdentityDetails`](crate::types::SsoIdentityDetails).
    pub fn builder() -> crate::types::builders::SsoIdentityDetailsBuilder {
        crate::types::builders::SsoIdentityDetailsBuilder::default()
    }
}

/// A builder for [`SsoIdentityDetails`](crate::types::SsoIdentityDetails).
#[non_exhaustive]
#[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::default::Default, ::std::fmt::Debug)]
pub struct SsoIdentityDetailsBuilder {
    pub(crate) instance_arn: ::std::option::Option<::std::string::String>,
    pub(crate) oidc_client_id: ::std::option::Option<::std::string::String>,
    pub(crate) sso_region: ::std::option::Option<::std::string::String>,
}
impl SsoIdentityDetailsBuilder {
    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn instance_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.instance_arn = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_instance_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.instance_arn = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_instance_arn(&self) -> &::std::option::Option<::std::string::String> {
        &self.instance_arn
    }

    #[allow(missing_docs)] // documentation missing in model
    /// This field is required.
    pub fn oidc_client_id(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.oidc_client_id = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_oidc_client_id(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.oidc_client_id = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_oidc_client_id(&self) -> &::std::option::Option<::std::string::String> {
        &self.oidc_client_id
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn sso_region(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.sso_region = ::std::option::Option::Some(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_sso_region(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.sso_region = input;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_sso_region(&self) -> &::std::option::Option<::std::string::String> {
        &self.sso_region
    }

    /// Consumes the builder and constructs a
    /// [`SsoIdentityDetails`](crate::types::SsoIdentityDetails). This method will fail if any
    /// of the following fields are not set:
    /// - [`instance_arn`](crate::types::builders::SsoIdentityDetailsBuilder::instance_arn)
    /// - [`oidc_client_id`](crate::types::builders::SsoIdentityDetailsBuilder::oidc_client_id)
    pub fn build(
        self,
    ) -> ::std::result::Result<crate::types::SsoIdentityDetails, ::aws_smithy_types::error::operation::BuildError> {
        ::std::result::Result::Ok(crate::types::SsoIdentityDetails {
            instance_arn: self.instance_arn.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "instance_arn",
                    "instance_arn was not specified but it is required when building SsoIdentityDetails",
                )
            })?,
            oidc_client_id: self.oidc_client_id.ok_or_else(|| {
                ::aws_smithy_types::error::operation::BuildError::missing_field(
                    "oidc_client_id",
                    "oidc_client_id was not specified but it is required when building SsoIdentityDetails",
                )
            })?,
            sso_region: self.sso_region,
        })
    }
}
