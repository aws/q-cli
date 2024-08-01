// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::unlock_service_linked_role::_unlock_service_linked_role_input::UnlockServiceLinkedRoleInputBuilder;
pub use crate::operation::unlock_service_linked_role::_unlock_service_linked_role_output::UnlockServiceLinkedRoleOutputBuilder;

impl crate::operation::unlock_service_linked_role::builders::UnlockServiceLinkedRoleInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.unlock_service_linked_role();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `UnlockServiceLinkedRole`.
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct UnlockServiceLinkedRoleFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::unlock_service_linked_role::builders::UnlockServiceLinkedRoleInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleOutput,
        crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleError,
    > for UnlockServiceLinkedRoleFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleOutput,
            crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl UnlockServiceLinkedRoleFluentBuilder {
    /// Creates a new `UnlockServiceLinkedRoleFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }

    /// Access the UnlockServiceLinkedRole as a reference.
    pub fn as_input(
        &self,
    ) -> &crate::operation::unlock_service_linked_role::builders::UnlockServiceLinkedRoleInputBuilder {
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
        crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins =
            crate::operation::unlock_service_linked_role::UnlockServiceLinkedRole::operation_runtime_plugins(
                self.handle.runtime_plugins.clone(),
                &self.handle.conf,
                self.config_override,
            );
        crate::operation::unlock_service_linked_role::UnlockServiceLinkedRole::orchestrate(&runtime_plugins, input)
            .await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being
    /// sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleOutput,
        crate::operation::unlock_service_linked_role::UnlockServiceLinkedRoleError,
        Self,
    > {
        crate::client::customize::CustomizableOperation::new(self)
    }

    pub(crate) fn config_override(
        mut self,
        config_override: impl ::std::convert::Into<crate::config::Builder>,
    ) -> Self {
        self.set_config_override(::std::option::Option::Some(config_override.into()));
        self
    }

    pub(crate) fn set_config_override(
        &mut self,
        config_override: ::std::option::Option<crate::config::Builder>,
    ) -> &mut Self {
        self.config_override = config_override;
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn role_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.role_arn(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_role_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_role_arn(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_role_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_role_arn()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn deletion_status(mut self, input: crate::types::DeletionStatus) -> Self {
        self.inner = self.inner.deletion_status(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_deletion_status(mut self, input: ::std::option::Option<crate::types::DeletionStatus>) -> Self {
        self.inner = self.inner.set_deletion_status(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_deletion_status(&self) -> &::std::option::Option<crate::types::DeletionStatus> {
        self.inner.get_deletion_status()
    }
}
