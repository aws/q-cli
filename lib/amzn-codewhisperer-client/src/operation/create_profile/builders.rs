// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_profile::_create_profile_output::CreateProfileOutputBuilder;

pub use crate::operation::create_profile::_create_profile_input::CreateProfileInputBuilder;

impl CreateProfileInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_profile::CreateProfileOutput,
        ::aws_smithy_http::result::SdkError<
            crate::operation::create_profile::CreateProfileError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_profile();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateProfile`.
///
/// Creates a CodeWhisperer profile which can then be associated to users/groups of an identity source
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateProfileFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_profile::builders::CreateProfileInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_profile::CreateProfileOutput,
        crate::operation::create_profile::CreateProfileError,
    > for CreateProfileFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_profile::CreateProfileOutput,
            crate::operation::create_profile::CreateProfileError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateProfileFluentBuilder {
    /// Creates a new `CreateProfile`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateProfile as a reference.
    pub fn as_input(&self) -> &crate::operation::create_profile::builders::CreateProfileInputBuilder {
        &self.inner
    }
    /// Sends the request and returns the response.
    ///
    /// If an error occurs, an `SdkError` will be returned with additional details that
    /// can be matched against.
    ///
    /// By default, any retryable failures will be retried twice. Retry behavior
    /// is configurable with the [RetryConfig](aws_smithy_types::retry::RetryConfig), which can be
    /// set when configuring the client.
    pub async fn send(
        self,
    ) -> ::std::result::Result<
        crate::operation::create_profile::CreateProfileOutput,
        ::aws_smithy_http::result::SdkError<
            crate::operation::create_profile::CreateProfileError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self.inner.build().map_err(::aws_smithy_http::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_profile::CreateProfile::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_profile::CreateProfile::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being
    /// sent.
    // TODO(enableNewSmithyRuntimeCleanup): Remove `async` and `Result` once we switch to orchestrator
    pub async fn customize(
        self,
    ) -> ::std::result::Result<
        crate::client::customize::orchestrator::CustomizableOperation<
            crate::operation::create_profile::CreateProfileOutput,
            crate::operation::create_profile::CreateProfileError,
            Self,
        >,
        ::aws_smithy_http::result::SdkError<crate::operation::create_profile::CreateProfileError>,
    > {
        ::std::result::Result::Ok(crate::client::customize::orchestrator::CustomizableOperation::new(self))
    }
    pub(crate) fn config_override(mut self, config_override: impl Into<crate::config::Builder>) -> Self {
        self.set_config_override(Some(config_override.into()));
        self
    }

    pub(crate) fn set_config_override(&mut self, config_override: Option<crate::config::Builder>) -> &mut Self {
        self.config_override = config_override;
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn identity_source(mut self, input: crate::types::IdentitySource) -> Self {
        self.inner = self.inner.identity_source(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_identity_source(mut self, input: ::std::option::Option<crate::types::IdentitySource>) -> Self {
        self.inner = self.inner.set_identity_source(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_identity_source(&self) -> &::std::option::Option<crate::types::IdentitySource> {
        self.inner.get_identity_source()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn profile_name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.profile_name(input.into());
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_profile_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_profile_name(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_profile_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_profile_name()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn reference_tracker_configuration(mut self, input: crate::types::ReferenceTrackerConfiguration) -> Self {
        self.inner = self.inner.reference_tracker_configuration(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_reference_tracker_configuration(mut self, input: ::std::option::Option<crate::types::ReferenceTrackerConfiguration>) -> Self {
        self.inner = self.inner.set_reference_tracker_configuration(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_reference_tracker_configuration(&self) -> &::std::option::Option<crate::types::ReferenceTrackerConfiguration> {
        self.inner.get_reference_tracker_configuration()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn client_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.client_token(input.into());
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_client_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_client_token(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_client_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_client_token()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn kms_key_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.kms_key_arn(input.into());
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_kms_key_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_kms_key_arn(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_kms_key_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_kms_key_arn()
    }
    /// Appends an item to `tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    #[allow(missing_docs)] // documentation missing in model
    pub fn tags(mut self, input: crate::types::Tag) -> Self {
        self.inner = self.inner.tags(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_tags(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::Tag>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_tags(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::Tag>> {
        self.inner.get_tags()
    }
}
