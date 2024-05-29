// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::allow_vended_log_delivery_for_resource::_allow_vended_log_delivery_for_resource_input::AllowVendedLogDeliveryForResourceInputBuilder;
pub use crate::operation::allow_vended_log_delivery_for_resource::_allow_vended_log_delivery_for_resource_output::AllowVendedLogDeliveryForResourceOutputBuilder;

impl crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.allow_vended_log_delivery_for_resource();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `AllowVendedLogDeliveryForResource`.
///
/// Internal API to authorize a CodeWhisperer resource for vended log delivery.
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct AllowVendedLogDeliveryForResourceFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput,
        crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceError,
    > for AllowVendedLogDeliveryForResourceFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput,
            crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl AllowVendedLogDeliveryForResourceFluentBuilder {
    /// Creates a new `AllowVendedLogDeliveryForResource`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }

    /// Access the AllowVendedLogDeliveryForResource as a reference.
    pub fn as_input(&self) -> &crate::operation::allow_vended_log_delivery_for_resource::builders::AllowVendedLogDeliveryForResourceInputBuilder{
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
        crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResource::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResource::orchestrate(
            &runtime_plugins,
            input,
        )
        .await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being
    /// sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceOutput,
        crate::operation::allow_vended_log_delivery_for_resource::AllowVendedLogDeliveryForResourceError,
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
    pub fn resource_arn_being_authorized(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.resource_arn_being_authorized(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_resource_arn_being_authorized(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_resource_arn_being_authorized(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_resource_arn_being_authorized(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_resource_arn_being_authorized()
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn delivery_source_arn(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.delivery_source_arn(input.into());
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn set_delivery_source_arn(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_delivery_source_arn(input);
        self
    }

    #[allow(missing_docs)] // documentation missing in model
    pub fn get_delivery_source_arn(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_delivery_source_arn()
    }
}
