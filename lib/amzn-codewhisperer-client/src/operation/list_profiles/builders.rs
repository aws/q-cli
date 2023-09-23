// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::list_profiles::_list_profiles_output::ListProfilesOutputBuilder;

pub use crate::operation::list_profiles::_list_profiles_input::ListProfilesInputBuilder;

impl ListProfilesInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::list_profiles::ListProfilesOutput,
        ::aws_smithy_http::result::SdkError<
            crate::operation::list_profiles::ListProfilesError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.list_profiles();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `ListProfiles`.
///
/// Lists one or more CodeWhisperer profiles that you have created.
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct ListProfilesFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::list_profiles::builders::ListProfilesInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::list_profiles::ListProfilesOutput,
        crate::operation::list_profiles::ListProfilesError,
    > for ListProfilesFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::list_profiles::ListProfilesOutput,
            crate::operation::list_profiles::ListProfilesError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl ListProfilesFluentBuilder {
    /// Creates a new `ListProfiles`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the ListProfiles as a reference.
    pub fn as_input(&self) -> &crate::operation::list_profiles::builders::ListProfilesInputBuilder {
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
        crate::operation::list_profiles::ListProfilesOutput,
        ::aws_smithy_http::result::SdkError<
            crate::operation::list_profiles::ListProfilesError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self.inner.build().map_err(::aws_smithy_http::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::list_profiles::ListProfiles::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::list_profiles::ListProfiles::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being
    /// sent.
    // TODO(enableNewSmithyRuntimeCleanup): Remove `async` and `Result` once we switch to orchestrator
    pub async fn customize(
        self,
    ) -> ::std::result::Result<
        crate::client::customize::orchestrator::CustomizableOperation<
            crate::operation::list_profiles::ListProfilesOutput,
            crate::operation::list_profiles::ListProfilesError,
            Self,
        >,
        ::aws_smithy_http::result::SdkError<crate::operation::list_profiles::ListProfilesError>,
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
    /// Create a paginator for this request
    ///
    /// Paginators are used by calling [`send().await`](crate::operation::list_profiles::paginator::ListProfilesPaginator::send) which returns a `Stream`.
    pub fn into_paginator(self) -> crate::operation::list_profiles::paginator::ListProfilesPaginator {
        crate::operation::list_profiles::paginator::ListProfilesPaginator::new(self.handle, self.inner)
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn max_results(mut self, input: i32) -> Self {
        self.inner = self.inner.max_results(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_max_results(mut self, input: ::std::option::Option<i32>) -> Self {
        self.inner = self.inner.set_max_results(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_max_results(&self) -> &::std::option::Option<i32> {
        self.inner.get_max_results()
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn next_token(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.next_token(input.into());
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn set_next_token(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_next_token(input);
        self
    }
    #[allow(missing_docs)] // documentation missing in model
    pub fn get_next_token(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_next_token()
    }
}
